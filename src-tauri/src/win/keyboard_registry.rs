use std::io;

use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};
use winreg::{RegKey, RegValue};

const REG_PATH: &str =
    r"SYSTEM\CurrentControlSet\Services\UniwillService\SOFTWARE\Uniwill\RGBKeyboardSingleColor";

fn open_key() -> io::Result<RegKey> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    hklm.open_subkey_with_flags(
        REG_PATH,
        KEY_READ | KEY_WRITE,
    )
}

/* =========================================================
   数值转换
   ========================================================= */

/// Rust 颜色值：0..=50
/// Registry 颜色值：0..=100 (0x64)
fn color_to_registry(value: u8) -> u8 {
    let value = value.min(50);

    ((value as u16 * 100 + 25) / 50)
        .min(100) as u8
}

/// Registry 颜色值：0..=100
/// Rust 颜色值：0..=50
fn color_from_registry(value: u8) -> u8 {
    let value = value.min(100);

    ((value as u16 * 50 + 50) / 100)
        .min(50) as u8
}

/// Rust 亮度：0..=50
/// Registry 亮度：0..=4
fn brightness_to_registry(value: u8) -> u8 {
    let value = value.min(50);

    ((value as u16 * 4 + 25) / 50)
        .min(4) as u8
}

/// Registry 亮度：0..=4
/// Rust 亮度：0..=50
fn brightness_from_registry(value: u8) -> u8 {
    let value = value.min(4);

    ((value as u16 * 50 + 2) / 4)
        .min(50) as u8
}

/// bool -> Registry DWORD
///
/// false = 0
/// true  = 1
fn bool_to_registry(value: bool) -> u32 {
    if value {
        1
    } else {
        0
    }
}

/// Registry DWORD -> bool
///
/// 0 = false
/// 非 0 = true
fn bool_from_registry(value: u32) -> bool {
    value != 0
}

/* =========================================================
   Power
   ========================================================= */

/// 设置 RGBKeyboardSingleColor 总电源
///
/// Rust:
/// false = 0
/// true  = 1
///
/// Registry:
/// Power = 0 / 1
pub fn set_power(
    power: bool,
) -> io::Result<()> {
    let key = open_key()?;

    let value = bool_to_registry(power);

    key.set_value(
        "Power",
        &value,
    )?;

    Ok(())
}

/// 读取 RGBKeyboardSingleColor 总电源
pub fn get_power() -> io::Result<bool> {
    let key = open_key()?;

    let value: u32 = key.get_value("Power")?;

    Ok(bool_from_registry(value))
}

/* =========================================================
   Effect
   ========================================================= */

/// 设置 Effect
///
/// Rust:
/// false = 0
/// true  = 1
///
/// Registry:
/// Effect = 0 / 1
pub fn set_effect(
    effect: bool,
) -> io::Result<()> {
    let key = open_key()?;

    let value = bool_to_registry(effect);

    key.set_value(
        "Effect",
        &value,
    )?;

    Ok(())
}

/// 读取 Effect
pub fn get_effect() -> io::Result<bool> {
    let key = open_key()?;

    let value: u32 = key.get_value("Effect")?;

    Ok(bool_from_registry(value))
}

/* =========================================================
   Brightness
   ========================================================= */

/// 设置 AC/DC 亮度
///
/// Rust:
/// 0..=50
///
/// Registry:
/// 0..=4
pub fn set_brightness(
    brightness: u8,
    ac: bool,
) -> io::Result<()> {
    let key = open_key()?;

    let value_name = if ac {
        "Light_AC"
    } else {
        "Light_DC"
    };

    let value = brightness as u32;

    key.set_value(
        value_name,
        &value,
    )?;

    Ok(())
}

/// 读取 AC/DC 亮度
///
/// Registry:
/// 0..=4
///
/// Rust:
/// 0..=50
pub fn get_brightness(
    ac: bool,
) -> io::Result<u8> {
    let key = open_key()?;

    let value_name = if ac {
        "Light_AC"
    } else {
        "Light_DC"
    };

    let value: u32 =
        key.get_value(value_name)?;

    Ok(
        brightness_from_registry(
            value.min(4) as u8
        )
    )
}

/* =========================================================
   Color
   ========================================================= */

/// 设置 AC/DC RGB
///
/// Rust:
/// 0..=50
///
/// Registry:
/// 0..=100
///
/// 例如：
///
/// Rust:
///     (50, 25, 0)
///
/// Registry:
///     (100, 50, 0)
pub fn set_color(
    red: u8,
    green: u8,
    blue: u8,
    ac: bool,
) -> io::Result<()> {
    let key = open_key()?;

    let value_name = if ac {
        "Color_AC"
    } else {
        "Color_DC"
    };

    let color = vec![
        color_to_registry(red),
        color_to_registry(green),
        color_to_registry(blue),
    ];

    key.set_raw_value(
        value_name,
        &RegValue {
            vtype: winreg::enums::RegType::REG_BINARY,
            bytes: color,
        },
    )?;

    Ok(())
}

/// 读取 AC/DC RGB
///
/// Registry:
/// 0..=100
///
/// Rust:
/// 0..=50
pub fn get_color(
    ac: bool,
) -> io::Result<(u8, u8, u8)> {
    let key = open_key()?;

    let value_name = if ac {
        "Color_AC"
    } else {
        "Color_DC"
    };

    let value =
        key.get_raw_value(value_name)?;

    if value.vtype
        != winreg::enums::RegType::REG_BINARY
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{value_name} 不是 REG_BINARY"
            ),
        ));
    }

    if value.bytes.len() < 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{value_name} 数据长度不足"
            ),
        ));
    }

    Ok((
        color_from_registry(value.bytes[0]),
        color_from_registry(value.bytes[1]),
        color_from_registry(value.bytes[2]),
    ))
}

/* =========================================================
   单独修改 RGB 通道
   ========================================================= */

/// 设置红色
pub fn set_red(
    red: u8,
    ac: bool,
) -> io::Result<()> {
    let (_, green, blue) =
        get_color(ac)?;

    set_color(
        red,
        green,
        blue,
        ac,
    )
}

/// 设置绿色
pub fn set_green(
    green: u8,
    ac: bool,
) -> io::Result<()> {
    let (red, _, blue) =
        get_color(ac)?;

    set_color(
        red,
        green,
        blue,
        ac,
    )
}

/// 设置蓝色
pub fn set_blue(
    blue: u8,
    ac: bool,
) -> io::Result<()> {
    let (red, green, _) =
        get_color(ac)?;

    set_color(
        red,
        green,
        blue,
        ac,
    )
}

/* =========================================================
   Timer
   ========================================================= */

/// 设置 AC Timer
pub fn set_timer_ac(
    timer: u32,
) -> io::Result<()> {
    let key = open_key()?;

    key.set_value(
        "Timer_AC",
        &timer,
    )?;

    Ok(())
}

/// 设置 DC Timer
pub fn set_timer_dc(
    timer: u32,
) -> io::Result<()> {
    let key = open_key()?;

    key.set_value(
        "Timer_DC",
        &timer,
    )?;

    Ok(())
}