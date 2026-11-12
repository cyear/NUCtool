use kc_led_hid::{Direction, Effect, KcKeyboard, NativeHid, Profile, Supply};
use serde::Serialize;

#[derive(Serialize)]
pub struct KeyboardHidInfo {
    connected: bool,
    firmware: String,
    layout: String,
    power: Option<bool>,
    effect: String,
    effect_id: u8,
    brightness_ac: Option<u8>,
    brightness_dc: Option<u8>,
    brightness_live: Option<u8>,
    speed: u8,
    direction: String,
    palette: Vec<[u8; 3]>,
    idle_ac: u32,
    idle_dc: u32,
    effect_live: Option<Vec<u8>>,
    error: Option<String>,
}

#[tauri::command]
pub fn get_keyboard_hid_info() -> Result<KeyboardHidInfo, String> {
    let mut keyboard = KcKeyboard::new(NativeHid::open().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let version = keyboard.firmware_version().map_err(|e| e.to_string())?;
    let layout = keyboard.keyboard_layout().map_err(|e| e.to_string())?;
    let effect_live = keyboard.read_effect().ok();
    let brightness_live = effect_live.map(|r| r[5]);
    let effect_code = effect_live.map(|r| r[3]);
    let effect = effect_code
        .and_then(|code| Effect::ALL.iter().copied().find(|effect| effect.hid_code() == code));
    Ok(KeyboardHidInfo {
        connected: true,
        firmware: format!("{}.{}.{}.{}", version[0], version[1], version[2], version[3]),
        layout: match layout { 85 => "US", 86 => "ISO", 87 => "JIS", 88 => "88-key", _ => "Unknown" }.into(),
        power: brightness_live.map(|value| value > 0),
        effect: effect.map(|effect| effect.name()).unwrap_or("unknown").into(),
        effect_id: effect_code.unwrap_or(0),
        brightness_ac: brightness_live,
        brightness_dc: brightness_live,
        brightness_live,
        speed: effect_live.map(|r| 10u8.saturating_sub(r[4])).unwrap_or(0),
        direction: match effect_live.map(|r| r[7]).unwrap_or(0) {
            1 => "right", 2 => "left", 3 => "up", 4 => "down", _ => "none",
        }.into(),
        palette: Vec::new(),
        idle_ac: 0,
        idle_dc: 0,
        effect_live: effect_live.map(|r| r.to_vec()),
        error: None,
    })
}

#[tauri::command]
pub fn set_keyboard_hid_power(enabled: bool, restore_level: u8) -> Result<(), String> {
    let level = if enabled { restore_level } else { 0 };
    let mut keyboard = open()?;
    keyboard.set_brightness(level).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_keyboard_hid_brightness(level: u8) -> Result<(), String> {
    let mut keyboard = open()?;
    keyboard.set_brightness(level).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_keyboard_hid_effect(
    effect: String, speed: u8, brightness: u8, direction: String,
    colors: Vec<[u8; 3]>, save: bool,
) -> Result<(), String> {
    let effect = effect.parse::<Effect>().map_err(|e| e.to_string())?;
    let direction = direction.parse::<Direction>().map_err(|e| e.to_string())?;
    if effect == Effect::UserMode {
        return Err("扩展逐键灯效请使用扩展灯效写入操作".into());
    }
    let profile = Profile {
        effect, speed, direction, palette: colors,
        brightness_ac: brightness, brightness_dc: brightness,
        ..Profile::default()
    };
    let mut keyboard = open()?;
    keyboard.apply_profile(&profile, Supply::current().map_err(|e| e.to_string())?, save)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_keyboard_hid_extended(colors: Vec<[u8; 3]>, brightness: u8, save: bool) -> Result<(), String> {
    if colors.len() != kc_led_hid::KEY_COUNT {
        return Err(format!("扩展灯效需要 {} 个键位颜色", kc_led_hid::KEY_COUNT));
    }
    let mut profile = Profile::factory_preset(Effect::UserMode);
    profile.brightness_ac = brightness;
    profile.brightness_dc = brightness;
    profile.key_colors = colors;
    let mut keyboard = open()?;
    keyboard.apply_profile(&profile, Supply::current().map_err(|e| e.to_string())?, save)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_keyboard_hid_timeout(enabled: bool, raw_time: u8, save: bool) -> Result<(), String> {
    let mut keyboard = open()?;
    keyboard.set_timeout_with_save(enabled, raw_time, save).map_err(|e| e.to_string())
}

fn open() -> Result<KcKeyboard<NativeHid>, String> {
    let hid = NativeHid::open().map_err(|e| e.to_string())?;
    KcKeyboard::new(hid).map_err(|e| e.to_string())
}
