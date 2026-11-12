use std::{error::Error, fmt, thread, time::Duration};

mod com;
mod hotkey;
mod keymap;
mod keys;
mod music;
pub use keymap::KeyLayout;
pub use keys::KeySetting;
mod native;
mod nss;
pub use music::run_music;
mod profile;
mod registry;
pub use hotkey::FnBrightnessEvents;
pub use native::{NativeHid, Supply, run_idle_policy, watch_profile};
pub use nss::{NssKeyGroups, NssProfile, paint_nss_group};
pub use profile::{Direction, Effect, KEY_COUNT, Profile, Rgb};
pub use registry::{
    load_console_profile, load_key_setting, project_aurora_enabled, project_aurora_installed,
    save_console_profile, save_key_setting, set_project_aurora,
};

const VID: u16 = 0x048d;
const PID: u16 = 0x6006;
const USAGE: u16 = 1;
const USAGE_PAGE: u16 = 0xff03;
const FEATURE_LEN: usize = 9;
const OUTPUT_LEN: usize = 65;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct HidError(pub String);
impl fmt::Display for HidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl Error for HidError {}

#[derive(Clone, Copy, Debug)]
pub struct EffectReport {
    pub control: u8,
    pub effect: u8,
    pub speed: u8,
    pub brightness: u8,
    pub color_index: u8,
    pub direction: u8,
    pub save: u8,
}

impl EffectReport {
    pub fn encode(self) -> [u8; FEATURE_LEN] {
        [
            0,
            0x08,
            self.control,
            self.effect,
            self.speed,
            self.brightness,
            self.color_index,
            self.direction,
            self.save,
        ]
    }
}

pub trait Transport {
    fn feature_len(&self) -> usize;
    fn set_feature(&self, report: &[u8]) -> Result<()>;
    fn get_feature(&self, report: &mut [u8]) -> Result<()>;
    fn write_output(&self, report: &[u8]) -> Result<()>;
    fn prepare_key_trigger(&self) -> Result<()> {
        Err(HidError("此传输没有实现 EC 按键触发".into()).into())
    }
    fn enable_key_trigger(&self) -> Result<()> {
        Err(HidError("此传输没有实现 EC 按键触发".into()).into())
    }
}

pub struct KcKeyboard<T: Transport> {
    io: T,
}
impl<T: Transport> KcKeyboard<T> {
    pub fn new(io: T) -> Result<Self> {
        if io.feature_len() != FEATURE_LEN {
            return Err(HidError(format!(
                "HID feature report 长度为 {}，协议要求 {FEATURE_LEN}",
                io.feature_len()
            ))
            .into());
        }
        Ok(Self { io })
    }

    pub fn set_effect(&mut self, effect: EffectReport) -> Result<()> {
        self.io.set_feature(&effect.encode())?;
        thread::sleep(Duration::from_millis(1));
        Ok(())
    }

    pub fn set_brightness(&mut self, level: u8) -> Result<()> {
        if level > 100 {
            return Err(HidError("亮度应为 0..=100".into()).into());
        }
        self.io.set_feature(&[0, 0x09, 2, level, 0, 0, 0, 0, 0])?;
        thread::sleep(Duration::from_millis(1));
        Ok(())
    }

    pub fn set_power(&mut self, enabled: bool) -> Result<()> {
        self.set_power_with_save(enabled, true)
    }

    pub fn set_power_with_save(&mut self, enabled: bool, save: bool) -> Result<()> {
        self.set_timeout_with_save(enabled, 4, save)
    }

    /// Send the factory ITE HID `HID_Set_TimeOut_1AH(enable, time, save)`
    /// command. `time` is the raw firmware byte; its unit is not established
    /// by the available factory console decompilation.
    pub fn set_timeout(&mut self, enabled: bool, time: u8) -> Result<()> {
        self.set_timeout_with_save(enabled, time, true)
    }

    pub fn set_timeout_with_save(&mut self, enabled: bool, time: u8, save: bool) -> Result<()> {
        self.io
            .set_feature(&[0, 0x1a, 0, u8::from(enabled), time, 0, 0, 0, u8::from(save)])?;
        thread::sleep(Duration::from_millis(1));
        Ok(())
    }

    /// Experimental HID query. The console uses its saved profile for config reads.
    pub fn read_effect(&mut self) -> Result<[u8; FEATURE_LEN]> {
        self.query(0x88, 1)
    }

    /// Raw version fields in order: high, low, test, customer.
    pub fn firmware_version(&mut self) -> Result<[u8; 4]> {
        Ok(self.query(0x80, 10)?[2..6].try_into()?)
    }

