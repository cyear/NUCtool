use crate::{HidError, KcKeyboard, NativeHid, Result, native::Ec};
use std::{str::FromStr, thread, time::Duration};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, SendInput, VK_CAPITAL, VK_NUMLOCK, VK_SCROLL,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeySetting {
    WindowsKey,
    FnLock,
    NumLock,
    ScrollLock,
    CapsLock,
    /// Disable the touchpad-toggle shortcut. The touchpad's current state is independent.
    DisableTouchpadToggle,
}

impl KeySetting {
    pub const ALL: [Self; 6] = [
        Self::WindowsKey,
        Self::FnLock,
        Self::NumLock,
        Self::ScrollLock,
        Self::CapsLock,
        Self::DisableTouchpadToggle,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::WindowsKey => "windows-key",
            Self::FnLock => "fn-lock",
            Self::NumLock => "num-lock",
            Self::ScrollLock => "scroll-lock",
            Self::CapsLock => "caps-lock",
            Self::DisableTouchpadToggle => "disable-touchpad-toggle",
        }
    }

    pub(crate) fn registry_value(self) -> Result<&'static str> {
        match self {
            Self::WindowsKey => Ok("WinKey"),
            Self::DisableTouchpadToggle => Ok("DisableTouchPadToggle"),
            _ => Err(HidError(format!("{} 没有独立的原厂注册表值", self.name())).into()),
        }
    }
}

impl FromStr for KeySetting {
    type Err = Box<dyn std::error::Error>;

    fn from_str(name: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|key| key.name().eq_ignore_ascii_case(name))
            .ok_or_else(|| HidError(format!("未知的键盘设置 {name}")).into())
    }
}

impl KcKeyboard<NativeHid> {
    /// Read the current EC or Windows-session value on the matched KC machine.
    pub fn key_setting(&self, key: KeySetting) -> Result<bool> {
        if let Some(vk) = lock_key_virtual_key(key) {
            return Ok(lock_key_state(vk));
        }
        let ec = Ec::open()?;
        read_ec_setting(key, |addr| ec.read(addr))
    }

    /// Apply a setting to the matched KC machine. Use save_key_setting for OEM startup settings.
    pub fn set_key_setting(&mut self, key: KeySetting, enabled: bool) -> Result<()> {
        if let Some(vk) = lock_key_virtual_key(key) {
            return set_lock_key(vk, enabled);
        }
        let ec = Ec::open()?;
        apply_ec_setting(
            key,
            enabled,
            |addr| ec.read(addr),
            |addr, value| ec.write(addr, value),
            || thread::sleep(Duration::from_millis(100)),
        )
    }
}

const WIN_KEY_COMMAND: u32 = 1895;
const WIN_KEY_STATUS: u32 = 1896;
const FN_LOCK: u32 = 1870;
const TOUCHPAD_TOGGLE: u32 = 1958;

fn read_ec_setting(key: KeySetting, mut read: impl FnMut(u32) -> Result<u8>) -> Result<bool> {
    Ok(match key {
        KeySetting::WindowsKey => read(WIN_KEY_STATUS)? & 1 == 0,
        KeySetting::FnLock => read(FN_LOCK)? & 0x10 != 0,
        KeySetting::DisableTouchpadToggle => read(TOUCHPAD_TOGGLE)? & 0x40 != 0,
        KeySetting::NumLock | KeySetting::ScrollLock | KeySetting::CapsLock => {
            unreachable!("Lock keys are handled by Windows")
        }
    })
}

fn apply_ec_setting(
    key: KeySetting,
    enabled: bool,
    mut read: impl FnMut(u32) -> Result<u8>,
    mut write: impl FnMut(u32, u8) -> Result<()>,
    mut wait: impl FnMut(),
) -> Result<()> {
    let (addr, mask) = match key {
        KeySetting::WindowsKey => {
            for _ in 0..11 {
                if read_ec_setting(key, &mut read)? == enabled {
                    return Ok(());
                }
                write(WIN_KEY_COMMAND, read(WIN_KEY_COMMAND)? | 1)?;
                wait();
            }
            if read_ec_setting(key, &mut read)? == enabled {
                return Ok(());
            }
            return Err(HidError("EC 未确认 Windows 键状态".into()).into());
        }
        KeySetting::FnLock => (FN_LOCK, 0x10),
        KeySetting::DisableTouchpadToggle => (TOUCHPAD_TOGGLE, 0x40),
        KeySetting::NumLock | KeySetting::ScrollLock | KeySetting::CapsLock => {
            unreachable!("Lock keys are handled by Windows")
        }
    };
    let current = read(addr)?;
    let next = (current & !mask) | if enabled { mask } else { 0 };
    if current != next {
        write(addr, next)?;
        if read_ec_setting(key, &mut read)? != enabled {
            return Err(HidError(format!("EC 未确认 {} 状态", key.name())).into());
        }
    }
    Ok(())
}

