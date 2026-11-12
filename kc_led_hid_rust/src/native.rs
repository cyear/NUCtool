use super::{FEATURE_LEN, HidError, OUTPUT_LEN, PID, Result, Transport, USAGE, USAGE_PAGE, VID};
use windows_sys::Win32::{
    Devices::{
        DeviceAndDriverInstallation::{
            DIGCF_DEVICEINTERFACE, DIGCF_PRESENT, SP_DEVICE_INTERFACE_DATA,
            SP_DEVICE_INTERFACE_DETAIL_DATA_W, SetupDiDestroyDeviceInfoList,
            SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW, SetupDiGetDeviceInterfaceDetailW,
        },
        HumanInterfaceDevice::{
            HIDD_ATTRIBUTES, HIDP_CAPS, HidD_FreePreparsedData, HidD_GetAttributes,
            HidD_GetFeature, HidD_GetHidGuid, HidD_GetPreparsedData, HidD_SetFeature, HidP_GetCaps,
        },
    },
    Foundation::{CloseHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    },
};
pub struct NativeHid {
    handle: HANDLE,
    feature_len: usize,
}
impl NativeHid {
    pub fn open() -> Result<Self> {
        unsafe {
            let mut guid = std::mem::zeroed();
            HidD_GetHidGuid(&mut guid);
            let set = SetupDiGetClassDevsW(
                &guid,
                std::ptr::null(),
                std::ptr::null_mut(),
                DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
            );
            if set == -1isize {
                return Err(last_error("SetupDiGetClassDevsW"));
            }
            let mut found = None;
            let mut index = 0;
            loop {
                let mut interface = SP_DEVICE_INTERFACE_DATA {
                    cbSize: std::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32,
                    ..Default::default()
                };
                if SetupDiEnumDeviceInterfaces(set, std::ptr::null(), &guid, index, &mut interface)
                    == 0
                {
                    break;
                }
                index += 1;
                let mut needed = 0;
                let _ = SetupDiGetDeviceInterfaceDetailW(
                    set,
                    &interface,
                    std::ptr::null_mut(),
                    0,
                    &mut needed,
                    std::ptr::null_mut(),
                );
                if needed < 6 {
                    continue;
                }
                let words = (needed as usize).div_ceil(4);
                let mut detail_buf = vec![0u32; words];
                let detail = detail_buf
                    .as_mut_ptr()
                    .cast::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>();
                (*detail).cbSize = if cfg!(target_pointer_width = "64") {
                    8
                } else {
                    6
                };
                if SetupDiGetDeviceInterfaceDetailW(
                    set,
                    &interface,
                    detail,
                    needed,
                    &mut needed,
                    std::ptr::null_mut(),
                ) == 0
                {
                    continue;
                }
                // DevicePath begins after the DWORD cbSize, with a NUL in the supplied buffer.
                let path = detail_buf.as_ptr().add(1).cast::<u16>();
                if !std::slice::from_raw_parts(path, (words - 1) * 2).contains(&0) {
                    continue;
                }
                let handle = CreateFileW(
                    path,
                    0xc0000000,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    std::ptr::null(),
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    std::ptr::null_mut(),
                );
                if handle == INVALID_HANDLE_VALUE {
                    continue;
                }
                let mut attrs = HIDD_ATTRIBUTES {
                    Size: std::mem::size_of::<HIDD_ATTRIBUTES>() as u32,
                    ..Default::default()
                };
                let mut preparsed = 0isize;
                let matches_id = HidD_GetAttributes(handle, &mut attrs)
                    && attrs.VendorID == VID
                    && attrs.ProductID == PID;
                if !matches_id || !HidD_GetPreparsedData(handle, &mut preparsed) {
                    CloseHandle(handle);
                    continue;
                }
                let mut caps = HIDP_CAPS::default();
                let status = HidP_GetCaps(preparsed, &mut caps);
                HidD_FreePreparsedData(preparsed);
                if status == 0x0011_0000 && caps.Usage == USAGE && caps.UsagePage == USAGE_PAGE {
                    found = Some((
                        handle,
                        caps.FeatureReportByteLength as usize,
                        caps.OutputReportByteLength as usize,
                    ));
                    break;
                }
                CloseHandle(handle);
            }
            SetupDiDestroyDeviceInfoList(set);
            let (handle, feature_len, output_len) = found.ok_or_else(|| HidError(format!(
                "未找到 KC HID {VID:04x}:{PID:04x} UsagePage={USAGE_PAGE:04x} Usage={USAGE}; 当前机型可能不匹配")))?;
            if feature_len != FEATURE_LEN || output_len != OUTPUT_LEN {
                CloseHandle(handle);
                return Err(HidError(format!(
                    "报告长度与已知协议不符：Feature={feature_len}, Output={output_len}; 拒绝写入"
                ))
                .into());
            }
            Ok(Self {
                handle,
                feature_len,
            })
        }
    }
}

