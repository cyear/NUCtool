use crate::{
    Effect, EffectReport, FnBrightnessEvents, HidError, KEY_COUNT, KcKeyboard, Profile, Result,
    Rgb, Transport,
};
use crate::{com::Com, native::RuntimeBrightness, profile::console_color};
use std::{
    thread,
    time::{Duration, Instant},
};
use windows::Win32::{
    Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
    Media::Audio::{
        DEVICE_STATE_ACTIVE, Endpoints::IAudioMeterInformation, IMMDevice, IMMDeviceEnumerator,
        MMDeviceEnumerator, eRender,
    },
    System::Com::{
        CLSCTX_ALL, CoCreateInstance, CoTaskMemFree, STGM_READ,
        StructuredStorage::{PropVariantClear, PropVariantToStringAlloc},
    },
};

/// Render the selected effect using the first playing non-THX render endpoint.
/// While waiting for playback, the callback remains responsive.
pub fn run_music<T: Transport>(
    keyboard: &mut KcKeyboard<T>,
    profile: &Profile,
    stop: impl FnMut() -> bool,
) -> Result<()> {
    watch_music(keyboard, &mut profile.clone(), None, stop)
}

pub(crate) fn watch_music<T: Transport>(
    keyboard: &mut KcKeyboard<T>,
    profile: &mut Profile,
    events: Option<&FnBrightnessEvents>,
    mut stop: impl FnMut() -> bool,
) -> Result<()> {
    profile.validate()?;
    if profile.effect != Effect::Music {
        return Err(HidError("音乐运行器需要 music 配置".into()).into());
    }
    if !profile.power {
        return keyboard.set_power_with_save(false, false);
    }

    let _com = Com::initialize()?;
    let mut endpoints = Meter::render_endpoints()?;
    let mut policy = RuntimeBrightness::new(events);
    let mut brightness = policy.update(profile)?;
    let blank = [[0; 3]; KEY_COUNT];
    let palette = music_palette(&profile.palette);
    keyboard.set_power_with_save(true, false)?;
    keyboard.set_effect(EffectReport {
        control: 2,
        effect: 34,
        speed: 0,
        brightness,
        color_index: 0,
        direction: 0,
        save: 0,
    })?;
    keyboard.set_user_mode(brightness, &blank, 0)?;

    let mut animator = Animator::new(profile);
    let mut previous = blank;
    let mut selected = None;
    let mut scan_again = Instant::now();
    let mut repeat_sample = false;
    while !stop() {
        if selected.is_none() && Instant::now() >= scan_again {
            for (index, endpoint) in endpoints.iter().enumerate() {
                if unsafe { endpoint.api.GetPeakValue()? } > 0.0 {
                    selected = Some(index);
                    break;
                }
            }
            scan_again = Instant::now() + Duration::from_secs(1);
        }
        let desired = policy.update(profile)?;
        if desired != brightness {
            keyboard.set_brightness(desired)?;
            brightness = desired;
        }
        let frame = match selected {
            Some(index) if brightness != 0 => {
                let started = Instant::now();
                let mut samples = Vec::with_capacity(animator.columns().div_ceil(2) + 1);
                while samples.len() * 2 < animator.columns() {
                    let pair = endpoints[index].peaks()?;
                    samples.push(pair);
                    if repeat_sample {
                        samples.push(pair);
                    }
                }
                thread::sleep(Duration::from_millis(30));
                repeat_sample = started.elapsed() >= Duration::from_millis(40);
                animator.render(&samples, &palette)
            }
            _ => {
                thread::sleep(Duration::from_millis(30));
                blank
            }
        };
        if frame != previous {
            keyboard.write_key_frame(&frame)?;
            previous = frame;
        }
    }
    keyboard.write_key_frame(&blank)
}

