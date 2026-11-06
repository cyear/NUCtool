#![cfg_attr(not(windows), allow(dead_code))]
#![allow(unsafe_op_in_unsafe_fn)]

#[cfg(windows)]
pub(crate) mod app {
    use std::{
        env,
        ffi::{c_char, c_void},
        fmt::Write as _,
        io::{stdin, stdout},
        process::{Command, Stdio},
        sync::{
            atomic::{AtomicBool, Ordering},
            mpsc, Mutex, OnceLock,
        },
    };
    use serde::{de::DeserializeOwned, Deserialize, Serialize};

    type Handle = *mut c_void;
    type ApiInitFn = unsafe extern "system" fn() -> Handle;
    type SetApoTypeFn = unsafe extern "system" fn(Handle, *const c_char) -> i32;
    type GetEndpointFn = unsafe extern "system" fn(Handle, *const u16, i32) -> u64;
    type GetEndpointExFn = unsafe extern "system" fn(Handle, *const u16, i32, *const Guid) -> u64;
    type FreeApiFn = unsafe extern "system" fn(Handle) -> i32;
    type GetFn = unsafe extern "system" fn(Handle, i32) -> f32;
    type SetFn = unsafe extern "system" fn(Handle, i32, f32) -> i32;
    type GetLastErrorFn = unsafe extern "system" fn(Handle) -> i32;
    type GetLastWindowsErrorFn = unsafe extern "system" fn(Handle) -> u32;
    type ContentFn = unsafe extern "system" fn(Handle, *const c_char) -> i32;
    type GetContentFn = unsafe extern "system" fn(Handle, *mut c_char, i32) -> i32;
    type GetProfileFn = unsafe extern "system" fn(Handle, *mut MsvcWString) -> i32;
    type GetProfilesFn =
        unsafe extern "system" fn(Handle, *mut i32, *mut *mut *mut u16, *mut u64) -> i32;
    type SetProfileFn = unsafe extern "system" fn(Handle, *const u16) -> i32;
    type GetProfileAliasFn = unsafe extern "system" fn(Handle, *const u16, *mut u16) -> i32;
    type FreeProfilesFn = unsafe extern "system" fn(Handle, i32, *mut *mut u16, u64) -> i32;
    type DeleteProfileFn = unsafe extern "system" fn(Handle, *const u16) -> i32;
    type CopyProfileFn = unsafe extern "system" fn(Handle, *const u16, *const u16) -> i32;
    type SetProfileAliasFn = unsafe extern "system" fn(Handle, *const u16, *const u16) -> i32;
    type ResetProfilesFn = unsafe extern "system" fn(Handle, i32) -> i32;
    type GetFxPropertyExFn =
        unsafe extern "system" fn(Handle, *const u16, *const Guid, i32, *mut Variant) -> i32;
    type SetFxPropertyExFn =
        unsafe extern "system" fn(Handle, *const u16, *const Guid, i32, *mut Variant) -> i32;
    type GetFxPropertyEx2Fn = unsafe extern "system" fn(
        Handle,
        *const u16,
        *const Guid,
        i32,
        *const Guid,
        i32,
        *mut Variant,
    ) -> i32;
    type SetFxPropertyEx2Fn = unsafe extern "system" fn(
        Handle,
        *const u16,
        *const Guid,
        i32,
        *const Guid,
        i32,
        *mut Variant,
    ) -> i32;

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Variant {
        vt: u16,
        reserved1: u16,
        reserved2: u16,
        reserved3: u16,
        data: u64,
        record_info: u64,
    }

