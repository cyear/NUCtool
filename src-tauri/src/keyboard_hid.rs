use kc_led_hid::{Direction, Effect, KeyLayout, KcKeyboard, NativeHid, Profile, Supply};
use serde::Serialize;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::atomic::AtomicBool;
use tauri::{AppHandle, Manager};

static LAST_IDLE_MINUTE: AtomicU32 = AtomicU32::new(u32::MAX);
static HID_STATUS_POLL_PAUSED: AtomicBool = AtomicBool::new(false);

#[derive(Serialize)]
pub struct KeyboardHidInfo {
    connected: bool,
    firmware: String,
    layout: String,
    keymap: Vec<Option<String>>,
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
pub fn get_keyboard_hid_info(app: AppHandle) -> Result<Option<KeyboardHidInfo>, String> {
    let Some(window) = app.get_webview_window("main") else {
        if !HID_STATUS_POLL_PAUSED.swap(true, Ordering::Relaxed) {
            println!("[KeyboardHID] 主窗口不可用，暂停设备状态读取");
        }
        return Ok(None);
    };
    let visible = window.is_visible().map_err(|error| {
        eprintln!("[KeyboardHID] 查询主窗口可见状态失败，跳过本轮状态读取: {error}");
        error.to_string()
    })?;
    let minimized = window.is_minimized().map_err(|error| {
        eprintln!("[KeyboardHID] 查询主窗口最小化状态失败，跳过本轮状态读取: {error}");
        error.to_string()
    })?;
    if !visible || minimized {
        if !HID_STATUS_POLL_PAUSED.swap(true, Ordering::Relaxed) {
            println!("[KeyboardHID] 主窗口已隐藏或最小化，暂停 HID 状态读取");
        }
        return Ok(None);
    }
    if HID_STATUS_POLL_PAUSED.swap(false, Ordering::Relaxed) {
        println!("[KeyboardHID] 主窗口已恢复显示，继续 HID 状态读取");
    }

    println!("[KeyboardHID] 开始探测兼容设备 048d:6006 / FF03:0001");
    let hid = NativeHid::open().map_err(|error| {
        eprintln!("[KeyboardHID] HID 打开失败: {error}");
        format!("HID_OPEN: {error}")
    })?;
    println!("[KeyboardHID] HID 设备已打开");
    let mut keyboard = KcKeyboard::new(hid).map_err(|error| {
        eprintln!("[KeyboardHID] HID 报告校验失败: {error}");
        format!("HID_REPORT: {error}")
    })?;
    let firmware = match keyboard.firmware_version() {
        Ok(version) => {
            println!("[KeyboardHID] 固件版本: {}.{}.{}.{}", version[0], version[1], version[2], version[3]);
            format!("{}.{}.{}.{}", version[0], version[1], version[2], version[3])
        }
        Err(error) => {
            eprintln!("[KeyboardHID] HID 已连接，但固件版本回读失败: {error}");
            "回读不可用".into()
        }
    };

    // The X15 HID path has no dependency on the factory ACPIDriver layout query.
    // Use the SDK's 85-key US key map, matching the pictured X15/Galleria keyboard.
    let mut keymap = vec![None; kc_led_hid::KEY_COUNT];
    for (name, slots) in KeyLayout::Us.keys() {
        for slot in *slots {
            if let Some(target) = keymap.get_mut(*slot) {
                *target = Some((*name).to_string());
            }
        }
    }
    let effect_live = match keyboard.read_effect() {
        Ok(report) => {
            println!("[KeyboardHID] 实时效果回读: {:?}", report);
            Some(report)
        }
        Err(error) => {
            eprintln!("[KeyboardHID] 固件不支持或拒绝 0x88 实时回读: {error}");
            None
        }
    };
    let brightness_live = effect_live.map(|r| r[5]);
    let effect_code = effect_live.map(|r| r[3]);
    let effect = effect_code
        .and_then(|code| Effect::ALL.iter().copied().find(|effect| effect.hid_code() == code));
    Ok(Some(KeyboardHidInfo {
        connected: true,
        firmware,
        layout: "US / 85-key map".into(),
        keymap,
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
    }))
}

#[tauri::command]
pub fn set_keyboard_hid_power(enabled: bool, restore_level: u8) -> Result<(), String> {
    let level = if enabled { restore_level } else { 0 };
    let mut keyboard = open()?;
    println!("[KeyboardHID] 设置灯光电源: enabled={enabled}, brightness={level}");
    keyboard.set_brightness(level).map_err(|error| {
        eprintln!("[KeyboardHID] 设置灯光电源失败: {error}");
        format!("HID_WRITE_POWER: {error}")
    })?;
    println!("[KeyboardHID] 灯光电源写入成功");
    Ok(())
}

#[tauri::command]
pub fn set_keyboard_hid_brightness(level: u8) -> Result<(), String> {
    let mut keyboard = open()?;
    println!("[KeyboardHID] 设置亮度: {level}%");
    keyboard.set_brightness(level).map_err(|error| {
        eprintln!("[KeyboardHID] 设置亮度失败: {error}");
        format!("HID_WRITE_BRIGHTNESS: {error}")
    })?;
    println!("[KeyboardHID] 亮度写入成功");
    Ok(())
}

#[tauri::command]
pub fn set_keyboard_hid_effect(
    effect: String, speed: u8, brightness: u8, direction: String,
    colors: Vec<[u8; 3]>, save: bool,
) -> Result<(), String> {
    let effect = effect.parse::<Effect>().map_err(|error| {
        eprintln!("[KeyboardHID] 无效的内置灯效参数: {error}");
        error.to_string()
    })?;
    let direction = direction.parse::<Direction>().map_err(|error| {
        eprintln!("[KeyboardHID] 无效的灯效方向参数: {error}");
        error.to_string()
    })?;
    if effect == Effect::UserMode {
        return Err("扩展逐键灯效请使用扩展灯效写入操作".into());
    }
    let profile = Profile {
        effect, speed, direction, palette: colors,
        brightness_ac: brightness, brightness_dc: brightness,
        ..Profile::default()
    };
    println!("[KeyboardHID] 写入内置灯效: effect={}, speed={speed}, direction={}, brightness={brightness}, colors={}, save={save}", effect.name(), direction.name(), profile.palette.len());
    let mut keyboard = open()?;
    let supply = Supply::current().map_err(|error| {
        eprintln!("[KeyboardHID] 读取供电状态失败，无法应用内置灯效: {error}");
        error.to_string()
    })?;
    keyboard.apply_profile(&profile, supply, save)
        .map_err(|error| {
            eprintln!("[KeyboardHID] 内置灯效写入失败: {error}");
            format!("HID_WRITE_EFFECT: {error}")
        })?;
    println!("[KeyboardHID] 内置灯效写入成功");
    Ok(())
}

#[tauri::command]
pub fn set_keyboard_hid_extended(colors: Vec<[u8; 3]>, brightness: u8, save: bool) -> Result<(), String> {
    if colors.len() != kc_led_hid::KEY_COUNT {
        eprintln!("[KeyboardHID] 扩展灯效键位数量错误: expected={}, actual={}", kc_led_hid::KEY_COUNT, colors.len());
        return Err(format!("扩展灯效需要 {} 个键位颜色", kc_led_hid::KEY_COUNT));
    }
    let mut profile = Profile::factory_preset(Effect::UserMode);
    profile.brightness_ac = brightness;
    profile.brightness_dc = brightness;
    profile.key_colors = colors;
    println!("[KeyboardHID] 写入扩展灯效: {} 键位, brightness={brightness}, save={save}", profile.key_colors.len());
    let mut keyboard = open()?;
    let supply = Supply::current().map_err(|error| {
        eprintln!("[KeyboardHID] 读取供电状态失败，无法应用扩展灯效: {error}");
        error.to_string()
    })?;
    keyboard.apply_profile(&profile, supply, save)
        .map_err(|error| {
            eprintln!("[KeyboardHID] 扩展灯效写入失败: {error}");
            format!("HID_WRITE_EXTENDED: {error}")
        })?;
    println!("[KeyboardHID] 扩展灯效写入成功");
    Ok(())
}

#[tauri::command]
pub fn get_keyboard_hid_idle_seconds() -> Result<u32, String> {
    let idle_seconds = crate::keyboard_activity::idle_seconds().map_err(|error| {
        eprintln!("[KeyboardHID] 读取键盘空闲时间失败: {error}");
        error
    })?;
    let idle_minute = idle_seconds / 60;
    let previous = LAST_IDLE_MINUTE.swap(idle_minute, Ordering::Relaxed);
    if previous != idle_minute {
        if idle_seconds < 60 && previous != u32::MAX {
            println!("[KeyboardHID] 检测到本机活动，空闲计时已重置");
        } else {
            println!("[KeyboardHID] 键盘空闲时间: {idle_minute} 分钟以上 ({idle_seconds} 秒)");
        }
    }
    Ok(idle_seconds)
}

fn open() -> Result<KcKeyboard<NativeHid>, String> {
    println!("[KeyboardHID] 打开设备以执行写入");
    let hid = NativeHid::open().map_err(|error| {
        eprintln!("[KeyboardHID] HID 打开失败: {error}");
        format!("HID_OPEN: {error}")
    })?;
    KcKeyboard::new(hid).map_err(|error| {
        eprintln!("[KeyboardHID] HID 报告校验失败: {error}");
        format!("HID_REPORT: {error}")
    })
}