struct Meter {
    api: IAudioMeterInformation,
    channels: Vec<f32>,
}
impl Meter {
    fn render_endpoints() -> Result<Vec<Self>> {
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let devices = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
            let mut meters = Vec::new();
            for index in 0..devices.GetCount()? {
                let device = devices.Item(index)?;
                if friendly_name(&device)?.to_uppercase().contains("THX") {
                    continue;
                }
                let api: IAudioMeterInformation = device.Activate(CLSCTX_ALL, None)?;
                let channels = vec![0.0; api.GetMeteringChannelCount()? as usize];
                meters.push(Self { api, channels });
            }
            if meters.is_empty() {
                return Err(HidError("没有可用的非 THX 音频输出端点".into()).into());
            }
            Ok(meters)
        }
    }

    fn peaks(&mut self) -> Result<[f32; 2]> {
        if self.channels.len() < 2 {
            return Ok([unsafe { self.api.GetPeakValue()? }; 2]);
        }
        unsafe { self.api.GetChannelsPeakValues(&mut self.channels)? };
        Ok([self.channels[0], self.channels[1]])
    }
}

fn friendly_name(device: &IMMDevice) -> Result<String> {
    unsafe {
        let store = device.OpenPropertyStore(STGM_READ)?;
        let mut property = store.GetValue(&PKEY_Device_FriendlyName)?;
        let name = PropVariantToStringAlloc(&property);
        let clear = PropVariantClear(&mut property);
        let name = name?;
        let text = name.to_string();
        CoTaskMemFree(Some(name.0.cast()));
        clear?;
        Ok(text?)
    }
}

fn music_palette(palette: &[Rgb]) -> [Rgb; 7] {
    let indexes = match palette.len() {
        0 => return [[0; 3]; 7],
        1 => [0; 7],
        2 => [0, 0, 0, 1, 1, 1, 1],
        3 => [0, 0, 1, 1, 2, 2, 2],
        4 => [0, 1, 1, 2, 2, 3, 3],
        5 => [0, 0, 1, 2, 3, 4, 4],
        6 => [0, 1, 2, 3, 4, 5, 5],
        _ => [0, 1, 2, 3, 4, 5, 6],
    };
    indexes.map(|i| console_color(palette[i]))
}

/// Reproduce `Translate_ITE_SpeedValue` followed by the factory
/// `AdjustMeterParam` table. The profile stores UI speed (0..=9).
fn factory_meter_speed(ui_speed: u8) -> f32 {
    match ui_speed {
        3 => 1.5,
        5 => 2.5,
        7 => 3.5,
        9 => 4.5,
        _ => 0.5,
    }
}

struct Animator {
    direction: u8,
    exponent: f64,
    min: f32,
    max: f32,
}
impl Animator {
    fn new(profile: &Profile) -> Self {
        let direction = profile.direction.hid_code();
        let meter_speed = factory_meter_speed(profile.speed);
        let base = if matches!(direction, 1 | 2) { 1.5 } else { 1.3 };
        Self {
            direction,
            exponent: base + f64::from(meter_speed * 0.1 + 0.1),
            min: f32::MAX,
            max: 0.0,
        }
    }

    fn columns(&self) -> usize {
        if matches!(self.direction, 1 | 2) {
            12
        } else {
            19
        }
    }

    fn pack_levels(&self, samples: &[[f32; 2]]) -> Vec<f32> {
        let columns = self.columns();
        let regress = |n: f32| (f64::from(n * 10.0).powf(self.exponent) as f32) / 2.0;
        let combined = || {
            prepare_meter(
                samples.iter().flatten().copied().collect(),
                columns,
                columns,
            )
        };
        if self.direction == 4 {
            let mut levels: Vec<_> = combined().into_iter().map(regress).collect();
            levels.sort_by(f32::total_cmp);
            let Some(&peak) = levels.last() else {
                return levels;
            };
            let mut packed = levels[..levels.len() / 2].to_vec();
            packed.extend([peak; 2]);
            packed.extend(levels.iter().rev().skip(levels.len() / 3).copied());
            return packed;
        }

        let channel =
            |i| prepare_meter(samples.iter().map(|s| s[i]).collect(), columns / 2, columns);
        let mut left = channel(0);
        let mut right = channel(1);
        if matches!(self.direction, 0 | 3)
            && let Some(last) = combined().last()
        {
            right.push(*last);
        }
        if self.direction == 1 {
            std::mem::swap(&mut left, &mut right);
        }
        if left.len() > 9 {
            left.truncate(10);
            right.truncate(9);
        }
        for group in [&mut left, &mut right] {
            group.iter_mut().for_each(|level| *level = regress(*level));
            group.sort_by(f32::total_cmp);
        }
        left.extend(right);
        left
    }