    #[repr(C)]
    struct RtlOsVersionInfoW {
        size: u32,
        major: u32,
        minor: u32,
        build: u32,
        platform_id: u32,
        service_pack: [u16; 128],
    }

    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn RtlGetVersion(version: *mut RtlOsVersionInfoW) -> i32;
    }

    #[link(name = "OleAut32")]
    unsafe extern "system" {
        fn VariantClear(value: *mut Variant) -> i32;
    }

    #[repr(C)]
    struct MsvcWString {
        storage: [u8; 16],
        len: usize,
        cap: usize,
    }
    impl MsvcWString {
        fn empty() -> Self {
            Self {
                storage: [0; 16],
                len: 0,
                cap: 7,
            }
        }
        unsafe fn as_string(&self) -> String {
            if self.len == 0 || self.len > 1_000_000 {
                return String::new();
            }
            let data = if self.cap < 8 {
                self.storage.as_ptr().cast::<u16>()
            } else {
                std::ptr::read_unaligned(self.storage.as_ptr().cast::<*const u16>())
            };
            if data.is_null() {
                return String::new();
            }
            String::from_utf16_lossy(std::slice::from_raw_parts(data, self.len))
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Guid {
        d1: u32,
        d2: u16,
        d3: u16,
        d4: [u8; 8],
    }

    const FX_PROPERTY_STORE_CONTEXT: Guid = Guid {
        d1: 0x9150_BAF2,
        d2: 0xB024,
        d3: 0x47D1,
        d4: [0x91, 0x3B, 0x85, 0x86, 0xF8, 0x20, 0x2A, 0xB8],
    };
    const INPUT_DISABLE_ALL_EFFECTS: Guid = Guid {
        d1: 0x1DA5_D803,
        d2: 0xD492,
        d3: 0x4EDD,
        d4: [0x8C, 0x23, 0xE0, 0xC0, 0xFF, 0xEE, 0x7F, 0x0E],
    };
    const INPUT_DNS_AND_PROFILE: Guid = Guid {
        d1: 0xCEBE_CD49,
        d2: 0x7EBB,
        d3: 0x4DAC,
        d4: [0x9E, 0x86, 0x8A, 0x25, 0x72, 0xD4, 0xBF, 0xE5],
    };

    fn uses_fx_property_store_context() -> bool {
        let mut version = RtlOsVersionInfoW {
            size: std::mem::size_of::<RtlOsVersionInfoW>() as u32,
            major: 0,
            minor: 0,
            build: 0,
            platform_id: 0,
            service_pack: [0; 128],
        };
        (unsafe { RtlGetVersion(&mut version) == 0 })
            && (version.major > 10 || (version.major == 10 && version.build >= 22_000))
    }
    #[repr(C)]
    struct UnknownVt {
        query: usize,
        add_ref: usize,
        release: unsafe extern "system" fn(*mut Unknown) -> u32,
    }
    #[repr(C)]
    struct Unknown {
        vt: *const UnknownVt,
    }
    #[repr(C)]
    struct EnumeratorVt {
        query: usize,
        add_ref: usize,
        release: unsafe extern "system" fn(*mut Unknown) -> u32,
        enum_endpoints: unsafe extern "system" fn(*mut Unknown, i32, u32, *mut *mut Unknown) -> i32,
        get_default: unsafe extern "system" fn(*mut Unknown, i32, i32, *mut *mut Unknown) -> i32,
        get_device: unsafe extern "system" fn(*mut Unknown, *const u16, *mut *mut Unknown) -> i32,
    }
    #[repr(C)]
    struct CollectionVt {
        query: usize,
        add_ref: usize,
        release: unsafe extern "system" fn(*mut Unknown) -> u32,
        get_count: unsafe extern "system" fn(*mut Unknown, *mut u32) -> i32,
        item: unsafe extern "system" fn(*mut Unknown, u32, *mut *mut Unknown) -> i32,
    }
    #[repr(C)]
    struct DeviceVt {
        query: usize,
        add_ref: usize,
        release: unsafe extern "system" fn(*mut Unknown) -> u32,
        activate: unsafe extern "system" fn(
            *mut Unknown,
            *const Guid,
            u32,
            *mut PropVariant,
            *mut *mut Unknown,
        ) -> i32,
        open_store: unsafe extern "system" fn(*mut Unknown, u32, *mut *mut Unknown) -> i32,
        get_id: unsafe extern "system" fn(*mut Unknown, *mut *mut u16) -> i32,
        get_state: unsafe extern "system" fn(*mut Unknown, *mut u32) -> i32,
    }
    #[repr(C)]
    struct EndpointVolumeVt {
        query: usize,
        add_ref: usize,
        release: unsafe extern "system" fn(*mut Unknown) -> u32,
        register_control_change_notify: usize,
        unregister_control_change_notify: usize,
        get_channel_count: usize,
        set_master_volume_level: usize,
        set_master_volume_level_scalar:
            unsafe extern "system" fn(*mut Unknown, f32, *const Guid) -> i32,
        get_master_volume_level: usize,
        get_master_volume_level_scalar: unsafe extern "system" fn(*mut Unknown, *mut f32) -> i32,
        set_channel_volume_level: usize,
        set_channel_volume_level_scalar: usize,
        get_channel_volume_level: usize,
        get_channel_volume_level_scalar: usize,
        set_mute: usize,
        get_mute: usize,
        get_volume_step_info: usize,
        volume_step_up: usize,
        volume_step_down: usize,
        query_hardware_support: usize,
        get_volume_range: usize,
    }
    #[repr(C)]
    struct PropertyStoreVt {
        query: usize,
        add_ref: usize,
        release: unsafe extern "system" fn(*mut Unknown) -> u32,
        get_count: usize,
        get_at: usize,
        get_value:
            unsafe extern "system" fn(*mut Unknown, *const PropertyKey, *mut PropVariant) -> i32,
    }
    #[repr(C)]
    struct PropertyKey {
        fmtid: Guid,
        pid: u32,
    }
    #[repr(C)]
    union VariantData {
        pwsz: *mut u16,
        align: u64,
        storage: [u64; 2],
    }
    #[repr(C)]
    struct PropVariant {
        vt: u16,
        reserved: [u16; 3],
        data: VariantData,
    }
    const _: [(); 24] = [(); std::mem::size_of::<PropVariant>()];
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AudioDevice {
        id: String,
        name: String,
        is_default: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AudioDeviceLists {
        pub outputs: Vec<AudioDevice>,
        pub inputs: Vec<AudioDevice>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AudioParameter {
        pub key: String,
        pub value: Option<f32>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AudioProfile {
        pub name: String,
        pub alias: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AudioTuningState {
        pub output_endpoint: String,
        pub input_endpoint: Option<String>,
        pub volume: f32,
        pub parameters: Vec<AudioParameter>,
        pub profile: String,
        pub mode: Option<String>,
        pub current_preset: Option<String>,
        pub profiles: Vec<AudioProfile>,
        pub apo_error: Option<String>,
        pub input_enhancement: Option<bool>,
        pub input_profile: Option<String>,
        pub input_dns: Option<bool>,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LoadLibraryA(name: *const c_char) -> Handle;
        fn GetProcAddress(module: Handle, name: *const c_char) -> *mut c_void;
        fn FreeLibrary(module: Handle) -> i32;
    }
    #[link(name = "ole32")]
    unsafe extern "system" {
        fn CoInitializeEx(reserved: *mut c_void, coinit: u32) -> i32;
        fn CoUninitialize();
        fn CoCreateInstance(
            clsid: *const Guid,
            outer: *mut Unknown,
            context: u32,
            iid: *const Guid,
            out: *mut *mut Unknown,
        ) -> i32;
        fn CoTaskMemFree(mem: *mut c_void);
        fn PropVariantClear(value: *mut PropVariant) -> i32;
    }

    struct ComInitGuard;

    impl Drop for ComInitGuard {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
        }
    }

    fn initialize_com() -> Result<ComInitGuard, String> {
        let hr = unsafe { CoInitializeEx(std::ptr::null_mut(), 0) };
        if hr < 0 {
            return Err(format!("COM 初始化失败: HRESULT 0x{:08X}", hr as u32));
        }
        Ok(ComInitGuard)
    }

    const CLSID_MMDEVICE_ENUMERATOR: Guid = Guid {
        d1: 0xbcde0395,
        d2: 0xe52f,
        d3: 0x467c,
        d4: [0x8e, 0x3d, 0xc4, 0x57, 0x92, 0x91, 0x69, 0x2e],
    };
    const IID_IMMDEVICE_ENUMERATOR: Guid = Guid {
        d1: 0xa95664d2,
        d2: 0x9614,
        d3: 0x4f35,
        d4: [0xa7, 0x46, 0xde, 0x8d, 0xb6, 0x36, 0x17, 0xe6],
    };
    const IID_IAUDIO_ENDPOINT_VOLUME: Guid = Guid {
        d1: 0x5cdf2c82,
        d2: 0x841e,
        d3: 0x4546,
        d4: [0x97, 0x22, 0x0c, 0xf7, 0x40, 0x78, 0x22, 0x9a],
    };
    const PKEY_DEVICE_FRIENDLY_NAME: PropertyKey = PropertyKey {
        fmtid: Guid {
            d1: 0xa45c254e,
            d2: 0xdf1c,
            d3: 0x4efd,
            d4: [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0],
        },
        pid: 14,
    };
    const E_RENDER: i32 = 0;
    const E_CAPTURE: i32 = 1;
    const DEVICE_STATE_ACTIVE: u32 = 1;

    unsafe fn release(ptr: *mut Unknown) {
        if !ptr.is_null() {
            ((*(*ptr).vt).release)(ptr);
        }
    }
    fn wide(ptr: *const u16) -> String {
        if ptr.is_null() {
            return String::new();
        }
        let mut n = 0;
        unsafe {
            while *ptr.add(n) != 0 {
                n += 1;
            }
        }
        String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(ptr, n) })
    }

    fn wide_z(value: &str, label: &str) -> Result<Vec<u16>, String> {
        if value.contains('\0') {
            return Err(format!("{label}不能包含 NUL 字符"));
        }
        Ok(value.encode_utf16().chain(std::iter::once(0)).collect())
    }

    fn json_quote(value: &str, out: &mut String) {
        out.push('"');
        for ch in value.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c.is_control() => {
                    let _ = write!(out, "\\u{:04x}", c as u32);
                }
                c => out.push(c),
            }
        }
        out.push('"');
    }
    unsafe fn device_info(device: *mut Unknown, is_default: bool) -> Result<AudioDevice, String> {
        let vt = &*((*device).vt as *const DeviceVt);
        let mut id_ptr = std::ptr::null_mut();
        let hr = (vt.get_id)(device, &mut id_ptr);
        if hr < 0 {
            return Err(format!(
                "IMMDevice::GetId 失败: HRESULT 0x{:08X}",
                hr as u32
            ));
        }
        let id = wide(id_ptr);
        CoTaskMemFree(id_ptr.cast());
        let mut store = std::ptr::null_mut();
        let hr = (vt.open_store)(device, 0, &mut store);
        if hr < 0 {
            return Err(format!("打开设备属性失败: HRESULT 0x{:08X}", hr as u32));
        }
        let mut pv = PropVariant {
            vt: 0,
            reserved: [0; 3],
            data: VariantData { storage: [0; 2] },
        };
        let pvt = &*((*store).vt as *const PropertyStoreVt);
        let hr = (pvt.get_value)(store, &PKEY_DEVICE_FRIENDLY_NAME, &mut pv);
        let name = if hr >= 0 && pv.vt == 31 {
            wide(pv.data.pwsz)
        } else {
            "(名称不可用)".into()
        };
        PropVariantClear(&mut pv);
        release(store);
        Ok(AudioDevice {
            id,
            name,
            is_default,
        })
    }
    unsafe fn enumerate(flow: i32) -> Result<Vec<AudioDevice>, String> {
        let _guard = initialize_com()?;
        let mut en = std::ptr::null_mut();
        let hr = CoCreateInstance(
            &CLSID_MMDEVICE_ENUMERATOR,
            std::ptr::null_mut(),
            1,
            &IID_IMMDEVICE_ENUMERATOR,
            &mut en,
        );
        if hr < 0 {
            return Err(format!(
                "创建 MMDeviceEnumerator 失败: HRESULT 0x{:08X}",
                hr as u32
            ));
        }
        let evt = &*((*en).vt as *const EnumeratorVt);
        let mut default_ptr = std::ptr::null_mut();
        let default_hr = (evt.get_default)(en, flow, 0, &mut default_ptr);
        let default_id = if default_hr >= 0 {
            let d = device_info(default_ptr, true)?.id;
            release(default_ptr);
            Some(d)
        } else {
            None
        };
        let mut collection = std::ptr::null_mut();
        let hr = (evt.enum_endpoints)(en, flow, DEVICE_STATE_ACTIVE, &mut collection);
        if hr < 0 {
            release(en);
            return Err(format!("枚举音频端点失败: HRESULT 0x{:08X}", hr as u32));
        }
        let cvt = &*((*collection).vt as *const CollectionVt);
        let mut count = 0;
        (cvt.get_count)(collection, &mut count);
        let mut result = Vec::new();
        for i in 0..count {
            let mut device = std::ptr::null_mut();
            if (cvt.item)(collection, i, &mut device) >= 0 {
                let info = device_info(device, false);
                release(device);
                if let Ok(mut info) = info {
                    info.is_default = default_id.as_deref() == Some(&info.id);
                    result.push(info);
                }
            }
        }
        release(collection);
        release(en);
        Ok(result)
    }
    fn print_devices(flow: i32) -> Result<(), String> {
        let devices = unsafe { enumerate(flow)? };
        if devices.is_empty() {
            println!("没有找到活动音频设备");
        }
        for d in devices {
            println!(
                "{}\t{}\t{}",
                if d.is_default { "默认" } else { " " },
                d.name,
                d.id
            );
        }
        Ok(())
    }
    fn print_default(flow: i32) -> Result<(), String> {
        let devices = unsafe { enumerate(flow)? };
        if let Some(d) = devices.iter().find(|d| d.is_default) {
            println!("默认音源: {}\n端点ID: {}", d.name, d.id);
            match unsafe { Api::load(&d.id) } {
                Ok(api) => print!("{}", api.read()),
                Err(e) => println!("当前音源的 APO 参数暂不可读: {e}"),
            }
        } else {
            println!("当前没有默认活动音频端点");
        }
        Ok(())
    }
    fn default_id(flow: i32) -> Result<String, String> {
        let devices = unsafe { enumerate(flow)? };
        devices
            .iter()
            .find(|d| d.is_default)
            .map(|d| d.id.clone())
            .ok_or_else(|| "当前没有默认活动播放端点".into())
    }

    unsafe fn endpoint_volume(endpoint_id: &str, new_percent: Option<f32>) -> Result<f32, String> {
        let _guard = initialize_com()?;
        let mut en = std::ptr::null_mut();
        let hr = CoCreateInstance(
            &CLSID_MMDEVICE_ENUMERATOR,
            std::ptr::null_mut(),
            1,
            &IID_IMMDEVICE_ENUMERATOR,
            &mut en,
        );
        if hr < 0 {
            return Err(format!(
                "创建 MMDeviceEnumerator 失败: HRESULT 0x{:08X}",
                hr as u32
            ));
        }
        let id: Vec<u16> = endpoint_id
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let evt = &*((*en).vt as *const EnumeratorVt);
        let mut device = std::ptr::null_mut();
        let hr = (evt.get_device)(en, id.as_ptr(), &mut device);
        release(en);
        if hr < 0 {
            return Err(format!("打开音频端点失败: HRESULT 0x{:08X}", hr as u32));
        }
        let dvt = &*((*device).vt as *const DeviceVt);
        let mut endpoint_volume = std::ptr::null_mut();
        let hr = (dvt.activate)(
            device,
            &IID_IAUDIO_ENDPOINT_VOLUME,
            23, // CLSCTX_ALL
            std::ptr::null_mut(),
            &mut endpoint_volume,
        );
        release(device);
        if hr < 0 {
            return Err(format!(
                "激活 IAudioEndpointVolume 失败: HRESULT 0x{:08X}",
                hr as u32
            ));
        }
        let vt = &*((*endpoint_volume).vt as *const EndpointVolumeVt);
        let result = if let Some(percent) = new_percent {
            if !(0.0..=100.0).contains(&percent) {
                Err("音量范围必须是 0..100".to_string())
            } else {
                let hr = (vt.set_master_volume_level_scalar)(
                    endpoint_volume,
                    percent / 100.0,
                    std::ptr::null(),
                );
                if hr < 0 {
                    Err(format!("设置主音量失败: HRESULT 0x{:08X}", hr as u32))
                } else {
                    let mut actual = 0.0;
                    let read_hr = (vt.get_master_volume_level_scalar)(endpoint_volume, &mut actual);
                    if read_hr < 0 {
                        Err(format!(
                            "音量已设置，但回读失败: HRESULT 0x{:08X}",
                            read_hr as u32
                        ))
                    } else {
                        Ok(actual * 100.0)
                    }
                }
            }
        } else {
            let mut actual = 0.0;
            let hr = (vt.get_master_volume_level_scalar)(endpoint_volume, &mut actual);
            if hr < 0 {
                Err(format!("读取主音量失败: HRESULT 0x{:08X}", hr as u32))
            } else {
                Ok(actual * 100.0)
            }
        };
        release(endpoint_volume);
        result
    }

    const PARAMS: &[(u32, &str)] = &[
        (0, "audio_enhancement"),
        (1, "bass"),
        (2, "detail"),
        (3, "voice"),
        (4, "surround"),
        (5, "eq_enabled"),
        (6, "eq_31hz"),
        (7, "eq_62hz"),
        (8, "eq_125hz"),
        (9, "eq_250hz"),
        (10, "eq_500hz"),
        (11, "eq_1khz"),
        (12, "eq_2khz"),
        (13, "eq_4khz"),
        (14, "eq_8khz"),
        (15, "eq_16khz"),
        (16, "bypass_all"),
    ];

    fn ui_value(id: u32, raw: f32) -> f32 {
        match id {
            0 | 5 | 16 => raw,
            1..=4 => raw * 100.0,
            6..=15 => raw * 200.0 - 100.0,
            _ => raw,
        }
    }

    struct ParsedSnapshot {
        endpoint: String,
        profile: String,
        values: [f32; 17],
    }

    fn json_read_hex4(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Result<u16, String> {
        let digits: String = (0..4)
            .map(|_| {
                chars
                    .next()
                    .ok_or_else(|| "JSON Unicode 转义不完整".to_string())
            })
            .collect::<Result<_, _>>()?;
        u16::from_str_radix(&digits, 16).map_err(|_| "JSON Unicode 转义格式错误".to_string())
    }

    fn json_unquote(token: &str) -> Result<String, String> {
        let mut chars = token.trim().chars().peekable();
        if chars.next() != Some('"') {
            return Err("JSON 字符串缺少起始引号".into());
        }
        let mut value = String::new();
        loop {
            match chars.next() {
                Some('"') => {
                    if chars.any(|ch| !ch.is_whitespace()) {
                        return Err("JSON 字符串后存在多余内容".into());
                    }
                    return Ok(value);
                }
                Some('\\') => match chars.next() {
                    Some('"') => value.push('"'),
                    Some('\\') => value.push('\\'),
                    Some('/') => value.push('/'),
                    Some('b') => value.push('\u{0008}'),
                    Some('f') => value.push('\u{000c}'),
                    Some('n') => value.push('\n'),
                    Some('r') => value.push('\r'),
                    Some('t') => value.push('\t'),
                    Some('u') => {
                        let first = json_read_hex4(&mut chars)?;
                        let codepoint = if (0xd800..=0xdbff).contains(&first) {
                            if chars.next() != Some('\\') || chars.next() != Some('u') {
                                return Err("JSON 高位代理项缺少低位代理项".into());
                            }
                            let second = json_read_hex4(&mut chars)?;
                            if !(0xdc00..=0xdfff).contains(&second) {
                                return Err("JSON 低位代理项无效".into());
                            }
                            0x10000 + (((first as u32 - 0xd800) << 10) | (second as u32 - 0xdc00))
                        } else if (0xdc00..=0xdfff).contains(&first) {
                            return Err("JSON 存在未配对的低位代理项".into());
                        } else {
                            first as u32
                        };
                        value.push(char::from_u32(codepoint).ok_or("JSON Unicode 码点无效")?);
                    }
                    _ => return Err("JSON 字符串包含未知转义".into()),
                },
                Some(ch) if ch.is_control() => return Err("JSON 字符串包含未转义控制字符".into()),
                Some(ch) => value.push(ch),
                None => return Err("JSON 字符串缺少结束引号".into()),
            }
        }
    }

    fn parse_profile_snapshot(text: &str) -> Result<ParsedSnapshot, String> {
        let mut version_ok = false;
        let mut endpoint = None;
        let mut profile = None;
        let mut values = [None; 17];
        let mut in_parameters = false;
        let mut parameters_closed = false;

        for line in text.lines() {
            let line = line.trim();
            if in_parameters {
                if line == "}" {
                    in_parameters = false;
                    parameters_closed = true;
                    continue;
                }
                let entry = line.strip_suffix(',').unwrap_or(line);
                let (key, value) = entry
                    .split_once(':')
                    .ok_or_else(|| format!("快照参数行格式错误：{line}"))?;
                let key = json_unquote(key)?;
                let index = PARAMS
                    .iter()
                    .position(|(_, name)| *name == key)
                    .ok_or_else(|| format!("快照含未知参数“{key}”"))?;
                if values[index].is_some() {
                    return Err(format!("快照重复包含参数“{key}”"));
                }
                let parsed: f32 = value
                    .trim()
                    .parse()
                    .map_err(|_| format!("参数 {key} 不是有效数字"))?;
                let id = PARAMS[index].0;
                let valid = match id {
                    0 | 5 | 16 => (0.0..=1.0).contains(&parsed),
                    1..=4 => (0.0..=100.0).contains(&parsed),
                    6..=15 => (-100.0..=100.0).contains(&parsed),
                    _ => false,
                };
                if !valid {
                    return Err(format!("快照参数 {key} 越界：{parsed}"));
                }
                values[index] = Some(parsed);
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                match key.trim() {
                    "\"format_version\"" => version_ok = value.trim().trim_end_matches(',') == "1",
                    "\"endpoint_id\"" => {
                        endpoint = Some(json_unquote(value.trim().trim_end_matches(','))?)
                    }
                    "\"profile\"" => {
                        profile = Some(json_unquote(value.trim().trim_end_matches(','))?)
                    }
                    "\"parameters\"" if value.trim() == "{" => in_parameters = true,
                    _ => {}
                }
            }
        }

        if !version_ok {
            return Err("快照格式版本缺失或不支持（要求 format_version=1）".into());
        }
        if in_parameters || !parameters_closed {
            return Err("快照 parameters 对象未完整结束".into());
        }
        let endpoint = endpoint.ok_or("快照缺少 endpoint_id")?;
        let profile = profile.ok_or("快照缺少 profile")?;
        let mut complete_values = [0.0; 17];
        for (index, value) in values.into_iter().enumerate() {
            complete_values[index] =
                value.ok_or_else(|| format!("快照缺少参数 {}", PARAMS[index].1))?;
        }
        Ok(ParsedSnapshot {
            endpoint,
            profile,
            values: complete_values,
        })
    }

    // The preset source values and their order come from the NSS
    // NUCAudioStudioLibNET MainModel EQPresets table.
    const EQ_PRESETS: &[(&str, [f32; 10])] = &[
        ("default", [0.0; 10]),
        (
            "acoustic",
            [0.5, 2.0, 3.0, 2.0, 1.4736842, 0.5, 3.5, 3.0, 3.0, 4.0],
        ),
        ("blues", [1.0, 2.5, 3.0, 1.5, 0.2, -1.0, 2.0, 2.0, 3.0, 3.0]),
        (
            "classical",
            [3.0, 4.0, 1.5, 0.9736842, 1.0, 0.0, 1.0, 2.0, 3.0, 4.0],
        ),
        (
            "country",
            [1.5, 2.0, 3.0, 2.5, 1.0, 0.0, 2.0, 2.0526316, 3.5, 2.0],
        ),
        ("dance", [2.0, 4.0, 3.0, -1.0, 0.0, 0.0, 1.0, 3.0, 3.0, 4.0]),
        (
            "electronic",
            [2.0, 4.0, 1.0, -1.0, 1.5, 0.0, 2.0, 3.0, 3.5, 3.5],
        ),
        (
            "hip_hop",
            [3.0, 4.0, 2.0, -1.0, 0.0, -2.0, 2.0, 3.0, 4.0, 2.0],
        ),
        ("jazz", [2.0, 3.0, 2.0, 1.5, 0.0, 0.0, 2.5, 2.0, 3.5, 3.0]),
        (
            "metal",
            [3.5, 4.0, 1.5, 2.0, 1.0, 0.5, 1.5263158, 2.5, 3.0, 2.0],
        ),
        ("pop", [0.0, 4.0, 1.0, 1.0, -0.5, 0.0, 3.5, 4.0, 4.0, 4.0]),
        ("rock", [3.0, 3.0, 1.5, 1.0, 1.0, 0.0, 4.0, 2.5, 4.0, 4.0]),
    ];

    fn detect_current_preset(parameters: &[AudioParameter]) -> Option<String> {
        let current_bands: Vec<f32> = PARAMS
            .iter()
            .skip(6)
            .take(10)
            .map(|(_, key)| {
                parameters
                    .iter()
                    .find(|parameter| parameter.key == *key)
                    .and_then(|parameter| parameter.value)
            })
            .collect::<Option<_>>()?;

        EQ_PRESETS
            .iter()
            .find(|(_, preset_values)| {
                current_bands
                    .iter()
                    .zip(preset_values)
                    .all(|(current, preset)| {
                        let expected = (preset + 18.0) * (200.0 / 36.0) - 100.0;
                        (current - expected).abs() <= 1.5
                    })
            })
            .map(|(name, _)| (*name).to_string())
    }

    fn error_name(code: i32) -> &'static str {
        match code {
            0 => "SUCCESS",
            -1 => "INVALID_HANDLE",
            -2 => "INVALID_APO_TYPE",
            -3 => "INVALID_PARAMETER_ID",
            -4 => "INVALID_PARAMETER_VALUE",
            -8 => "PRESET_DOESNT_EXIST",
            -9 => "APO_TYPE_NOT_SET",
            -10 => "PARAMETER_NOT_SUPPORTED",
            -11 => "INVALID_ENDPOINT_ID",
            -12 => "FAILED_TO_LOAD_SECOMN_DLL",
            -13 => "INCOMPLETE_API_IN_SECOMN_DLL",
            -14 => "SECOMN_EXE_RPC_FAILURE",
            -15 => "ENDPOINT_ACCESS_FAILURE",
            -16 => "ENDPOINT_PROP_ACCESS_FAILURE",
            -17 => "ENDPOINT_PROP_VALTYPE_NOTSUPPORTED",
            -18 => "ENGINE_NOT_SET",
            -19 => "INVALID_ENGINE_NAME",
            -20 => "PROFILE_ALIAS_DOESNT_EXIST",
            -21 => "FAILED_TO_SET_PROFILE_ALIAS",
            -22 => "INVALID_PARAMETER_SIZE",
            -256 => "WINDOWS_ERRORCODE",
            _ => "UNKNOWN",
        }
    }

    fn windows_error_name(code: u32) -> &'static str {
        match code {
            5 => "ACCESS_DENIED；当前调用上下文未获该操作授权",
            87 => "INVALID_PARAMETER",
            1168 => "NOT_FOUND",
            _ => "WIN32_ERROR",
        }
    }

    static ISOLATED_AUDIO_WORKER: AtomicBool = AtomicBool::new(false);

    struct Api {
        module: Handle,
        api: Handle,
        h: Handle,
        get: GetFn,
        get_last_error: GetLastErrorFn,
        get_last_windows_error: GetLastWindowsErrorFn,
        set: SetFn,
        content: ContentFn,
        get_content: Option<GetContentFn>,
        get_profile: GetProfileFn,
        get_profiles: GetProfilesFn,
        free_profiles: FreeProfilesFn,
        set_profile: SetProfileFn,
        get_profile_alias: GetProfileAliasFn,
        delete_profile: DeleteProfileFn,
        copy_profile: CopyProfileFn,
        set_profile_alias: SetProfileAliasFn,
        reset_profiles: ResetProfilesFn,
        get_fx_property_ex: GetFxPropertyExFn,
        set_fx_property_ex: SetFxPropertyExFn,
        get_fx_property_ex2: GetFxPropertyEx2Fn,
        set_fx_property_ex2: SetFxPropertyEx2Fn,
        free_api: FreeApiFn,
    }
    impl Drop for Api {
        fn drop(&mut self) {
            // The vendor DLL's cleanup path corrupts the worker process during teardown.
            // Each isolated worker handles one request, so let Windows reclaim these
            // handles when that process exits instead of calling the unstable API cleanup.
            if ISOLATED_AUDIO_WORKER.load(Ordering::Relaxed) {
                return;
            }
            unsafe {
                (self.free_api)(self.api);
                FreeLibrary(self.module);
            }
        }
    }
    impl Api {
        unsafe fn load(endpoint: &str) -> Result<Self, String> {
            Self::load_with_proxy(endpoint, false)
        }
        unsafe fn load_with_proxy(endpoint: &str, no_proxy_apo: bool) -> Result<Self, String> {
            let module = LoadLibraryA(c"SECOMN64.dll".as_ptr());
            if module.is_null() {
                return Err("找不到 SECOMN64.dll。请安装/修复 Sound Research Audio Effects Component 驱动。".into());
            }
            macro_rules! sym {
                ($name:literal, $ty:ty) => {{
                    let p = GetProcAddress(module, concat!($name, "\0").as_ptr().cast());
                    if p.is_null() {
                        FreeLibrary(module);
                        return Err(concat!("SECOMN64.dll 缺少导出 ", $name).into());
                    }
                    std::mem::transmute::<*mut c_void, $ty>(p)
                }};
            }
            let init: ApiInitFn = sym!("SEAPOCOMAPI_InitializeAPI", ApiInitFn);
            let set_apo_type: SetApoTypeFn = sym!("SEAPOCOMAPI_SetAPOType", SetApoTypeFn);
            let get_endpoint: GetEndpointFn = sym!("SEAPOCOMAPI_GetEndpoint", GetEndpointFn);
            let get_endpoint_ex: GetEndpointExFn =
                sym!("SEAPOCOMAPI_GetEndpointEx", GetEndpointExFn);
            let get = sym!("SEAPOCOMAPI_GetUserParameter", GetFn);
            let get_last_error = sym!("SEAPOCOMAPI_GetLastError", GetLastErrorFn);
            let get_last_windows_error =
                sym!("SEAPOCOMAPI_GetWindowsLastError", GetLastWindowsErrorFn);
            let set = sym!("SEAPOCOMAPI_SetUserParameter", SetFn);
            let content = sym!("SEAPOCOMAPI_SetContentType", ContentFn);
            let get_content_ptr = GetProcAddress(module, c"SEAPOCOMAPI_GetContentType".as_ptr());
            let get_content = (!get_content_ptr.is_null()).then(|| {
                std::mem::transmute::<*mut c_void, GetContentFn>(get_content_ptr)
            });
            let get_profile = sym!("SEAPOCOMAPI_GetUserProfileW", GetProfileFn);
            let get_profiles = sym!("SEAPOCOMAPI_GetUserProfiles", GetProfilesFn);
            let free_profiles = sym!("SEAPOCOMAPI_FreeProfileNames", FreeProfilesFn);
            let set_profile = sym!("SEAPOCOMAPI_SetUserProfileW", SetProfileFn);
            let get_profile_alias = sym!("SEAPOCOMAPI_GetUserProfileAlias", GetProfileAliasFn);
            let delete_profile = sym!("SEAPOCOMAPI_DeleteUserProfile", DeleteProfileFn);
            let copy_profile = sym!("SEAPOCOMAPI_CopyUserProfile", CopyProfileFn);
            let set_profile_alias = sym!("SEAPOCOMAPI_SetUserProfileAlias", SetProfileAliasFn);
            let reset_profiles = sym!("SEAPOCOMAPI_ResetUserParameters", ResetProfilesFn);
            let get_fx_property_ex = sym!(
                "SEAPOCOMAPI_GetEndpointFxPropertyValueEx",
                GetFxPropertyExFn
            );
            let set_fx_property_ex = sym!(
                "SEAPOCOMAPI_SetEndpointFxPropertyValueEx",
                SetFxPropertyExFn
            );
            let get_fx_property_ex2 = sym!(
                "SEAPOCOMAPI_GetEndpointFxPropertyValueEx2",
                GetFxPropertyEx2Fn
            );
            let set_fx_property_ex2 = sym!(
                "SEAPOCOMAPI_SetEndpointFxPropertyValueEx2",
                SetFxPropertyEx2Fn
            );
            let free_api: FreeApiFn = sym!("SEAPOCOMAPI_FreeAPI", FreeApiFn);
            let api = init();
            if api.is_null() {
                FreeLibrary(module);
                return Err("SEAPOCOMAPI_InitializeAPI 返回空句柄".into());
            }
            let ep: Vec<u16> = endpoint.encode_utf16().chain(std::iter::once(0)).collect();
            let zero_guid = Guid {
                d1: 0,
                d2: 0,
                d3: 0,
                d4: [0; 8],
            };
            let no_proxy_apo_flag = i32::from(no_proxy_apo);
            let endpoint_handle = get_endpoint_ex(api, ep.as_ptr(), no_proxy_apo_flag, &zero_guid);
            let endpoint_handle = if endpoint_handle == 0 {
                get_endpoint(api, ep.as_ptr(), no_proxy_apo_flag)
            } else {
                endpoint_handle
            };
            if endpoint_handle == 0 {
                free_api(api);
                FreeLibrary(module);
                return Err(format!(
                    "SEAPOCOMAPI_GetEndpoint 无法打开 endpoint: {endpoint}"
                ));
            }
            if set_apo_type(endpoint_handle as usize as Handle, c"EFX".as_ptr()) == 0 {
                free_api(api);
                FreeLibrary(module);
                return Err("SEAPOCOMAPI_SetAPOType(EFX) 失败".into());
            }
            Ok(Self {
                module,
                api,
                h: endpoint_handle as usize as Handle,
                get,
                get_last_error,
                get_last_windows_error,
                set,
                content,
                get_content,
                get_profile,
                get_profiles,
                free_profiles,
                set_profile,
                get_profile_alias,
                delete_profile,
                copy_profile,
                set_profile_alias,
                reset_profiles,
                get_fx_property_ex,
                set_fx_property_ex,
                get_fx_property_ex2,
                set_fx_property_ex2,
                free_api,
            })
        }
        fn read(&self) -> String {
            let mut out = String::new();
            for &(id, name) in PARAMS {
                let raw = unsafe { (self.get)(self.h, id as i32) };
                let api_error = unsafe { (self.get_last_error)(self.api) };
                if api_error != 0 || !(0.0..=1.0).contains(&raw) {
                    let win = if api_error == -256 {
                        let code = unsafe { (self.get_last_windows_error)(self.api) };
                        format!(", Win32={code} ({})", windows_error_name(code))
                    } else {
                        String::new()
                    };
                    let _ = writeln!(
                        out,
                        "{name}=<不可用，驱动返回 {raw:.3}，API错误 {api_error}: {}{}>",
                        error_name(api_error),
                        win
                    );
                    continue;
                }
                let v = ui_value(id, raw);
                let _ = writeln!(out, "{name}={v:.2}");
            }
            out
        }
        fn profile_snapshot_json(&self, endpoint: &str) -> Result<String, String> {
            let profile = self.profile()?;
            let mut out = String::new();
            out.push_str("{\n  \"format_version\": 1,\n  \"endpoint_id\": ");
            json_quote(endpoint, &mut out);
            out.push_str(",\n  \"profile\": ");
            json_quote(&profile, &mut out);
            out.push_str(",\n  \"parameters\": {\n");

            for (index, &(id, name)) in PARAMS.iter().enumerate() {
                let raw = unsafe { (self.get)(self.h, id as i32) };
                let api_error = unsafe { (self.get_last_error)(self.api) };
                if api_error != 0 || !(0.0..=1.0).contains(&raw) {
                    return Err(format!(
                        "导出中止：读取 {name} 失败（值={raw:.4}；{}）",
                        self.last_error_context()
                    ));
                }
                let value = match id {
                    0 | 5 | 16 => raw,
                    1..=4 => raw * 100.0,
                    6..=15 => raw * 200.0 - 100.0,
                    _ => raw,
                };
                out.push_str("    ");
                json_quote(name, &mut out);
                let _ = write!(
                    out,
                    ": {value:.4}{}\n",
                    if index + 1 == PARAMS.len() { "" } else { "," }
                );
            }
            out.push_str("  }\n}\n");
            Ok(out)
        }
        fn profile_import(&self, endpoint: &str, snapshot: &str) -> Result<(), String> {
            let parsed = parse_profile_snapshot(snapshot)?;
            if parsed.endpoint != endpoint {
                return Err(format!(
                    "快照属于端点 {}，当前目标为 {endpoint}；拒绝跨端点恢复",
                    parsed.endpoint
                ));
            }
            let current_profile = self.profile()?;
            if current_profile != parsed.profile {
                return Err(format!(
                    "快照 profile 为“{}”，当前 profile 为“{}”；拒绝写入其他 profile",
                    parsed.profile, current_profile
                ));
            }

            // Read and validate the whole snapshot before the first write.
            // ID 16 (bypass_all) is currently read-only in this CLI; refuse a
            // restore that would silently leave that value different.
            let mut previous = [0.0f32; 17];
            for (index, &(id, name)) in PARAMS.iter().enumerate() {
                let raw = unsafe { (self.get)(self.h, id as i32) };
                let api_error = unsafe { (self.get_last_error)(self.api) };
                if api_error != 0 || !(0.0..=1.0).contains(&raw) {
                    return Err(format!(
                        "恢复前读取 {name} 失败（值={raw:.4}；{}）；没有写入任何参数",
                        self.last_error_context()
                    ));
                }
                previous[index] = ui_value(id, raw);
                if index == 16 && (parsed.values[index] - previous[index]).abs() > 0.001 {
                    return Err(format!(
                        "快照中的 bypass_all={}，当前值={}；该参数尚未开放写入，未修改任何设置",
                        parsed.values[index], previous[index]
                    ));
                }
            }

            let mut attempted = Vec::new();
            for index in 0..16 {
                let name = PARAMS[index].1;
                attempted.push(index);
                if let Err(error) = self.write(name, parsed.values[index]) {
                    let mut rollback_errors = Vec::new();
                    for rollback_index in attempted.iter().copied().rev() {
                        let rollback_name = PARAMS[rollback_index].1;
                        if let Err(rollback_error) =
                            self.write(rollback_name, previous[rollback_index])
                        {
                            rollback_errors.push(format!("{rollback_name}: {rollback_error}"));
                        }
                    }
                    return Err(if rollback_errors.is_empty() {
                        format!("恢复 {name} 失败；已尝试回滚此前写入：{error}")
                    } else {
                        format!(
                            "恢复 {name} 失败：{error}；回滚也有失败：{}",
                            rollback_errors.join("；")
                        )
                    });
                }
            }
            Ok(())
        }
        fn write(&self, key: &str, value: f32) -> Result<(), String> {
            let (id, _) = PARAMS
                .iter()
                .find(|(_, n)| *n == key)
                .ok_or_else(|| format!("未知设置项 {key}; 使用 list 查看名称"))?;
            let valid = match *id {
                0 | 5 => (0.0..=1.0).contains(&value),
                1..=4 => (0.0..=100.0).contains(&value),
                6..=15 => (-100.0..=100.0).contains(&value),
                16 => false,
                _ => false,
            };
            if !valid {
                return Err(format!("{key} 值越界"));
            }
            let current = unsafe { (self.get)(self.h, *id as i32) };
            let api_error = unsafe { (self.get_last_error)(self.api) };
            if api_error != 0 || !(0.0..=1.0).contains(&current) {
                return Err(format!(
                    "当前端点无法读取 {key}（驱动返回 {current:.3}；{}），已拒绝写入以避免盲写",
                    self.last_error_context()
                ));
            }
            let raw = match *id {
                0 | 5 => value,
                1..=4 => value / 100.0,
                6..=15 => (value + 100.0) / 200.0,
                _ => value,
            };
            let rc = unsafe { (self.set)(self.h, *id as i32, raw) };
            if rc != 0 {
                let got = unsafe { (self.get)(self.h, *id as i32) };
                let readback = match *id {
                    0 | 5 => got,
                    1..=4 => got * 100.0,
                    6..=15 => got * 200.0 - 100.0,
                    _ => got,
                };
                if (readback - value).abs() > 0.02 {
                    Err(format!(
                        "写入请求成功，但回读值为 {readback:.2}（请求 {value:.2}）；驱动可能做了量化或钳位"
                    ))
                } else {
                    Ok(())
                }
            } else {
                Err(format!("写入失败，驱动返回码 {rc}"))
            }
        }
        fn apply_preset(&self, name: &str) -> Result<(), String> {
            let (_, values) = EQ_PRESETS
                .iter()
                .find(|(preset, _)| preset.eq_ignore_ascii_case(name))
                .ok_or_else(|| {
                    format!(
                        "未知 EQ 预设“{name}”；可选：{}",
                        EQ_PRESETS
                            .iter()
                            .map(|(n, _)| *n)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })?;

            // Read all bands before the first write, so an inaccessible APO
            // cannot leave a half-applied preset.
            let mut previous = [0.0f32; 10];
            for (index, old) in previous.iter_mut().enumerate() {
                let id = 6 + index as i32;
                *old = unsafe { (self.get)(self.h, id) };
                let error = unsafe { (self.get_last_error)(self.api) };
                if error != 0 || !(0.0..=1.0).contains(old) {
                    let win = if error == -256 {
                        let code = unsafe { (self.get_last_windows_error)(self.api) };
                        format!(", Win32={code} ({})", windows_error_name(code))
                    } else {
                        String::new()
                    };
                    return Err(format!(
                        "应用预设前读取 EQ band {} 失败：API {error} {}{}；没有写入任何频段",
                        index + 1,
                        error_name(error),
                        win
                    ));
                }
            }

            let rollback = || {
                let mut failures = Vec::new();
                for (index, raw) in previous.iter().copied().enumerate() {
                    let id = 6 + index as i32;
                    let set_ok = unsafe { (self.set)(self.h, id, raw) } != 0;
                    let got = unsafe { (self.get)(self.h, id) };
                    let error = unsafe { (self.get_last_error)(self.api) };
                    if !set_ok || error != 0 || (got - raw).abs() > 0.005 {
                        failures.push(format!("EQ band {}", index + 1));
                    }
                }
                failures
            };

            for (index, source_value) in values.iter().copied().enumerate() {
                // Preserve the NSS preset mapping into the -100..100 EQ slider.
                let ui_value = (source_value + 18.0) * (200.0 / 36.0) - 100.0;
                let raw = (ui_value + 100.0) / 200.0;
                let id = 6 + index as i32;
                let set_ok = unsafe { (self.set)(self.h, id, raw) } != 0;
                let set_error = unsafe { (self.get_last_error)(self.api) };
                let got = unsafe { (self.get)(self.h, id) };
                let get_error = unsafe { (self.get_last_error)(self.api) };
                if !set_ok || set_error != 0 || get_error != 0 || (got - raw).abs() > 0.005 {
                    let rollback_failures = rollback();
                    return Err(format!(
                        "预设“{name}”写入 EQ band {} 失败（set={set_error} {}, read={get_error} {}）；已尝试恢复旧 EQ{}",
                        index + 1,
                        error_name(set_error),
                        error_name(get_error),
                        if rollback_failures.is_empty() {
                            String::new()
                        } else {
                            format!("；恢复未确认：{}", rollback_failures.join(", "))
                        }
                    ));
                }
            }
            Ok(())
        }
        fn mode(&self, mode: &str) -> Result<(), String> {
            if !matches!(mode, "AnyContent" | "Voice" | "Movie" | "Gaming") {
                return Err("音效模式应为 AnyContent、Voice、Movie 或 Gaming".into());
            }
            let c = std::ffi::CString::new(mode).map_err(|_| "模式名称不能包含 NUL 字符")?;
            let rc = unsafe { (self.content)(self.h, c.as_ptr()) };
            if rc != 0 {
                Ok(())
            } else {
                Err(format!("模式切换失败，驱动返回码 {rc}"))
            }
        }
        fn current_mode(&self) -> Option<String> {
            let get_content = self.get_content?;
            let mut buffer = [0 as c_char; 64];
            let ok = unsafe {
                get_content(self.h, buffer.as_mut_ptr(), buffer.len() as i32)
            };
            if ok == 0 {
                return None;
            }
            let bytes = unsafe {
                std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), buffer.len())
            };
            let end = bytes.iter().position(|byte| *byte == 0)?;
            let mode = std::str::from_utf8(&bytes[..end]).ok()?.trim();
            matches!(mode, "AnyContent" | "Voice" | "Movie" | "Gaming")
                .then(|| mode.to_string())
        }
        fn profile(&self) -> Result<String, String> {
            let mut native = MsvcWString::empty();
            let ok = unsafe { (self.get_profile)(self.h, &mut native) };
            if ok == 0 {
                return Err(format!(
                    "读取当前配置文件失败（{}）",
                    self.last_error_context()
                ));
            }
            let name = unsafe { native.as_string() };
            Ok(name)
        }
        fn profiles(&self) -> Result<Vec<String>, String> {
            let mut count = 0i32;
            let mut ptr: *mut *mut u16 = std::ptr::null_mut();
            let mut tag = 0u64;
            let ok = unsafe { (self.get_profiles)(self.h, &mut count, &mut ptr, &mut tag) };
            if ok == 0 {
                return Err(format!(
                    "枚举配置文件失败（tag={tag}；{}）",
                    self.last_error_context()
                ));
            }
            if count < 0 || count > 4096 {
                return Err(format!("驱动返回异常配置文件数量 {count}"));
            }
            let mut names = Vec::with_capacity(count as usize);
            if !ptr.is_null() {
                for i in 0..count as usize {
                    let item = unsafe { *ptr.add(i) };
                    names.push(wide(item));
                }
                unsafe { (self.free_profiles)(self.h, count, ptr, tag) };
            }
            Ok(names)
        }
        fn last_error_context(&self) -> String {
            let api_error = unsafe { (self.get_last_error)(self.api) };
            let windows_error = unsafe { (self.get_last_windows_error)(self.api) };
            format!(
                "Sound Research API 错误 {api_error} ({})，Win32 错误 {windows_error} ({})",
                error_name(api_error),
                windows_error_name(windows_error)
            )
        }
        fn set_profile(&self, name: &str) -> Result<(), String> {
            let actual_name = if name.eq_ignore_ascii_case("default") || name == "默认" {
                ""
            } else {
                name
            };
            let available = self.profiles()?;
            if !actual_name.is_empty() && !available.iter().any(|n| n == actual_name) {
                return Err(format!(
                    "当前端点不支持配置文件“{name}”；先运行 profiles 查看可选项"
                ));
            }
            let wide = wide_z(actual_name, "配置文件名称")?;
            let rc = unsafe { (self.set_profile)(self.h, wide.as_ptr()) };
            if rc == 0 {
                return Err(format!(
                    "切换配置文件失败：{name}（{}）",
                    self.last_error_context()
                ));
            }
            let current = self.profile()?;
            if current != actual_name {
                return Err(format!("配置文件切换后回读为“{current}”，请求“{name}”"));
            }
            Ok(())
        }
        fn set_profile_alias(&self, name: &str, alias: &str) -> Result<(), String> {
            if !self.profiles()?.iter().any(|candidate| candidate == name) {
                return Err(format!("不存在配置文件“{name}”"));
            }
            if alias.trim().is_empty() {
                return Err("配置文件显示名称不能为空".into());
            }
            let name_wide = wide_z(name, "配置文件名称")?;
            let alias_wide = wide_z(alias, "配置文件显示名称")?;
            let ok = unsafe {
                (self.set_profile_alias)(self.h, name_wide.as_ptr(), alias_wide.as_ptr())
            };
            if ok == 0 {
                return Err(format!(
                    "设置配置文件显示名称失败（{}）",
                    self.last_error_context()
                ));
            }
            Ok(())
        }
        fn profile_alias(&self, name: &str) -> Result<String, String> {
            if !self.profiles()?.iter().any(|candidate| candidate == name) {
                return Err(format!("不存在配置文件“{name}”"));
            }
            let name_wide = wide_z(name, "配置文件名称")?;
            // The native export copies into a fixed 0x200-byte wide buffer.
            let mut alias = [0u16; 256];
            let ok =
                unsafe { (self.get_profile_alias)(self.h, name_wide.as_ptr(), alias.as_mut_ptr()) };
            if ok == 0 {
                return Err(format!(
                    "读取配置文件显示名称失败（{}）",
                    self.last_error_context()
                ));
            }
            Ok(wide(alias.as_ptr()))
        }
        fn copy_profile(&self, source: &str, destination: &str) -> Result<(), String> {
            let names = self.profiles()?;
            if !source.is_empty() && !names.iter().any(|name| name == source) {
                return Err(format!("源配置文件“{source}”不存在"));
            }
            if names.iter().any(|name| name == destination) {
                return Err(format!("目标配置文件“{destination}”已存在"));
            }
            let source_wide = wide_z(source, "源配置文件名称")?;
            let destination_wide = wide_z(destination, "目标配置文件名称")?;
            let ok = unsafe {
                (self.copy_profile)(self.h, source_wide.as_ptr(), destination_wide.as_ptr())
            };
            if ok == 0 {
                return Err(format!("复制配置文件失败（{}）", self.last_error_context()));
            }
            if !self.profiles()?.iter().any(|name| name == destination) {
                return Err("驱动报告复制成功，但新配置文件未出现在回读列表中".into());
            }
            Ok(())
        }
        fn create_profile(&self, alias: &str) -> Result<String, String> {
            if alias.trim().is_empty() {
                return Err("配置文件显示名称不能为空".into());
            }
            let source = self.profile()?;
            let names = self.profiles()?;
            let mut index = 1u32;
            let destination = loop {
                let candidate = format!("Profile{index}");
                if !names.iter().any(|name| name == &candidate) {
                    break candidate;
                }
                index = index
                    .checked_add(1)
                    .ok_or_else(|| "可用 ProfileN 名称已耗尽".to_string())?;
            };
            self.copy_profile(&source, &destination)?;
            if let Err(error) = self.set_profile_alias(&destination, alias) {
                let _ = self.delete_profile(&destination);
                return Err(format!("创建档案后设置显示名称失败，已尝试回滚：{error}"));
            }
            if let Err(error) = self.set_profile(&destination) {
                let _ = self.delete_profile(&destination);
                return Err(format!("新档案已复制但切换失败，已尝试回滚：{error}"));
            }
            Ok(destination)
        }
        fn delete_profile(&self, name: &str) -> Result<(), String> {
            if !self.profiles()?.iter().any(|candidate| candidate == name) {
                return Err(format!("配置文件“{name}”不存在"));
            }
            let name_wide = wide_z(name, "配置文件名称")?;
            let ok = unsafe { (self.delete_profile)(self.h, name_wide.as_ptr()) };
            if ok == 0 {
                return Err(format!("删除配置文件失败（{}）", self.last_error_context()));
            }
            if self.profiles()?.iter().any(|candidate| candidate == name) {
                return Err(format!(
                    "驱动报告已删除，但配置文件“{name}”仍出现在回读列表中"
                ));
            }
            Ok(())
        }
        fn reset_parameters(&self, globally: bool) -> Result<(), String> {
            let ok = unsafe { (self.reset_profiles)(self.h, if globally { 1 } else { 0 }) };
            if ok == 0 {
                return Err(format!(
                    "{}失败（{}）",
                    if globally {
                        "恢复所有参数默认值"
                    } else {
                        "恢复当前 profile 默认值"
                    },
                    self.last_error_context()
                ));
            }
            Ok(())
        }
        fn get_input_fx_property(
            &self,
            endpoint: &str,
            property: &Guid,
            property_id: i32,
            use_store_context: bool,
        ) -> Result<Option<u32>, String> {
            let endpoint_wide = wide_z(endpoint, "输入 endpoint ID")?;
            let mut value = Variant::default();
            let ok = unsafe {
                if use_store_context && uses_fx_property_store_context() {
                    (self.get_fx_property_ex2)(
                        self.api,
                        endpoint_wide.as_ptr(),
                        &FX_PROPERTY_STORE_CONTEXT,
                        0,
                        property,
                        property_id,
                        &mut value,
                    )
                } else {
                    (self.get_fx_property_ex)(
                        self.api,
                        endpoint_wide.as_ptr(),
                        property,
                        property_id,
                        &mut value,
                    )
                }
            };
            if ok == 0 {
                unsafe { VariantClear(&mut value) };
                let api_error = unsafe { (self.get_last_error)(self.api) };
                let windows_error = unsafe { (self.get_last_windows_error)(self.api) };
                if matches!(api_error, -15 | -16 | -17) {
                    return Ok(None);
                }
                return Err(format!(
                    "读取输入 APO FX 属性 GUID={:08X}-{:04X}-{:04X} ID={property_id} 失败（API {api_error} {}，Win32 {windows_error} {}）",
                    property.d1,
                    property.d2,
                    property.d3,
                    error_name(api_error),
                    windows_error_name(windows_error)
                ));
            }
            let variant_type = value.vt;
            let raw = value.data as u32;
            unsafe { VariantClear(&mut value) };
            if variant_type == 0 {
                return Ok(None);
            }
            if variant_type != 19 && variant_type != 3 {
                return Err(format!(
                    "输入 APO FX 属性返回 VARIANT 类型 {variant_type}，预期为 32 位整数"
                ));
            }
            Ok(Some(raw))
        }
        fn set_input_fx_property(
            &self,
            endpoint: &str,
            property: &Guid,
            property_id: i32,
            use_store_context: bool,
            raw: u32,
        ) -> Result<(), String> {
            let endpoint_wide = wide_z(endpoint, "输入 endpoint ID")?;
            let mut value = Variant {
                vt: 19,
                data: u64::from(raw),
                ..Variant::default()
            };
            let ok = unsafe {
                if use_store_context && uses_fx_property_store_context() {
                    (self.set_fx_property_ex2)(
                        self.api,
                        endpoint_wide.as_ptr(),
                        &FX_PROPERTY_STORE_CONTEXT,
                        0,
                        property,
                        property_id,
                        &mut value,
                    )
                } else {
                    (self.set_fx_property_ex)(
                        self.api,
                        endpoint_wide.as_ptr(),
                        property,
                        property_id,
                        &mut value,
                    )
                }
            };
            if ok == 0 {
                return Err(format!(
                    "写入输入 APO FX 属性 GUID={:08X}-{:04X}-{:04X} ID={property_id} 失败（{}）",
                    property.d1,
                    property.d2,
                    property.d3,
                    self.last_error_context()
                ));
            }
            Ok(())
        }
        fn input_read(&self, endpoint: &str) -> Result<String, String> {
            let disabled =
                self.get_input_fx_property(endpoint, &INPUT_DISABLE_ALL_EFFECTS, 5, false)?;
            let conference =
                self.get_input_fx_property(endpoint, &INPUT_DNS_AND_PROFILE, 121, true)?;
            let dns = self.get_input_fx_property(endpoint, &INPUT_DNS_AND_PROFILE, 120, true)?;
            let enhancement = match disabled {
                Some(0) => "开启",
                Some(_) => "关闭",
                None => "未返回该属性",
            };
            let profile = match conference {
                Some(1) => "ConferenceCall",
                Some(_) => "Headset",
                None => "未返回该属性",
            };
            let dns_state = match dns {
                Some(1) => "开启",
                Some(_) => "关闭",
                None => "未返回该属性",
            };
            Ok(format!(
                "输入端点: {endpoint}\n输入音频增强: {enhancement}\n输入模式: {profile}\n降噪: {dns_state}"
            ))
        }
        fn input_set(&self, endpoint: &str, key: &str, setting: &str) -> Result<String, String> {
            let boolean = || -> Result<u32, String> {
                match setting.to_ascii_lowercase().as_str() {
                    "1" | "on" | "true" | "enable" | "enabled" => Ok(1),
                    "0" | "off" | "false" | "disable" | "disabled" => Ok(0),
                    _ => Err("输入开关值应为 0/1、on/off 或 true/false".into()),
                }
            };
            let (property, property_id, use_store_context, expected, label) = match key {
                "enhancement" => {
                    let enabled = boolean()?;
                    (
                        &INPUT_DISABLE_ALL_EFFECTS,
                        5,
                        false,
                        if enabled == 1 { 0 } else { 1 },
                        "输入音频增强",
                    )
                }
                "dns" => (&INPUT_DNS_AND_PROFILE, 120, true, boolean()?, "降噪"),
                "profile" => {
                    let profile = match setting.to_ascii_lowercase().as_str() {
                        "conference" | "conferencecall" | "conference_call" => 1,
                        "headset" | "personal" => 0,
                        _ => return Err("输入 profile 应为 headset/personal 或 conference".into()),
                    };
                    (&INPUT_DNS_AND_PROFILE, 121, true, profile, "输入模式")
                }
                _ => return Err("输入设置项应为 enhancement、dns 或 profile".into()),
            };
            self.set_input_fx_property(
                endpoint,
                property,
                property_id,
                use_store_context,
                expected,
            )?;
            match self.get_input_fx_property(endpoint, property, property_id, use_store_context) {
                Ok(Some(actual)) if actual == expected => {
                    Ok(format!("{label}写入并回读确认：{actual}"))
                }
                Ok(Some(actual)) => Err(format!(
                    "{label}写入 API 已接受，但回读为 {actual}，请求值为 {expected}"
                )),
                Ok(None) => Ok(format!(
                    "{label}写入 API 已接受，但驱动未返回该属性的回读值"
                )),
                Err(error) => Err(format!("{label}写入 API 已接受，但回读失败：{error}")),
            }
        }
    }

    fn resolve_endpoint(
        requested: Option<&str>,
        devices: &[AudioDevice],
        flow_name: &str,
    ) -> Result<String, String> {
        if let Some(requested) = requested.filter(|value| !value.is_empty() && *value != "default") {
            return devices
                .iter()
                .find(|device| device.id == requested)
                .map(|device| device.id.clone())
                .ok_or_else(|| format!("所选{flow_name}设备已不可用，请刷新设备列表"));
        }

        devices
            .iter()
            .find(|device| device.is_default)
            .map(|device| device.id.clone())
            .ok_or_else(|| format!("当前没有默认活动{flow_name}设备"))
    }

    type AudioJob = Box<dyn FnOnce() + Send + 'static>;

    static AUDIO_WORKER: OnceLock<Mutex<Option<mpsc::Sender<AudioJob>>>> = OnceLock::new();

    fn audio_worker_sender() -> Result<mpsc::Sender<AudioJob>, String> {
        let worker = AUDIO_WORKER.get_or_init(|| Mutex::new(None));
        let mut sender = worker
            .lock()
            .map_err(|_| "音频工作线程状态锁异常".to_string())?;
        if let Some(sender) = sender.as_ref() {
            return Ok(sender.clone());
        }

        let (job_sender, job_receiver) = mpsc::channel::<AudioJob>();
        std::thread::Builder::new()
            .name("nuctool-audio-com".into())
            .spawn(move || {
                while let Ok(job) = job_receiver.recv() {
                    job();
                }
            })
            .map_err(|error| format!("启动音频工作线程失败: {error}"))?;
        *sender = Some(job_sender.clone());
        Ok(job_sender)
    }

    fn run_in_audio_thread<T, F>(operation: F) -> Result<T, String>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, String> + Send + 'static,
    {
        let sender = audio_worker_sender()?;
        let (result_sender, result_receiver) = mpsc::sync_channel(1);
        sender
            .send(Box::new(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation))
                    .map_err(|_| "音频操作发生异常".to_string())
                    .and_then(|result| result);
                let _ = result_sender.send(result);
            }))
            .map_err(|_| "音频工作线程已停止".to_string())?;
        result_receiver
            .recv()
            .map_err(|_| "音频工作线程异常终止".to_string())?
    }

    pub fn list_devices() -> Result<AudioDeviceLists, String> {
        run_in_audio_thread(|| {
            Ok(AudioDeviceLists {
                outputs: unsafe { enumerate(E_RENDER)? },
                inputs: unsafe { enumerate(E_CAPTURE)? },
            })
        })
    }

    pub fn read_settings(
        output_endpoint: Option<String>,
        input_endpoint: Option<String>,
    ) -> Result<AudioTuningState, String> {
        let (output_endpoint, input_endpoint, volume) = run_in_audio_thread(move || {
            let outputs = unsafe { enumerate(E_RENDER)? };
            let output_endpoint =
                resolve_endpoint(output_endpoint.as_deref(), &outputs, "播放")?;
            let inputs = unsafe { enumerate(E_CAPTURE)? };
            let input_endpoint = if inputs.is_empty() {
                None
            } else {
                Some(resolve_endpoint(input_endpoint.as_deref(), &inputs, "录制")?)
            };
            let volume = unsafe { endpoint_volume(&output_endpoint, None)? };
            Ok((output_endpoint, input_endpoint, volume))
        })?;

        run_in_audio_thread(move || {
            read_settings_inner(output_endpoint, input_endpoint, volume)
        })
    }

    fn read_settings_inner(
        output_endpoint: String,
        input_endpoint: Option<String>,
        volume: f32,
    ) -> Result<AudioTuningState, String> {
        let (
            parameters,
            profile,
            mode,
            profiles,
            apo_error,
            input_enhancement,
            input_profile,
            input_dns,
        ) = match unsafe { Api::load(&output_endpoint) } {
                Ok(api) => {
                    let mode = api.current_mode();
                    let parameters = PARAMS
                        .iter()
                        .map(|&(id, name)| {
                            let raw = unsafe { (api.get)(api.h, id as i32) };
                            let error = unsafe { (api.get_last_error)(api.api) };
                            AudioParameter {
                                key: name.to_string(),
                                value: if error == 0 && (0.0..=1.0).contains(&raw) {
                                    Some(ui_value(id, raw))
                                } else {
                                    None
                                },
                            }
                        })
                        .collect::<Vec<_>>();
                    let profile = api.profile().unwrap_or_default();
                    let profiles = api
                        .profiles()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|name| AudioProfile {
                            alias: api.profile_alias(&name).unwrap_or_else(|_| name.clone()),
                            name,
                        })
                        .collect();
                    let (enhancement, input_profile, dns) =
                        if let Some(endpoint) = input_endpoint.as_deref() {
                            let enhancement = api
                                .get_input_fx_property(
                                    endpoint,
                                    &INPUT_DISABLE_ALL_EFFECTS,
                                    5,
                                    false,
                                )
                                .ok()
                                .flatten()
                                .map(|value| value == 0);
                            let input_profile = api
                                .get_input_fx_property(endpoint, &INPUT_DNS_AND_PROFILE, 121, true)
                                .ok()
                                .flatten()
                                .map(|value| {
                                    if value == 1 { "conference" } else { "headset" }.to_string()
                                });
                            let dns = api
                                .get_input_fx_property(endpoint, &INPUT_DNS_AND_PROFILE, 120, true)
                                .ok()
                                .flatten()
                                .map(|value| value != 0);
                            (enhancement, input_profile, dns)
                        } else {
                            (None, None, None)
                        };
                    (
                        parameters,
                        profile,
                        mode,
                        profiles,
                        None,
                        enhancement,
                        input_profile,
                        dns,
                    )
                }
                Err(error) => (
                    PARAMS
                        .iter()
                        .map(|&(_, name)| AudioParameter {
                            key: name.to_string(),
                            value: None,
                        })
                        .collect::<Vec<_>>(),
                    String::new(),
                    None,
                    Vec::new(),
                    Some(error),
                    None,
                    None,
                    None,
                ),
            };
        let current_preset = detect_current_preset(&parameters);
        Ok(AudioTuningState {
            output_endpoint,
            input_endpoint,
            volume,
            parameters,
            profile,
            mode,
            current_preset,
            profiles,
            apo_error,
            input_enhancement,
            input_profile,
            input_dns,
        })
    }

    pub fn set_parameter(endpoint: &str, key: &str, value: f32) -> Result<(), String> {
        let endpoint = endpoint.to_string();
        let key = key.to_string();
        run_in_audio_thread(move || unsafe { Api::load(&endpoint)? }.write(&key, value))
    }

    pub fn apply_preset(endpoint: &str, preset: &str) -> Result<(), String> {
        let endpoint = endpoint.to_string();
        let preset = preset.to_string();
        run_in_audio_thread(move || unsafe { Api::load(&endpoint)? }.apply_preset(&preset))
    }

    pub fn presets() -> Vec<String> {
        EQ_PRESETS.iter().map(|(name, _)| (*name).to_string()).collect()
    }

    pub fn set_mode(endpoint: &str, mode: &str) -> Result<(), String> {
        let endpoint = endpoint.to_string();
        let mode = mode.to_string();
        run_in_audio_thread(move || unsafe { Api::load(&endpoint)? }.mode(&mode))
    }

    pub fn set_profile(endpoint: &str, profile: &str) -> Result<(), String> {
        let endpoint = endpoint.to_string();
        let profile = profile.to_string();
        run_in_audio_thread(move || unsafe { Api::load(&endpoint)? }.set_profile(&profile))
    }

    pub fn create_profile(endpoint: &str, alias: &str) -> Result<String, String> {
        let endpoint = endpoint.to_string();
        let alias = alias.to_string();
        run_in_audio_thread(move || unsafe { Api::load(&endpoint)? }.create_profile(&alias))
    }

    pub fn delete_profile(endpoint: &str, profile: &str) -> Result<(), String> {
        let endpoint = endpoint.to_string();
        let profile = profile.to_string();
        run_in_audio_thread(move || unsafe { Api::load(&endpoint)? }.delete_profile(&profile))
    }

    pub fn reset_profile(endpoint: &str) -> Result<(), String> {
        let endpoint = endpoint.to_string();
        run_in_audio_thread(move || unsafe { Api::load(&endpoint)? }.reset_parameters(false))
    }

    pub fn set_volume(endpoint: &str, value: f32) -> Result<f32, String> {
        let endpoint = endpoint.to_string();
        run_in_audio_thread(move || unsafe { endpoint_volume(&endpoint, Some(value)) })
    }

    pub fn set_input_setting(
        output_endpoint: &str,
        input_endpoint: &str,
        key: &str,
        value: &str,
    ) -> Result<String, String> {
        let output_endpoint = output_endpoint.to_string();
        let input_endpoint = input_endpoint.to_string();
        let key = key.to_string();
        let value = value.to_string();
        run_in_audio_thread(move || {
            unsafe { Api::load(&output_endpoint)? }.input_set(&input_endpoint, &key, &value)
        })
    }

    pub fn worker_request<T: DeserializeOwned>(
        action: &str,
        args: serde_json::Value,
    ) -> Result<T, String> {
        let executable = env::current_exe()
            .map_err(|error| format!("获取 NUCtool 程序路径失败: {error}"))?;
        let mut child = Command::new(executable)
            .arg("--nuctool-audio-worker")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("启动音频隔离进程失败: {error}"))?;

        let request = serde_json::json!({ "action": action, "args": args });
        let child_stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "音频隔离进程 stdin 不可用".to_string())?;
        serde_json::to_writer(child_stdin, &request)
            .map_err(|error| format!("发送音频请求失败: {error}"))?;
        drop(child.stdin.take());

        let output = child
            .wait_with_output()
            .map_err(|error| format!("等待音频隔离进程失败: {error}"))?;
        let response: serde_json::Value = match serde_json::from_slice(&output.stdout) {
            Ok(response) => response,
            Err(error) if !output.status.success() => {
                let detail = String::from_utf8_lossy(&output.stderr);
                return Err(format!(
                    "音频隔离进程异常退出 ({:?}){}",
                    output.status,
                    if detail.trim().is_empty() {
                        format!("；响应无效: {error}")
                    } else {
                        format!("；{}", detail.trim())
                    }
                ));
            }
            Err(error) => return Err(format!("解析音频隔离进程响应失败: {error}")),
        };
        if response.get("ok").and_then(serde_json::Value::as_bool) != Some(true) {
            return Err(response
                .get("error")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("音频隔离进程返回未知错误")
                .to_string());
        }
        serde_json::from_value(
            response
                .get("value")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        )
        .map_err(|error| format!("解析音频隔离进程数据失败: {error}"))
    }

    pub fn run_worker() -> Result<(), String> {
        ISOLATED_AUDIO_WORKER.store(true, Ordering::Relaxed);
        let request: serde_json::Value = serde_json::from_reader(stdin().lock())
            .map_err(|error| format!("读取音频隔离请求失败: {error}"))?;
        let action = request
            .get("action")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "音频隔离请求缺少 action".to_string())?;
        let args = request
            .get("args")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        let result = (|| -> Result<serde_json::Value, String> {
            let string_arg = |name: &str| -> Result<String, String> {
                args.get(name)
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
                    .ok_or_else(|| format!("音频隔离请求缺少 {name}"))
            };
            let optional_string_arg = |name: &str| -> Result<Option<String>, String> {
                match args.get(name) {
                    None | Some(serde_json::Value::Null) => Ok(None),
                    Some(value) => value
                        .as_str()
                        .map(|value| Some(value.to_string()))
                        .ok_or_else(|| format!("音频隔离请求参数 {name} 格式错误")),
                }
            };
            let number_arg = |name: &str| -> Result<f32, String> {
                args.get(name)
                    .and_then(serde_json::Value::as_f64)
                    .map(|value| value as f32)
                    .ok_or_else(|| format!("音频隔离请求缺少数字参数 {name}"))
            };

            match action {
                "list_devices" => serde_json::to_value(list_devices()?)
                    .map_err(|error| format!("序列化设备列表失败: {error}")),
                "read_settings" => serde_json::to_value(read_settings(
                    optional_string_arg("outputEndpoint")?,
                    optional_string_arg("inputEndpoint")?,
                )?)
                .map_err(|error| format!("序列化音频设置失败: {error}")),
                "set_parameter" => {
                    set_parameter(&string_arg("endpoint")?, &string_arg("key")?, number_arg("value")?)?;
                    Ok(serde_json::Value::Null)
                }
                "apply_preset" => {
                    apply_preset(&string_arg("endpoint")?, &string_arg("preset")?)?;
                    Ok(serde_json::Value::Null)
                }
                "set_mode" => {
                    set_mode(&string_arg("endpoint")?, &string_arg("mode")?)?;
                    Ok(serde_json::Value::Null)
                }
                "set_profile" => {
                    set_profile(&string_arg("endpoint")?, &string_arg("profile")?)?;
                    Ok(serde_json::Value::Null)
                }
                "create_profile" => serde_json::to_value(create_profile(
                    &string_arg("endpoint")?,
                    &string_arg("alias")?,
                )?)
                .map_err(|error| format!("序列化配置文件结果失败: {error}")),
                "delete_profile" => {
                    delete_profile(&string_arg("endpoint")?, &string_arg("profile")?)?;
                    Ok(serde_json::Value::Null)
                }
                "reset_profile" => {
                    reset_profile(&string_arg("endpoint")?)?;
                    Ok(serde_json::Value::Null)
                }
                "set_volume" => serde_json::to_value(set_volume(
                    &string_arg("endpoint")?,
                    number_arg("value")?,
                )?)
                .map_err(|error| format!("序列化音量结果失败: {error}")),
                "set_input_setting" => serde_json::to_value(set_input_setting(
                    &string_arg("outputEndpoint")?,
                    &string_arg("inputEndpoint")?,
                    &string_arg("key")?,
                    &string_arg("value")?,
                )?)
                .map_err(|error| format!("序列化输入设置结果失败: {error}")),
                _ => Err(format!("未知音频隔离操作: {action}")),
            }
        })();

        let response = match result {
            Ok(value) => serde_json::json!({ "ok": true, "value": value }),
            Err(error) => serde_json::json!({ "ok": false, "error": error }),
        };
        serde_json::to_writer(stdout().lock(), &response)
            .map_err(|error| format!("写入音频隔离响应失败: {error}"))
    }

    fn usage() {
        println!(
            "nuc-audio-tuning source\n\
                  nuc-audio-tuning sources [render|capture]\n\
                  nuc-audio-tuning settings-default <文件.json>\n\
                  nuc-audio-tuning --endpoint <MMDevice endpoint ID|default> read\n\
                  nuc-audio-tuning --endpoint <ID|default> volume\n\
                  nuc-audio-tuning --endpoint <ID|default> volume-set <0..100>\n\
                  nuc-audio-tuning --endpoint <ID|default> probe-params [--no-proxy]\n\
                  nuc-audio-tuning --endpoint <ID|default> set <key> <value>\n\
                  nuc-audio-tuning --endpoint <ID|default> preset <name>\n\
                  nuc-audio-tuning --endpoint <ID|default> mode <AnyContent|Voice|Movie|Gaming>\n\
                  nuc-audio-tuning --endpoint <ID|default> profile\n\
                  nuc-audio-tuning --endpoint <ID|default> profiles\n\
                  nuc-audio-tuning --endpoint <ID|default> profile-set <名称|default>\n\
                  nuc-audio-tuning --endpoint <ID|default> profile-alias <名称>\n\
                  nuc-audio-tuning --endpoint <ID|default> profile-new <显示名称>\n\
                  nuc-audio-tuning --endpoint <ID|default> profile-copy <源名称> <新名称>\n\
                  nuc-audio-tuning --endpoint <ID|default> profile-delete <名称>\n\
                  nuc-audio-tuning --endpoint <ID|default> profile-alias-set <名称> <显示名称>\n\
                  nuc-audio-tuning --endpoint <ID|default> profile-reset\n\
                  nuc-audio-tuning --endpoint <ID|default> reset-all\n\
                  nuc-audio-tuning --endpoint <ID|default> profile-export <文件.json>\n\
                  nuc-audio-tuning --endpoint <ID|default> profile-import <文件.json>\n\
                  nuc-audio-tuning --endpoint <capture-ID|default> input-read\n\
                  nuc-audio-tuning --endpoint <capture-ID|default> input-set <enhancement|dns|profile> <值>\n\
                  nuc-audio-tuning list\n\n设置项：audio_enhancement, bass, detail, voice, surround, eq_enabled, eq_31hz..eq_16khz, bypass_all\n\
                  EQ 预设：default, acoustic, blues, classical, country, dance, electronic, hip_hop, jazz, metal, pop, rock\n\
                  开关使用 0/1；Bass/Detail/Voice/Surround 范围 0..100；EQ 使用驱动滑块值 -100..100（界面刻度约 -6..+6 dB）。"
        );
    }
    pub fn run() -> Result<(), String> {
        let a: Vec<String> = env::args().skip(1).collect();
        if a.first().map(String::as_str) == Some("settings-default") && a.len() == 2 {
            let defaults = "{\n  \"Enable\": false,\n  \"Input\": {\n    \"Enable\": false,\n    \"Setting\": \"Personal\"\n  },\n  \"Output\": {\n    \"EqEnable\": false,\n    \"Preset\": \"Default\",\n    \"Mode\": null,\n    \"Profile\": null\n  }\n}\n";
            std::fs::write(&a[1], defaults)
                .map_err(|error| format!("写入 NSS 音频默认设置失败：{error}"))?;
            println!("已导出 NSS AudioTuningCustomSettings 默认对象：{}", a[1]);
            println!("注意：这是应用层默认对象，不是驱动 APO 参数快照，也不会应用到音频端点。");
            return Ok(());
        }
        if a.first().map(String::as_str) == Some("source") && a.len() == 1 {
            return print_default(E_RENDER);
        }
        if a.first().map(String::as_str) == Some("sources") && a.len() <= 2 {
            let flow = match a.get(1).map(String::as_str).unwrap_or("render") {
                "render" => E_RENDER,
                "capture" => E_CAPTURE,
                _ => return Err("sources 类型应为 render 或 capture".into()),
            };
            return print_devices(flow);
        }
        if a.first().map(String::as_str) == Some("list") {
            for &(id, n) in PARAMS {
                println!("{id:>2} {n}");
            }
            return Ok(());
        }
        if a.first().map(String::as_str) == Some("presets") && a.len() == 1 {
            for (name, _) in EQ_PRESETS {
                println!("{name}");
            }
            return Ok(());
        }
        if a.len() < 3 || a[0] != "--endpoint" {
            usage();
            return Err("缺少 endpoint 参数".into());
        }
        let endpoint = if a[1].eq_ignore_ascii_case("default") {
            default_id(E_RENDER)?
        } else {
            a[1].clone()
        };
        if a[2] == "volume" && a.len() == 3 {
            let volume = unsafe { endpoint_volume(&endpoint, None)? };
            println!("端点主音量: {volume:.1}%");
            return Ok(());
        }
        if a[2] == "volume-set" && a.len() == 4 {
            let value: f32 = a[3].parse().map_err(|_| "音量必须是 0..100 的数字")?;
            let actual = unsafe { endpoint_volume(&endpoint, Some(value))? };
            println!("主音量已设置并回读: {actual:.1}%");
            return Ok(());
        }
        if a[2] == "input-read" && a.len() == 3 {
            let input_endpoint = if a[1].eq_ignore_ascii_case("default") {
                default_id(E_CAPTURE)?
            } else {
                a[1].clone()
            };
            let output_endpoint = default_id(E_RENDER)?;
            let api = unsafe { Api::load(&output_endpoint)? };
            println!("{}", api.input_read(&input_endpoint)?);
            return Ok(());
        }
        if a[2] == "input-set" && a.len() == 5 {
            let input_endpoint = if a[1].eq_ignore_ascii_case("default") {
                default_id(E_CAPTURE)?
            } else {
                a[1].clone()
            };
            let output_endpoint = default_id(E_RENDER)?;
            let api = unsafe { Api::load(&output_endpoint)? };
            println!("{}", api.input_set(&input_endpoint, &a[3], &a[4])?);
            return Ok(());
        }
        let no_proxy_apo = match (a[2].as_str(), a.len()) {
            ("probe-params", 3) => false,
            ("probe-params", 4) if a[3] == "--no-proxy" => true,
            ("probe-params", _) => {
                return Err("probe-params 可选参数仅支持 --no-proxy".into());
            }
            _ => false,
        };
        let api = unsafe { Api::load_with_proxy(&endpoint, no_proxy_apo)? };
        match a[2].as_str() {
            "probe-params" if a.len() == 3 || a.len() == 4 => {
                println!("endpoint noProxyAPO={no_proxy_apo}");
                for id in 0..22 {
                    let value = unsafe { (api.get)(api.h, id) };
                    let error = unsafe { (api.get_last_error)(api.api) };
                    println!(
                        "{id:>5}: {value:.4}  API错误 {error}: {}{}",
                        error_name(error),
                        if error == -256 {
                            let code = unsafe { (api.get_last_windows_error)(api.api) };
                            format!(" Win32={code} ({})", windows_error_name(code))
                        } else {
                            String::new()
                        }
                    );
                }
                for id in 0x8000..0x8016 {
                    let value = unsafe { (api.get)(api.h, id) };
                    let error = unsafe { (api.get_last_error)(api.api) };
                    println!(
                        "{id:>5}: {value:.4}  API错误 {error}: {}{}",
                        error_name(error),
                        if error == -256 {
                            let code = unsafe { (api.get_last_windows_error)(api.api) };
                            format!(" Win32={code} ({})", windows_error_name(code))
                        } else {
                            String::new()
                        }
                    );
                }
            }
            "read" if a.len() == 3 => {
                println!("端点ID: {endpoint}");
                print!("{}", api.read());
            }
            "set" if a.len() == 5 => {
                let value: f32 = a[4].parse().map_err(|_| "value 必须是数字")?;
                api.write(&a[3], value)?;
                println!("已写入 {}={value}", a[3]);
            }
            "preset" if a.len() == 4 => {
                api.apply_preset(&a[3])?;
                println!("已应用 EQ 预设：{}", a[3]);
            }
            "mode" if a.len() == 4 => {
                api.mode(&a[3])?;
                println!("已切换模式：{}", a[3]);
            }
            "profile" if a.len() == 3 => {
                let name = api.profile()?;
                println!(
                    "当前配置文件: {}",
                    if name.is_empty() {
                        "默认（无命名 profile）"
                    } else {
                        &name
                    }
                );
            }
            "profile-alias" if a.len() == 4 => {
                let alias = api.profile_alias(&a[3])?;
                println!("{} 的显示名称: {}", a[3], alias);
            }
            "profiles" if a.len() == 3 => {
                let names = api.profiles()?;
                if names.is_empty() {
                    println!("此端点未返回命名 profile；当前 profile 查询为空");
                }
                for name in names {
                    println!("{name}");
                }
            }
            "profile-set" if a.len() == 4 => {
                api.set_profile(&a[3])?;
                println!("已切换配置文件：{}", a[3]);
            }
            "profile-new" if a.len() == 4 => {
                let name = api.create_profile(&a[3])?;
                println!("已创建并切换到配置文件：{name}（显示名称：{}）", a[3]);
            }
            "profile-copy" if a.len() == 5 => {
                api.copy_profile(&a[3], &a[4])?;
                println!("已复制配置文件：{} -> {}", a[3], a[4]);
            }
            "profile-delete" if a.len() == 4 => {
                api.delete_profile(&a[3])?;
                println!("已删除配置文件：{}", a[3]);
            }
            "profile-alias-set" if a.len() == 5 => {
                api.set_profile_alias(&a[3], &a[4])?;
                println!("已修改配置文件显示名称：{} -> {}", a[3], a[4]);
            }
            "profile-reset" if a.len() == 3 => {
                api.reset_parameters(false)?;
                println!("已恢复当前 profile 的驱动默认参数");
            }
            "reset-all" if a.len() == 3 => {
                api.reset_parameters(true)?;
                println!("已请求恢复所有用户参数默认值");
            }
            "profile-export" if a.len() == 4 => {
                let snapshot = api.profile_snapshot_json(&endpoint)?;
                std::fs::write(&a[3], snapshot)
                    .map_err(|error| format!("写入配置快照失败：{error}"))?;
                println!("已导出当前 APO profile 快照：{}", a[3]);
            }
            "profile-import" if a.len() == 4 => {
                let snapshot = std::fs::read_to_string(&a[3])
                    .map_err(|error| format!("读取配置快照失败：{error}"))?;
                api.profile_import(&endpoint, &snapshot)?;
                println!("已恢复快照中的可写参数：{}", a[3]);
            }
            _ => {
                usage();
                return Err("命令格式错误".into());
            }
        }
        Ok(())
    }
}

fn main() {
    #[cfg(windows)]
    if let Err(e) = app::run() {
        eprintln!("错误：{e}");
        std::process::exit(2);
    }
    #[cfg(not(windows))]
    {
        eprintln!("仅支持 Windows x64；调用 Windows Sound Research APO。");
        std::process::exit(2);
    }
}