pub(crate) fn last_error(context: &str) -> Box<dyn std::error::Error> {
    let code = unsafe { GetLastError() };
    HidError(format!("{context} 失败，Win32 错误 {code}")).into()
}

impl Transport for NativeHid {
    fn prepare_key_trigger(&self) -> Result<()> {
        Ec::open()?.read(1857)?;
        Ok(())
    }

    fn enable_key_trigger(&self) -> Result<()> {
        Ec::open()?.enable_key_trigger()
    }
    fn feature_len(&self) -> usize {
        self.feature_len
    }
    fn set_feature(&self, report: &[u8]) -> Result<()> {
        if report.len() != self.feature_len {
            return Err(HidError("Feature Report 长度错误".into()).into());
        }
        let ok =
            unsafe { HidD_SetFeature(self.handle, report.as_ptr().cast(), report.len() as u32) };
        if !ok {
            return Err(last_error("HidD_SetFeature"));
        }
        Ok(())
    }
    fn get_feature(&self, report: &mut [u8]) -> Result<()> {
        if report.len() != self.feature_len {
            return Err(HidError("Feature Report 长度错误".into()).into());
        }
        let ok = unsafe {
            HidD_GetFeature(self.handle, report.as_mut_ptr().cast(), report.len() as u32)
        };
        if !ok {
            return Err(last_error("HidD_GetFeature"));
        }
        Ok(())
    }
    fn write_output(&self, report: &[u8]) -> Result<()> {
        if report.len() != OUTPUT_LEN {
            return Err(HidError("Output Report 应为 65 字节".into()).into());
        }
        // Matches FileStream.Write on the service's HID handle (IRP_MJ_WRITE).
        let mut written = 0u32;
        let ok = unsafe {
            windows_sys::Win32::Storage::FileSystem::WriteFile(
                self.handle,
                report.as_ptr(),
                report.len() as u32,
                &mut written,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(last_error("WriteFile(Output Report)"));
        }
        if written as usize != report.len() {
            return Err(HidError(format!(
                "Output Report 仅写入 {written}/{} 字节",
                report.len()
            ))
            .into());
        }
        Ok(())
    }
}
impl Drop for NativeHid {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Supply {
    Ac,
    Dc,
}

impl Supply {
    pub fn current() -> Result<Self> {
        use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
        let mut status = SYSTEM_POWER_STATUS::default();
        if unsafe { GetSystemPowerStatus(&mut status) } == 0 {
            return Err(last_error("GetSystemPowerStatus"));
        }
        Ok(if status.ACLineStatus == 1 {
            Self::Ac
        } else {
            Self::Dc
        })
    }
}

impl super::KcKeyboard<NativeHid> {
    pub fn open() -> Result<Self> {
        Self::new(NativeHid::open()?)
    }