    fn query(&mut self, command: u8, attempts: usize) -> Result<[u8; FEATURE_LEN]> {
        for i in 0..attempts {
            if i != 0 {
                thread::sleep(Duration::from_millis(20));
            }
            self.io.set_feature(&[0, command, 0, 0, 0, 0, 0, 0, 0])?;
            thread::sleep(Duration::from_millis(1));
            let mut response = [0u8; FEATURE_LEN];
            self.io.get_feature(&mut response)?;
            if response[0] == 0 && response[1] == command {
                return Ok(response);
            }
        }
        Err(HidError(format!("未收到匹配的 HID 0x{command:02x} 应答")).into())
    }

    pub fn set_palette(&mut self, colors: &[Rgb]) -> Result<u8> {
        if colors.len() > 7 {
            return Err(HidError("一组效果最多支持 7 个颜色槽".into()).into());
        }
        if colors.is_empty() {
            return Ok(1);
        }
        let loop_count = if colors.len() == 7 {
            0
        } else {
            colors.len() as u8
        };
        for (i, [r, g, b]) in colors.iter().copied().enumerate() {
            self.io
                .set_feature(&[0, 0x14, loop_count, (i + 1) as u8, r, g, b, 0, 0])?;
            thread::sleep(Duration::from_millis(1));
        }
        Ok(8)
    }

    pub fn set_user_mode(
        &mut self,
        brightness: u8,
        colors: &[Rgb; KEY_COUNT],
        save: u8,
    ) -> Result<()> {
        if brightness > 100 {
            return Err(HidError("亮度应为 0..=100".into()).into());
        }
        self.set_effect(EffectReport {
            control: 2,
            effect: 51,
            speed: 0,
            brightness,
            color_index: 0,
            direction: 0,
            save,
        })?;
        self.write_key_frame(colors)
    }