    fn render(&mut self, samples: &[[f32; 2]], palette: &[Rgb; 7]) -> [Rgb; KEY_COUNT] {
        let mut frame = [[0; 3]; KEY_COUNT];
        let levels = self.pack_levels(samples);
        if matches!(self.direction, 1 | 2) {
            let peak = levels.iter().copied().fold(0.0, f32::max);
            if peak < 0.005 {
                self.min = f32::MAX;
                self.max = 0.0;
                return frame;
            }
            self.min = self
                .min
                .min(levels.iter().copied().fold(f32::MAX, f32::min));
            self.max = self.max.max(peak);
            let span = self.max - self.min;
            let mean =
                (levels.iter().map(|&n| f64::from(n)).sum::<f64>() / levels.len() as f64) as f32;
            let length = if span > 0.0 {
                ((9.0 / span * 4.0 / 3.0) * mean).round_ties_even() as usize
            } else {
                0
            };
            draw_horizontal(&mut frame, palette, self.direction == 1, length);
            return frame;
        }

        for (column, level) in levels.into_iter().skip(1).take(21).enumerate() {
            let height = (level.round_ties_even() as usize).min(6);
            for (band, &color) in palette.iter().take(height).enumerate() {
                let row = if self.direction == 3 { band } else { 5 - band };
                if column < 20 || row == 5 {
                    frame[row * 21 + column] = color;
                }
            }
        }
        frame
    }
}

// AudioData.MeterBufferProcess pads short channel buffers to the combined size,
// but trims long channel buffers to the per-channel size.
fn prepare_meter(mut buffer: Vec<f32>, limit: usize, columns: usize) -> Vec<f32> {
    let floor = if buffer.len() > 1 {
        (f64::from(buffer.iter().copied().fold(0.0, f32::max)) * 0.6) as f32
    } else {
        0.0
    };
    if buffer.len() > limit {
        buffer.drain(..buffer.len() - limit);
    } else if !buffer.is_empty() && buffer.len() < limit {
        let fill = if buffer.len() > 1 {
            let mut sorted = buffer.clone();
            sorted.sort_by(|a, b| b.total_cmp(a));
            let index = 0.75 * (sorted.len() - 1) as f64;
            let a = f64::from(sorted[index.floor() as usize]);
            let b = f64::from(sorted[index.ceil() as usize]);
            (a + index.fract() * (b - a)) as f32
        } else {
            0.0
        };
        buffer.resize(columns, fill);
    }
    for value in &mut buffer {
        *value = value.max(floor);
    }
    buffer
}