    /// Keyboard layout code reported by the factory EC path.
    pub fn keyboard_layout(&self) -> Result<u8> {
        Ok(match Ec::open()?.read(1852)? {
            17 => 86,
            73 => 87,
            65 => 88,
            _ => 85,
        })
    }
}

pub(crate) struct Ec(HANDLE);

impl Ec {
    pub(crate) fn open() -> Result<Self> {
        let path: Vec<_> = r"\\.\ACPIDriver".encode_utf16().chain([0]).collect();
        let handle = unsafe {
            CreateFileW(
                path.as_ptr(),
                0xc0000000,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                std::ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(last_error(r"打开 \\.\ACPIDriver"));
        }
        Ok(Self(handle))
    }

    fn call(&self, code: u32, input: &[u32]) -> Result<(u32, u32)> {
        use windows_sys::Win32::System::IO::DeviceIoControl;
        let mut value = 0u32;
        let mut returned = 0;
        let ok = unsafe {
            DeviceIoControl(
                self.0,
                code,
                input.as_ptr().cast(),
                std::mem::size_of_val(input) as u32,
                (&mut value as *mut u32).cast(),
                4,
                &mut returned,
                std::ptr::null_mut(),
            )
        };

        if ok == 0 {
            return Err(last_error("ACPIDriver DeviceIoControl"));
        }
        std::thread::sleep(std::time::Duration::from_millis(6));
        Ok((value, returned))
    }

    pub(crate) fn read(&self, addr: u32) -> Result<u8> {
        let (value, returned) = self.call(0x9c40_a488, &[addr])?;
        if returned != 4 {
            return Err(HidError(format!("EC 读取应返回 4 字节，实际 {returned}")).into());
        }
        Ok(value as u8)
    }

    pub(crate) fn write(&self, addr: u32, value: u8) -> Result<()> {
        self.call(0x9c40_a48c, &[addr, u32::from(value)])?;
        Ok(())
    }

    fn enable_key_trigger(&self) -> Result<()> {
        let current = self.read(1857)?;
        if current & 8 == 0 {
            self.write(1857, current | 8)?;
        }
        Ok(())
    }
}

impl Drop for Ec {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

/// Run the session idle and AC/DC brightness policy until the callback requests a stop.
pub fn run_idle_policy<T: Transport>(
    keyboard: &mut super::KcKeyboard<T>,
    profile: &super::Profile,
    stop: impl FnMut() -> bool,
) -> Result<()> {
    watch_lighting(keyboard, &mut profile.clone(), None, stop)
}

/// Run a profile in the current session, including Music when selected.
/// When hotkey events are supplied, Fn changes update the caller's profile.
pub fn watch_profile<T: Transport>(
    keyboard: &mut super::KcKeyboard<T>,
    profile: &mut super::Profile,
    events: Option<&super::FnBrightnessEvents>,
    stop: impl FnMut() -> bool,
) -> Result<()> {
    if profile.effect == super::Effect::Music {
        super::music::watch_music(keyboard, profile, events, stop)
    } else {
        watch_lighting(keyboard, profile, events, stop)
    }
}

fn watch_lighting<T: Transport>(
    keyboard: &mut super::KcKeyboard<T>,
    profile: &mut super::Profile,
    events: Option<&super::FnBrightnessEvents>,
    mut stop: impl FnMut() -> bool,
) -> Result<()> {
    use std::{thread, time::Duration};
    keyboard.apply_profile(profile, Supply::current()?, false)?;
    if !profile.power {
        return Ok(());
    }
    let mut last = None;
    let mut brightness = RuntimeBrightness::new(events);
    while !stop() {
        let level = brightness.update(profile)?;
        if last != Some(level) {
            keyboard.set_brightness(level)?;
            last = Some(level);
        }
        thread::sleep(Duration::from_millis(250));
    }
    if last == Some(0) {
        keyboard.set_brightness(profile.brightness(Supply::current()?))?;
    }
    Ok(())
}

pub(crate) struct RuntimeBrightness<'a> {
    events: Option<&'a super::FnBrightnessEvents>,
    last_press: Option<std::time::Instant>,
}

impl<'a> RuntimeBrightness<'a> {
    pub(crate) fn new(events: Option<&'a super::FnBrightnessEvents>) -> Self {
        Self {
            events,
            last_press: None,
        }
    }

    pub(crate) fn update(&mut self, profile: &mut super::Profile) -> Result<u8> {
        let supply = Supply::current()?;
        if let Some(events) = self.events {
            let presses = events.poll()?;
            if presses != 0 {
                self.last_press = Some(std::time::Instant::now());
                for _ in 0..presses {
                    profile.cycle_brightness(supply);
                }
            }
        }
        idle_brightness(profile, supply, self.last_press)
    }
}

fn idle_brightness(
    profile: &super::Profile,
    supply: Supply,
    last_press: Option<std::time::Instant>,
) -> Result<u8> {
    use windows_sys::Win32::{
        System::SystemInformation::GetTickCount,
        UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
    };
    let timeout = profile.idle_seconds(supply);
    if timeout == 0 {
        return Ok(profile.brightness(supply));
    }
    let mut input = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    if unsafe { GetLastInputInfo(&mut input) } == 0 {
        return Err(HidError("GetLastInputInfo 失败".into()).into());
    }
    let elapsed = unsafe { GetTickCount() }.wrapping_sub(input.dwTime) / 1000;
    let hotkey_idle =
        last_press.is_none_or(|press| press.elapsed().as_secs() >= u64::from(timeout));
    Ok(if elapsed >= timeout && hotkey_idle {
        0
    } else {
        profile.brightness(supply)
    })
}
