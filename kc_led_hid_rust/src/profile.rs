use crate::{EffectReport, HidError, KcKeyboard, Result, Supply, Transport};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    str::FromStr,
};

pub const KEY_COUNT: usize = 126;
pub type Rgb = [u8; 3];

macro_rules! choices {
    ($name:ident { $($item:ident => $label:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[repr(u32)]
        pub enum $name { $(#[serde(rename = $label)] $item),+ }

        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$item),+];

            pub fn name(self) -> &'static str {
                match self { $(Self::$item => $label),+ }
            }

            pub fn from_console(value: u32) -> Result<Self> {
                Self::ALL.get(value as usize).copied().ok_or_else(|| {
                    HidError(format!("未知的原厂 {} 值 {value}", stringify!($name))).into()
                })
            }
        }

        impl FromStr for $name {
            type Err = Box<dyn std::error::Error>;

            fn from_str(value: &str) -> Result<Self> {
                Self::ALL.iter().copied().find(|item| item.name().eq_ignore_ascii_case(value))
                    .ok_or_else(|| HidError(format!("未知的 {} 名称 {value}", stringify!($name))).into())
            }
        }
    };
}

// Discriminants are the IntelSDK registry enums, independent of the HID bytes.
choices!(Effect {
    Monocolor => "mono", Breathing => "breath", Wave => "wave",
    Reactive => "reactive", Rainbow => "rainbow", Ripple => "ripple",
    Raindrop => "raindrop", Marquee => "marquee", Aurora => "aurora",
    Spark => "spark", Music => "music", UserMode => "user", GamingMode => "gaming",
});
choices!(Direction {
    None => "none", Right => "right", Left => "left", Up => "up", Down => "down",
    Auto => "auto", PressAnyKey => "press-any-key",
});

impl Effect {
    pub fn hid_code(self) -> u8 {
        match self {
            Self::Monocolor => 1,
            Self::Breathing => 2,
            Self::Wave => 3,
            Self::Reactive => 4,
            Self::Rainbow => 5,
            Self::Ripple => 6,
            Self::Raindrop => 10,
            Self::Marquee => 9,
            Self::Aurora => 14,
            Self::Spark => 17,
            Self::Music => 34,
            Self::UserMode => 51,
            Self::GamingMode => 21,
        }
    }
}

impl Direction {
    pub fn hid_code(self) -> u8 {
        match self {
            Self::None | Self::Auto => 0,
            Self::Right | Self::PressAnyKey => 1,
            Self::Left => 2,
            Self::Up => 3,
            Self::Down => 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Profile {
    pub power: bool,
    pub effect: Effect,
    pub brightness_ac: u8,
    pub brightness_dc: u8,
    pub idle_ac: u32,
    pub idle_dc: u32,
    /// Console speed in 0..=9.
    pub speed: u8,
    pub direction: Direction,
    pub palette: Vec<Rgb>,
    /// Empty uses the console's all-black default. Otherwise exactly 126 slots.
    pub key_colors: Vec<Rgb>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            power: true,
            effect: Effect::Monocolor,
            brightness_ac: 100,
            brightness_dc: 0,
            idle_ac: 0,
            idle_dc: 0,
            speed: 0,
            direction: Direction::None,
            palette: vec![[255; 3]],
            key_colors: Vec::new(),
        }
    }
}

impl Profile {
    /// Build the KC preset values exposed by the factory console's
    /// `KeyboardLedCustomSettings.BuildDefaultPatterns` implementation.
    pub fn factory_preset(effect: Effect) -> Self {
        let mut profile = Self {
            effect,
            brightness_ac: 100,
            brightness_dc: 50,
            idle_ac: 0,
            idle_dc: 30,
            ..Self::default()
        };
        match effect {
            Effect::Monocolor => {
                profile.speed = 0;
                profile.direction = Direction::None;
                profile.palette = vec![[255, 255, 255]];
            }
            Effect::GamingMode => {
                profile.speed = 0;
                profile.direction = Direction::None;
                profile.palette = vec![[255, 0, 0]];
            }
            Effect::Rainbow => {
                profile.speed = 0;
                profile.direction = Direction::None;
                profile.palette = vec![
                    [255, 0, 0],
                    [255, 165, 0],
                    [255, 255, 0],
                    [0, 128, 0],
                    [0, 0, 255],
                    [75, 0, 130],
                    [238, 130, 238],
                ];
            }
            Effect::Breathing
            | Effect::Raindrop
            | Effect::Marquee
            | Effect::Wave
            | Effect::Reactive
            | Effect::Ripple
            | Effect::Spark
            | Effect::Aurora => {
                profile.speed = 5;
                profile.palette = vec![
                    [255, 0, 0],
                    [255, 165, 0],
                    [255, 255, 0],
                    [0, 128, 0],
                    [0, 0, 255],
                    [75, 0, 130],
                    [238, 130, 238],
                ];
                profile.direction = match effect {
                    Effect::Wave => Direction::Right,
                    Effect::Reactive | Effect::Ripple | Effect::Spark | Effect::Aurora => {
                        Direction::Auto
                    }
                    _ => Direction::None,
                };
            }
            Effect::Music => {
                profile.speed = 0;
                profile.direction = Direction::None;
                profile.palette = vec![
                    [255, 0, 0],
                    [255, 165, 0],
                    [255, 255, 0],
                    [0, 128, 0],
                    [0, 0, 255],
                    [75, 0, 130],
                    [238, 130, 238],
                ];
            }
            Effect::UserMode => {
                profile.speed = 0;
                profile.direction = Direction::None;
                profile.key_colors = vec![[0, 0, 0]; KEY_COUNT];
            }
        }
        profile
    }