    /// Update the 126 slots after entering UserMode.
    pub fn write_key_frame(&mut self, colors: &[Rgb; KEY_COUNT]) -> Result<()> {
        // Firmware expects six rows of 21 keys; each row's color planes are B, G, R.
        for row in 0..6usize {
            let mut report = [0u8; OUTPUT_LEN];
            for col in 0..21usize {
                let [r, g, b] = colors[(5 - row) * 21 + col];
                report[2 + col] = b;
                report[23 + col] = g;
                report[44 + col] = r;
            }
            self.io
                .set_feature(&[0, 0x16, 0, row as u8, 0, 0, 0, 0, 0])?;
            self.io.write_output(&report)?;
            thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }
}

impl<T: Transport + ?Sized> Transport for &T {
    fn feature_len(&self) -> usize {
        (**self).feature_len()
    }
    fn set_feature(&self, report: &[u8]) -> Result<()> {
        (**self).set_feature(report)
    }
    fn get_feature(&self, report: &mut [u8]) -> Result<()> {
        (**self).get_feature(report)
    }
    fn write_output(&self, report: &[u8]) -> Result<()> {
        (**self).write_output(report)
    }
    fn prepare_key_trigger(&self) -> Result<()> {
        (**self).prepare_key_trigger()
    }
    fn enable_key_trigger(&self) -> Result<()> {
        (**self).enable_key_trigger()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    struct Mock {
        reports: RefCell<Vec<Vec<u8>>>,
        feature_len: usize,
        output: RefCell<Vec<Vec<u8>>>,
        response: [u8; 9],
    }
    impl Transport for Mock {
        fn feature_len(&self) -> usize {
            self.feature_len
        }
        fn set_feature(&self, r: &[u8]) -> Result<()> {
            self.reports.borrow_mut().push(r.to_vec());
            Ok(())
        }
        fn get_feature(&self, r: &mut [u8]) -> Result<()> {
            r.copy_from_slice(&self.response);
            Ok(())
        }
        fn write_output(&self, r: &[u8]) -> Result<()> {
            self.output.borrow_mut().push(r.to_vec());
            Ok(())
        }
    }
    fn mock() -> Mock {
        Mock {
            reports: RefCell::new(vec![]),
            feature_len: 9,
            output: RefCell::new(vec![]),
            response: [0, 0x88, 2, 51, 0, 75, 1, 0, 0],
        }
    }
    #[test]
    fn effect_encoding_is_exact() {
        assert_eq!(
            EffectReport {
                control: 2,
                effect: 3,
                speed: 7,
                brightness: 80,
                color_index: 1,
                direction: 4,
                save: 1
            }
            .encode(),
            [0, 8, 2, 3, 7, 80, 1, 4, 1]
        );
    }
    #[test]
    fn readback_validates_command_echo() {
        let m = mock();
        let mut k = KcKeyboard::new(m).unwrap();
        assert_eq!(k.read_effect().unwrap()[3], 51);
    }
    #[test]
    fn timeout_command_preserves_raw_parameters_and_save_flag() {
        let m = mock();
        let mut k = KcKeyboard::new(m).unwrap();
        k.set_timeout_with_save(true, 17, false).unwrap();
        k.set_timeout(false, 255).unwrap();
        assert_eq!(
            *k.io.reports.borrow(),
            vec![
                vec![0, 0x1a, 0, 1, 17, 0, 0, 0, 0],
                vec![0, 0x1a, 0, 0, 255, 0, 0, 0, 1]
            ]
        );
    }
    #[test]
    fn user_mode_layout_has_six_rows_and_reversed_order() {
        let m = mock();
        let mut k = KcKeyboard::new(m).unwrap();
        let colors = std::array::from_fn(|i| [i as u8, 0, 0]);
        k.set_user_mode(75, &colors, 1).unwrap();
        let outputs = k.io.output.borrow();
        assert_eq!(outputs.len(), 6);
        assert_eq!(outputs[0][44], 105);
        assert_eq!(outputs[5][44], 0);
    }
    #[test]
    fn rejects_wrong_feature_report_size() {
        let m = Mock {
            feature_len: 8,
            ..mock()
        };
        assert!(KcKeyboard::new(m).is_err());
    }
    #[test]
    fn user_mode_output_report_planes_follow_bgr_layout() {
        let m = mock();
        let mut k = KcKeyboard::new(m).unwrap();
        let colors = [[1u8, 2u8, 3u8]; 126];
        k.set_user_mode(80, &colors, 1).unwrap();
        let reports = k.io.output.borrow();
        assert_eq!(&reports[0][2..23], &[3u8; 21]);
        assert_eq!(&reports[0][23..44], &[2u8; 21]);
        assert_eq!(&reports[0][44..65], &[1u8; 21]);
    }

    #[test]
    fn gaming_profile_uses_dc_and_four_corrected_colors() {
        let mut keyboard = KcKeyboard::new(mock()).unwrap();
        let profile = Profile {
            effect: Effect::GamingMode,
            speed: 3,
            brightness_dc: 62,
            palette: vec![[250, 100, 10], [1, 2, 3]],
            ..Profile::default()
        };
        keyboard.apply_profile(&profile, Supply::Dc, false).unwrap();
        let reports = keyboard.io.reports.borrow();
        assert_eq!(reports.len(), 6);
        assert_eq!(reports[0], [0, 0x1a, 0, 1, 4, 0, 0, 0, 0]);
        for i in 1..=4 {
            assert_eq!(reports[i], [0, 0x14, 4, i as u8, 250, 70, 10, 0, 0]);
        }
        assert_eq!(reports[5], [0, 8, 2, 21, 7, 62, 8, 0, 0]);
        assert_eq!(profile.palette[0], [250, 100, 10]);
    }

    #[test]
    fn key_trigger_requires_ec_before_sending_any_reports() {
        let mut keyboard = KcKeyboard::new(mock()).unwrap();
        let profile = Profile {
            effect: Effect::Ripple,
            direction: Direction::PressAnyKey,
            ..Profile::default()
        };
        assert!(keyboard.apply_profile(&profile, Supply::Ac, true).is_err());
        assert!(keyboard.io.reports.borrow().is_empty());
    }

    #[test]
    fn firmware_fields_are_distinct_from_saved_effect_ids() {
        let mut m = mock();
        m.response = [0, 0x80, 1, 2, 3, 4, 0, 0, 0];
        let mut keyboard = KcKeyboard::new(m).unwrap();
        assert_eq!(keyboard.firmware_version().unwrap(), [1, 2, 3, 4]);
        assert!(keyboard.read_effect().is_err());
        assert_eq!(Effect::from_console(6).unwrap().hid_code(), 10);
        assert_eq!(Effect::from_console(7).unwrap().hid_code(), 9);
    }

    #[test]
    fn named_key_paints_all_slots_and_round_trips_through_json() {
        let mut profile = Profile::default();
        let (_, enter) = KeyLayout::Iso
            .keys()
            .iter()
            .find(|(key, _)| *key == "ENTER")
            .unwrap();
        assert!(enter.len() > 1);
        profile
            .paint_key(KeyLayout::Iso, "enter", [11, 22, 33])
            .unwrap();
        assert_eq!(profile.effect, Effect::UserMode);
        for (i, &color) in profile.key_colors.iter().enumerate() {
            assert_eq!(
                color,
                if enter.contains(&i) {
                    [11, 22, 33]
                } else {
                    [0; 3]
                }
            );
        }
        let mut json = Vec::new();
        profile.write_json(&mut json).unwrap();
        assert_eq!(Profile::from_json(json.as_slice()).unwrap(), profile);
    }
}