fn draw_horizontal(frame: &mut [Rgb; KEY_COUNT], palette: &[Rgb; 7], center: bool, length: usize) {
    for row in frame.as_chunks_mut::<21>().0 {
        let (forward, reverse) = if center { (8, 8) } else { (0, 15) };
        for distance in 0..length.min(if center { 10 } else { 9 }) {
            row[forward + distance] = palette[distance.saturating_sub(1).min(6)];
        }
        for distance in 0..length.min(9) {
            row[reverse - distance] = palette[distance.saturating_sub(1).min(6)];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Direction;

    #[test]
    fn meter_trim_pad_and_floor_follow_factory_order() {
        // The loud sample is removed by the trim, but still contributes to the floor.
        let trimmed = prepare_meter(vec![0.8, 0.1, 0.5], 2, 4);
        assert!((trimmed[0] - 0.48).abs() < 1e-6);
        assert_eq!(trimmed[1], 0.5);

        // A short channel uses the descending 75th percentile, then the 60% floor.
        let padded = prepare_meter(vec![0.8, 0.7, 0.4], 4, 6);
        assert_eq!(padded.len(), 6);
        assert!((padded[3] - 0.55).abs() < 1e-6);
        assert_eq!(padded[3], padded[5]);
    }

    #[test]
    fn music_packs_stereo_and_normal_with_different_lengths() {
        let input = [[0.1, 0.3]; 10];
        let stereo = Animator::new(&Profile::default()).pack_levels(&input);
        assert_eq!(stereo.len(), 19);
        assert_eq!(&stereo[..9], &[0.5; 9]);
        assert!(stereo[9..].iter().all(|&level| level > 0.5));

        let normal = Animator::new(&Profile {
            direction: Direction::Down,
            ..Profile::default()
        })
        .pack_levels(&input);
        assert_eq!(normal.len(), 24);
        assert_eq!(normal[9], normal[10]);
    }

    #[test]
    fn music_vertical_direction_reverses_rows() {
        let palette = music_palette(&[[20, 30, 40], [1, 2, 3]]);
        let input = [[0.2, 0.2]; 10];
        let mut top = Animator::new(&Profile {
            direction: Direction::Up,
            ..Profile::default()
        });
        let mut bottom = Animator::new(&Profile::default());
        let top = top.render(&input, &palette);
        let bottom = bottom.render(&input, &palette);
        assert_eq!(top[0], palette[0]);
        assert_eq!(bottom[105], palette[0]);
        for row in 0..6 {
            assert_eq!(
                top[row * 21..(row + 1) * 21],
                bottom[(5 - row) * 21..(6 - row) * 21]
            );
        }
    }

    #[test]
    fn center_horizontal_bars_grow_outward_from_the_center_column() {
        let palette = std::array::from_fn(|i| [i as u8, 0, 0]);
        let mut frame = [[0; 3]; KEY_COUNT];
        draw_horizontal(&mut frame, &palette, true, 4);
        let row = &frame[..21];
        assert_eq!(row[8], palette[0]);
        assert_eq!(row[9], palette[0]);
        assert_eq!(row[10], palette[1]);
        assert_eq!(row[7], palette[0]);
        assert_eq!(row[6], palette[1]);
        assert_eq!(row[15], [0; 3]);
    }

    #[test]
    fn music_palette_expansion_matches_factory_color_buffer_for_one_to_seven_colors() {
        let colors = [
            [10, 0, 0],
            [20, 0, 0],
            [30, 0, 0],
            [40, 0, 0],
            [50, 0, 0],
            [60, 0, 0],
            [70, 0, 0],
        ];
        let indexes = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 1, 1, 1, 1],
            [0, 0, 1, 1, 2, 2, 2],
            [0, 1, 1, 2, 2, 3, 3],
            [0, 0, 1, 2, 3, 4, 4],
            [0, 1, 2, 3, 4, 5, 5],
            [0, 1, 2, 3, 4, 5, 6],
        ];
        for count in 1..=7 {
            assert_eq!(
                music_palette(&colors[..count]),
                indexes[count - 1].map(|index| colors[index])
            );
        }
        assert_eq!(music_palette(&[]), [[0; 3]; 7]);
    }

    #[test]
    fn music_speed_uses_factory_adjust_meter_param_after_ui_conversion() {
        let expected = [0.5, 0.5, 0.5, 1.5, 0.5, 2.5, 0.5, 3.5, 0.5, 4.5];
        for (speed, expected) in expected.into_iter().enumerate() {
            assert_eq!(factory_meter_speed(speed as u8), expected);
        }
    }
}