    pub fn validate(&self) -> Result<()> {
        if self.brightness_ac > 100 || self.brightness_dc > 100 || self.speed > 9 {
            return Err(HidError("亮度范围为 0..=100，配置速度范围为 0..=9".into()).into());
        }
        if self.idle_ac > i32::MAX as u32 || self.idle_dc > i32::MAX as u32 {
            return Err(HidError("空闲秒数超出原厂配置范围".into()).into());
        }
        if self.palette.len() > 7
            || (!self.key_colors.is_empty() && self.key_colors.len() != KEY_COUNT)
        {
            return Err(HidError("调色板最多 7 色，逐键配置必须为空或包含 126 色".into()).into());
        }
        if self.effect == Effect::GamingMode && self.palette.is_empty() {
            return Err(HidError("游戏模式需要至少一个颜色".into()).into());
        }
        Ok(())
    }

    pub fn from_json(reader: impl Read) -> Result<Self> {
        let profile: Self = serde_json::from_reader(reader)?;
        profile.validate()?;
        Ok(profile)
    }

    pub fn write_json(&self, mut writer: impl Write) -> Result<()> {
        self.validate()?;
        serde_json::to_writer_pretty(&mut writer, self)?;
        writeln!(writer)?;
        Ok(())
    }

    pub fn brightness(&self, supply: Supply) -> u8 {
        match supply {
            Supply::Ac => self.brightness_ac,
            Supply::Dc => self.brightness_dc,
        }
    }

    /// Apply the factory Fn-key brightness cycle to one power source.
    pub fn cycle_brightness(&mut self, supply: Supply) -> u8 {
        let level = match supply {
            Supply::Ac => &mut self.brightness_ac,
            Supply::Dc => &mut self.brightness_dc,
        };
        *level = match *level {
            0..50 => 50,
            50..100 => 100,
            _ => 0,
        };
        *level
    }

