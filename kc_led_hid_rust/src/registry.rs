use crate::{Direction, Effect, HidError, KEY_COUNT, KeySetting, Profile, Result, Rgb};
use std::io;
use winreg::{RegKey, RegValue, enums::*, transaction::Transaction};

const PATH: &str = r"SYSTEM\CurrentControlSet\Services\UniwillService\SOFTWARE\Uniwill\RGBKeyboard";
const KEY_SETTINGS_PATH: &str =
    r"SYSTEM\CurrentControlSet\Services\UniwillService\SOFTWARE\Uniwill\Keyboard";

/// Read an OEM startup setting for the Windows key or touchpad shortcut.
pub fn load_key_setting(setting: KeySetting) -> Result<bool> {
    let name = setting.registry_value()?;
    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(KEY_SETTINGS_PATH, KEY_READ | KEY_WOW64_64KEY)?;
    Ok(dword(&key, name, 0)? > 0)
}

/// Write one OEM startup value for the Windows key or touchpad shortcut.
pub fn save_key_setting(setting: KeySetting, enabled: bool) -> Result<()> {
    let name = setting.registry_value()?;
    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(KEY_SETTINGS_PATH, KEY_SET_VALUE | KEY_WOW64_64KEY)?;
    Ok(key.set_value(name, &u32::from(enabled))?)
}

pub fn load_console_profile() -> Result<Profile> {
    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(PATH, KEY_READ | KEY_WOW64_64KEY)?;
    read_profile(&key)
}

fn read_profile(key: &RegKey) -> Result<Profile> {
    let defaults = Profile::default();
    let count = dword(key, "ColorLength", 1)?;
    let mut palette = Vec::new();
    for i in 0..if count <= 7 { count } else { 0 } {
        palette.push(rgb_data(key, &format!("Color_{i}"), 1)?.map_or([255; 3], |v| v[0]));
    }
    let profile = Profile {
        power: dword(key, "Power", 1)? == 1,
        effect: Effect::from_console(dword(key, "Effect", 0)?)?,
        brightness_ac: dword(key, "Light_AC", defaults.brightness_ac.into())?.try_into()?,
        brightness_dc: dword(key, "Light_DC", defaults.brightness_dc.into())?.try_into()?,
        idle_ac: dword(key, "Timer_AC", 0)?,
        idle_dc: dword(key, "Timer_DC", 0)?,
        speed: dword(key, "Speed", 0)?.try_into()?,
        direction: Direction::from_console(dword(key, "Direction", 0)?)?,
        palette,
        key_colors: rgb_data(key, "KeyColor", KEY_COUNT)?.unwrap_or_default(),
    };
    profile.validate()?;
    Ok(profile)
}

/// Update the registry used at console/service startup.
pub fn save_console_profile(profile: &Profile) -> Result<()> {
    profile.validate()?;
    let transaction = Transaction::new()?;
    let key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_transacted_with_flags(
        PATH,
        &transaction,
        KEY_SET_VALUE | KEY_WOW64_64KEY,
    )?;
    write_profile(&key, profile)?;
    drop(key);
    Ok(transaction.commit()?)
}

fn write_profile(key: &RegKey, profile: &Profile) -> Result<()> {
    for (i, color) in profile.palette.iter().enumerate() {
        write_rgb(key, &format!("Color_{i}"), &[*color])?;
    }
    if profile.key_colors.is_empty() {
        match key.delete_value("KeyColor") {
            Ok(()) => (),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    } else {
        write_rgb(key, "KeyColor", &profile.key_colors)?;
    }
    for (name, value) in [
        ("Timer_AC", profile.idle_ac),
        ("Timer_DC", profile.idle_dc),
        ("Light_AC", profile.brightness_ac.into()),
        ("Light_DC", profile.brightness_dc.into()),
        ("Speed", profile.speed.into()),
        ("Direction", profile.direction as u32),
        ("ColorLength", profile.palette.len() as u32),
        ("Effect", profile.effect as u32),
        ("Power", profile.power.into()),
    ] {
        key.set_value(name, &value)?;
    }
    Ok(())
}

fn dword(key: &RegKey, name: &str, default: u32) -> io::Result<u32> {
    match key.get_value(name) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(default),
        value => value,
    }
}

fn rgb_data(key: &RegKey, name: &str, count: usize) -> Result<Option<Vec<Rgb>>> {
    let value = match key.get_raw_value(name) {
        Ok(value) => value,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    if value.vtype != REG_BINARY || value.bytes.len() != count * 3 {
        return Err(HidError(format!(
            "原厂配置 {name} 应为 {} 字节 REG_BINARY",
            count * 3
        ))
        .into());
    }
    Ok(Some(value.bytes.as_chunks::<3>().0.to_vec()))
}

fn write_rgb(key: &RegKey, name: &str, colors: &[Rgb]) -> io::Result<()> {
    key.set_raw_value(
        name,
        &RegValue {
            vtype: REG_BINARY,
            bytes: colors.iter().flatten().copied().collect(),
        },
    )
}

const AURORA_PATH: &str = r"SOFTWARE\OEM\Aurora";
const AURORA_UNINSTALL: &str = r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\{9444602B-C5D8-4EF5-9D5B-E76D06B53C71}_is1";

/// State of the Project Aurora integration switch used by NSS.
pub fn project_aurora_enabled() -> Result<bool> {
    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(AURORA_PATH, KEY_READ | KEY_WOW64_64KEY);
    match key {
        Ok(key) => Ok(dword(&key, "AuroraSwitch", 0)? == 1),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

/// Write the switch read by Project Aurora and the factory console.
pub fn set_project_aurora(enabled: bool) -> Result<()> {
    let (key, _) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .create_subkey_with_flags(AURORA_PATH, KEY_SET_VALUE | KEY_WOW64_64KEY)?;
    key.set_value("AuroraSwitch", &u32::from(enabled))?;
    Ok(())
}

/// Check the installer's registry record used by the factory integration.
pub fn project_aurora_installed() -> Result<bool> {
    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(AURORA_UNINSTALL, KEY_READ | KEY_WOW64_64KEY);
    match key {
        Ok(key) => Ok(!key
            .get_value::<String, _>("InstallLocation")
            .unwrap_or_default()
            .is_empty()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a Windows session allowed to create a temporary HKCU registry key"]
    fn console_registry_round_trip_rolls_back_test_key() -> Result<()> {
        let transaction = Transaction::new()?;
        let root = RegKey::predef(HKEY_CURRENT_USER);
        let path = format!(r"Software\kc-led-hid-test-{}", std::process::id());
        let (key, _) = root.create_subkey_transacted(&path, &transaction)?;
        let mut profile = Profile {
            effect: Effect::UserMode,
            direction: Direction::PressAnyKey,
            brightness_dc: 35,
            idle_ac: 90,
            palette: vec![[13, 17, 23], [31, 37, 41]],
            key_colors: vec![[1, 2, 3]; KEY_COUNT],
            ..Profile::default()
        };
        write_profile(&key, &profile)?;
        assert_eq!(read_profile(&key)?, profile);
        assert_eq!(key.get_raw_value("KeyColor")?.bytes.len(), 378);
        profile.palette.clear();
        profile.key_colors.clear();
        write_profile(&key, &profile)?;
        assert_eq!(read_profile(&key)?, profile);
        drop(key);
        transaction.rollback()?;
        Ok(())
    }
}