fn lock_key_virtual_key(key: KeySetting) -> Option<u16> {
    match key {
        KeySetting::NumLock => Some(VK_NUMLOCK),
        KeySetting::ScrollLock => Some(VK_SCROLL),
        KeySetting::CapsLock => Some(VK_CAPITAL),
        _ => None,
    }
}

fn num_lock_event_flags() -> [u32; 2] {
    [
        KEYEVENTF_EXTENDEDKEY,
        KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP,
    ]
}

fn lock_key_state(virtual_key: u16) -> bool {
    unsafe { GetKeyState(i32::from(virtual_key)) & 1 != 0 }
}

fn set_lock_key(virtual_key: u16, enabled: bool) -> Result<()> {
    if lock_key_state(virtual_key) == enabled {
        return Ok(());
    }
    let events = num_lock_event_flags().map(|dw_flags| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: virtual_key,
                dwFlags: dw_flags,
                ..Default::default()
            },
        },
    });
    let size = std::mem::size_of::<INPUT>() as i32;
    let sent = unsafe { SendInput(2, events.as_ptr(), size) };
    if sent == 2 {
        return Ok(());
    }
    let error = std::io::Error::last_os_error();
    if sent == 1 {
        unsafe { SendInput(1, events[1..].as_ptr(), size) };
    }
    Err(HidError(format!(
        "Num Lock SendInput 只发送了 {sent}/2 个事件，Win32 {error}"
    ))
    .into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn fn_lock_and_touchpad_toggle_preserve_other_ec_bits() {
        for (key, addr, mask) in [
            (KeySetting::FnLock, FN_LOCK, 0x10),
            (KeySetting::DisableTouchpadToggle, TOUCHPAD_TOGGLE, 0x40),
        ] {
            let before = 0xaf & !mask;
            let register = Cell::new(before);
            let read = |actual| {
                assert_eq!(actual, addr);
                Ok(register.get())
            };
            let write = |actual, value| {
                assert_eq!(actual, addr);
                register.set(value);
                Ok(())
            };
            apply_ec_setting(key, true, read, write, || {}).unwrap();
            assert_eq!(register.get(), before | mask);
            apply_ec_setting(key, false, read, write, || {}).unwrap();
            assert_eq!(register.get(), before);
        }
    }

    #[test]
    fn windows_key_uses_command_register_and_checks_inverted_status() {
        let writes = Cell::new(0);
        let read = |addr| match addr {
            WIN_KEY_STATUS => Ok(u8::from(writes.get() < 2)),
            WIN_KEY_COMMAND => Ok(0xa0),
            _ => panic!("unexpected EC read {addr}"),
        };
        let write = |addr, value| {
            assert_eq!((addr, value), (WIN_KEY_COMMAND, 0xa1));
            writes.set(writes.get() + 1);
            Ok(())
        };
        apply_ec_setting(KeySetting::WindowsKey, true, read, write, || {}).unwrap();
        apply_ec_setting(KeySetting::WindowsKey, true, read, write, || {}).unwrap();
        assert_eq!(writes.get(), 2);
    }

    #[test]
    fn num_lock_key_events_include_factory_extended_key_flags() {
        assert_eq!(
            num_lock_event_flags(),
            [
                KEYEVENTF_EXTENDEDKEY,
                KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP
            ]
        );
        assert_eq!(lock_key_virtual_key(KeySetting::NumLock), Some(144));
        assert_eq!(lock_key_virtual_key(KeySetting::ScrollLock), Some(145));
        assert_eq!(lock_key_virtual_key(KeySetting::CapsLock), Some(20));
    }

    #[test]
    fn windows_key_reports_firmware_that_never_confirms_the_toggle() {
        let writes = Cell::new(0);
        let result = apply_ec_setting(
            KeySetting::WindowsKey,
            true,
            |_| Ok(1),
            |_, _| {
                writes.set(writes.get() + 1);
                Ok(())
            },
            || {},
        );
        assert!(result.is_err());
        assert_eq!(writes.get(), 11);
    }
}