    pub fn idle_seconds(&self, supply: Supply) -> u32 {
        match supply {
            Supply::Ac => self.idle_ac,
            Supply::Dc => self.idle_dc,
        }
    }
}

// UWLedKeyboard applies this to outgoing colors. Keep the saved profile intact.
pub(crate) fn console_color([r, mut g, b]: Rgb) -> Rgb {
    if r > 230 && b < 20 {
        g = (f64::from(g) * 0.7) as u8;
    }
    [r, g, b]
}

impl<T: Transport> KcKeyboard<T> {
    pub fn apply_profile(&mut self, profile: &Profile, supply: Supply, save: bool) -> Result<()> {
        profile.validate()?;
        if profile.power && profile.effect == Effect::Music {
            return Err(HidError("音乐效果需要持续运行，请调用 run_music".into()).into());
        }
        let effect = profile.effect.hid_code();
        let direction = profile.direction.hid_code();
        if profile.power && key_trigger(effect, direction) {
            self.io.prepare_key_trigger()?;
        }
        self.set_power_with_save(profile.power, save)?;
        if !profile.power {
            return Ok(());
        }
        let brightness = profile.brightness(supply);
        if profile.effect == Effect::UserMode {
            let mut colors = [[0; 3]; KEY_COUNT];
            for (slot, rgb) in colors.iter_mut().zip(&profile.key_colors) {
                *slot = console_color(*rgb);
            }
            return self.set_user_mode(brightness, &colors, u8::from(save));
        }
        let mut palette: Vec<_> = profile.palette.iter().copied().map(console_color).collect();
        if profile.effect == Effect::GamingMode {
            palette = vec![palette[0]; 4];
        }
        self.send_firmware_effect(
            EffectReport {
                control: 2,
                effect,
                speed: 10 - profile.speed,
                brightness,
                color_index: 1,
                direction,
                save: u8::from(save),
            },
            &palette,
        )
    }

    pub fn set_firmware_effect(
        &mut self,
        effect: u8,
        speed: u8,
        brightness: u8,
        direction: u8,
        palette: &[Rgb],
        save: bool,
    ) -> Result<()> {
        if speed > 10 || brightness > 100 || direction > 4 {
            return Err(HidError("HID 速度、亮度或方向超出范围".into()).into());
        }
        if matches!(effect, 34 | 51 | 255) {
            return Err(HidError("请选择固件效果，音乐和逐键模式有独立接口".into()).into());
        }
        if key_trigger(effect, direction) {
            self.io.prepare_key_trigger()?;
        }
        self.send_firmware_effect(
            EffectReport {
                control: 2,
                effect,
                speed,
                brightness,
                color_index: 1,
                direction,
                save: u8::from(save),
            },
            palette,
        )
    }

    fn send_firmware_effect(&mut self, mut report: EffectReport, palette: &[Rgb]) -> Result<()> {
        report.color_index = self.set_palette(palette)?;
        self.set_effect(report)?;
        if key_trigger(report.effect, report.direction) {
            self.io.enable_key_trigger()?;
        }
        Ok(())
    }
}

fn key_trigger(effect: u8, direction: u8) -> bool {
    direction == 1 && matches!(effect, 4 | 6 | 14 | 17)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_presets_match_console_defaults_and_are_valid() {
        for effect in Effect::ALL {
            let profile = Profile::factory_preset(*effect);
            profile.validate().unwrap();
            assert_eq!(profile.brightness_ac, 100);
            assert_eq!(profile.brightness_dc, 50);
            assert_eq!(profile.idle_ac, 0);
            assert_eq!(profile.idle_dc, 30);
            assert_eq!(profile.effect, *effect);
        }
        let wave = Profile::factory_preset(Effect::Wave);
        assert_eq!(wave.direction, Direction::Right);
        assert_eq!(wave.speed, 5);
        assert_eq!(wave.palette[0], [255, 0, 0]);
        let reactive = Profile::factory_preset(Effect::Reactive);
        assert_eq!(reactive.direction, Direction::Auto);
        assert_eq!(reactive.palette.len(), 7);
        assert_eq!(
            Profile::factory_preset(Effect::Monocolor).palette,
            vec![[255; 3]]
        );
        assert_eq!(
            Profile::factory_preset(Effect::GamingMode).palette,
            vec![[255, 0, 0]]
        );
        assert_eq!(
            Profile::factory_preset(Effect::UserMode).key_colors,
            vec![[0; 3]; KEY_COUNT]
        );
    }
}
