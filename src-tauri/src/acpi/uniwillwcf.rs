use libloading::{Library, Symbol};
use sha2::{Digest, Sha256};
use std::{ffi::CString, fs, os::raw::c_int};
use serde::{Deserialize, Serialize};

const EXPECTED_HASH_V2: &str = "838E83709F1E2C45A470868F210EEF1A3A48EAE10E13DBA896905E6DAA387497";
const KEY: &str = "NUCtool@cyear";

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NativePerformanceState {
    pub benchmark_mode: c_int,
    pub gaming_power_saver_mode: c_int,
    pub passive_cooling: c_int,
    pub selected_profile_index: c_int,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NativeRgb {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub reserved: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NativeLimitedRgb {
    pub blue: c_int,
    pub green: c_int,
    pub red: c_int,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NativeKeyboardLedProfileSingleColor {
    pub ac_color: NativeLimitedRgb,
    pub brightness_ac: c_int,
    pub brightness_dc: c_int,
    pub dc_color: NativeLimitedRgb,
    pub effect: c_int,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NativeKeyboardLedProfile {
    pub brightness_ac: c_int,
    pub brightness_dc: c_int,
    pub direction: c_int,
    pub effect: c_int,
    pub speed: c_int,
    pub color_count: c_int,
    pub key_color_count: c_int,
}

pub enum RGBKeyboardEffect {
    // 单色 √
    Monocolor = 0,
    // 呼吸
    Breathing = 1,
    // 波浪
    Wave = 2,
    // 反应
    Reactive = 3,
    // 彩虹 √
    Rainbow = 4,
    // 波纹
    Ripple = 5,
    // 雨滴
    Raindrop = 6,
    // 活动字幕
    Marquee = 7,
    // 极光
    Aurora = 8,
    // 火花
    Spark = 9,
    // 音乐
    Music = 10,
    // ???
    UserMode = 11,
    // 游戏模式
    GamingMode = 12,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct NativeLightbarSetting {
    pub blue_brightness: c_int,
    pub green_brightness: c_int,
    pub red_brightness: c_int,
    pub effect: c_int,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct NativeLightbarProfile {
    pub ac: NativeLightbarSetting,
    pub dc: NativeLightbarSetting,
    pub breathing_enable: c_int,
}

// pub enum LightBarEffect {
//     Monocolor = 0,
//     Rainbow = 1,
// }

type WcfGetLastErrorFn = unsafe extern "system" fn(
    buffer: *mut u8,
    capacity: c_int,
    requiredLength: *mut c_int,
) -> c_int;
type WcfClearLastErrorFn = unsafe extern "system" fn() -> ();
type WcfConnectFn = unsafe extern "system" fn(key: *const std::os::raw::c_char) -> c_int;
type WcfDisconnectFn = unsafe extern "system" fn();
type WcfIsConnectedFn = unsafe extern "system" fn() -> c_int;
type WcfGetCurrentStateFn = unsafe extern "system" fn(state: *mut NativePerformanceState) -> c_int;
type WcfApplyProfileFn = unsafe extern "system" fn(index: c_int) -> c_int;
type WcfSetPowerPlanFn = unsafe extern "system" fn(mode: c_int) -> c_int;
type WcfApplyBenchmarkModeFn = unsafe extern "system" fn(enable: c_int) -> c_int;
type WcfGetBatteryChargingLevelFn = unsafe extern "system" fn() -> c_int;
type WcfSetBatteryChargingLevelFn = unsafe extern "system" fn(level: c_int) -> c_int;
type WcfEnableDisplayModeMgmtFn = unsafe extern "system" fn(enable: c_int) -> c_int;
type WcfSetDisplayModeFn = unsafe extern "system" fn(index: c_int) -> c_int;
type WcfEnableKeyboardLedsFn = unsafe extern "system" fn(enable: c_int) -> c_int;
type WcfGetKeyboardLedsPowerFn = unsafe extern "system" fn() -> c_int;
type WcfSetKeyboardBrightnessFn = unsafe extern "system" fn(brightness: c_int, ac: c_int) -> c_int;
type WcfGetCurrentKeyboardLedProfileSingleColorFn = unsafe extern "system" fn(profile: *mut NativeKeyboardLedProfileSingleColor) -> c_int;
type WcfGetCurrentKeyboardLedProfileFn = unsafe extern "system" fn(profile: *mut NativeKeyboardLedProfile) -> c_int;
type WcfGetCurrentKeyboardLedProfileColorFn = unsafe extern "system" fn(index: c_int, color: *mut NativeRgb) -> c_int;
type WcfGetCurrentKeyboardLedProfileKeyColorFn = unsafe extern "system" fn(index: c_int, color: *mut NativeRgb) -> c_int;
type WcfApplyKeyboardLedProfileFn = unsafe extern "system" fn(
    brightnessAc: c_int,
    brightnessDc: c_int,
    direction: c_int,
    effect: c_int,
    speed: c_int,
    colors: *mut NativeRgb,
    colorCount: c_int,
    keyColors: *mut NativeRgb,
    keyColorCount: c_int,
) -> c_int;
type WcfGetCurrentLightbarProfileFn = unsafe extern "system" fn(profile: *mut NativeLightbarProfile) -> c_int;
type WcfSetLightbarProfileFn = unsafe extern "system" fn(profile: *mut NativeLightbarProfile) -> c_int;

// ============================================================
// Uniwill WCF interface
// ============================================================

pub struct UniwillWcfEc {
    lib: Library,
    wcf_get_last_error: Symbol<'static, WcfGetLastErrorFn>,
    wcf_clear_last_error: Symbol<'static, WcfClearLastErrorFn>,
    wcf_connect: Symbol<'static, WcfConnectFn>,
    wcf_disconnect: Symbol<'static, WcfDisconnectFn>,
    wcf_is_connected: Symbol<'static, WcfIsConnectedFn>,
    wcf_get_current_state: Symbol<'static, WcfGetCurrentStateFn>,
    wcf_apply_profile: Symbol<'static, WcfApplyProfileFn>,
    wcf_set_power_plan: Symbol<'static, WcfSetPowerPlanFn>,
    wcf_apply_benchmark_mode: Symbol<'static, WcfApplyBenchmarkModeFn>,
    // wcf_disable_passive_cooling: Symbol<'static, WcfDisablePassiveCoolingFn>,
    // wcf_get_supported_features_count: Symbol<'static, WcfGetSupportedFeaturesCountFn>,
    wcf_get_battery_charging_level: Symbol<'static, WcfGetBatteryChargingLevelFn>,
    wcf_set_battery_charging_level: Symbol<'static, WcfSetBatteryChargingLevelFn>,
    wcf_enable_display_mode_mgmt: Symbol<'static, WcfEnableDisplayModeMgmtFn>,
    wcf_set_display_mode: Symbol<'static, WcfSetDisplayModeFn>,
    wcf_enable_keyboard_leds: Symbol<'static, WcfEnableKeyboardLedsFn>,
    wcf_get_keyboard_leds_power: Symbol<'static, WcfGetKeyboardLedsPowerFn>,
    wcf_set_keyboard_brightness: Symbol<'static, WcfSetKeyboardBrightnessFn>,
    wcf_get_current_keyboard_led_profile_single_color: Symbol<'static, WcfGetCurrentKeyboardLedProfileSingleColorFn>,
    wcf_get_current_keyboard_led_profile: Symbol<'static, WcfGetCurrentKeyboardLedProfileFn>,
    wcf_get_current_keyboard_led_profile_color: Symbol<'static, WcfGetCurrentKeyboardLedProfileColorFn>,
    wcf_get_current_keyboard_led_profile_key_color: Symbol<'static, WcfGetCurrentKeyboardLedProfileKeyColorFn>,
    wcf_apply_keyboard_led_profile: Symbol<'static, WcfApplyKeyboardLedProfileFn>,
    wcf_get_current_lightbar_profile: Symbol<'static, WcfGetCurrentLightbarProfileFn>,
    wcf_set_lightbar_profile: Symbol<'static, WcfSetLightbarProfileFn>,
}

impl UniwillWcfEc {
    // ========================================================
    // 加载 DLL
    // ========================================================

    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let exe_path = std::env::current_exe()?;
        let install_dir = exe_path.parent().unwrap();
        let dll_path = install_dir.join("NUCtoolV2.dll");
        println!("DLL PATH: {:?}", &dll_path);
        let dll_data = fs::read(&dll_path)?;
        // 计算 SHA-256
        let hash = Sha256::digest(&dll_data);
        let hash = hex::encode(hash);
        println!("HASH: {}", hash);
        if !hash.eq_ignore_ascii_case(EXPECTED_HASH_V2) {
            return Err(std::io::Error::other(
                "NUCtool.dll 完整性校验失败，文件可能已被修改或替换",
            )
            .into());
        } else {
            println!("NUCtool.dll 完整性校验成功");
        }
        let lib = unsafe { Library::new(&dll_path)? };
        // Library 会被结构体持有，所以这里延长 Symbol 生命周期
        let lib_ref: &'static Library = unsafe { std::mem::transmute(&lib) };
        unsafe {
            Ok(Self {
                wcf_get_last_error: lib_ref.get(b"wcf_get_last_error")?,
                wcf_clear_last_error: lib_ref.get(b"wcf_clear_last_error")?,
                wcf_connect: lib_ref.get(b"wcf_connect")?,
                wcf_disconnect: lib_ref.get(b"wcf_disconnect")?,
                wcf_is_connected: lib_ref.get(b"wcf_is_connected")?,
                wcf_get_current_state: lib_ref.get(b"wcf_get_current_state")?,
                wcf_apply_profile: lib_ref.get(b"wcf_apply_profile")?,
                wcf_set_power_plan: lib_ref.get(b"wcf_set_power_plan")?,
                wcf_apply_benchmark_mode: lib_ref.get(b"wcf_apply_benchmark_mode")?,
                wcf_get_battery_charging_level: lib_ref.get(b"wcf_get_battery_charging_level")?,
                wcf_set_battery_charging_level: lib_ref.get(b"wcf_set_battery_charging_level")?,
                wcf_enable_display_mode_mgmt: lib_ref.get(b"wcf_enable_display_mode_mgmt")?,
                wcf_set_display_mode: lib_ref.get(b"wcf_set_display_mode")?,
                wcf_enable_keyboard_leds: lib_ref.get(b"wcf_enable_keyboard_leds")?,
                wcf_get_keyboard_leds_power: lib_ref.get(b"wcf_get_keyboard_leds_power")?,
                wcf_set_keyboard_brightness: lib_ref.get(b"wcf_set_keyboard_brightness")?,
                wcf_get_current_keyboard_led_profile_single_color: lib_ref.get(b"wcf_get_current_keyboard_led_profile_single_color")?,
                wcf_get_current_keyboard_led_profile: lib_ref.get(b"wcf_get_current_keyboard_led_profile")?,
                wcf_get_current_keyboard_led_profile_color: lib_ref.get(b"wcf_get_current_keyboard_led_profile_color")?,
                wcf_get_current_keyboard_led_profile_key_color: lib_ref.get(b"wcf_get_current_keyboard_led_profile_key_color")?,
                wcf_apply_keyboard_led_profile: lib_ref.get(b"wcf_apply_keyboard_led_profile")?,
                wcf_get_current_lightbar_profile: lib_ref.get(b"wcf_get_current_lightbar_profile")?,
                wcf_set_lightbar_profile: lib_ref.get(b"wcf_set_lightbar_profile")?,
                lib,
            })
        }
    }

    // ========================================================
    // 连接
    // ========================================================

    pub fn connect(&self) -> i32 {
        unsafe { (self.wcf_connect)(CString::new(KEY).unwrap().as_ptr()) }
    }

    pub fn disconnect(&self) {
        unsafe {
            (self.wcf_disconnect)();
        }
    }

    pub fn is_connected(&self) -> bool {
        unsafe { (self.wcf_is_connected)() != 0 }
    }

    pub fn last_error(&self) -> Result<String, Box<dyn std::error::Error>> {
        unsafe {
            let mut required: c_int = 0;
            if (self.wcf_get_last_error)(std::ptr::null_mut(), 0, &mut required) == 0 {
                return Ok(String::new());
            }

            let mut buf = vec![0u8; (required.max(0) as usize) + 1];
            let mut required2: c_int = 0;
            if (self.wcf_get_last_error)(buf.as_mut_ptr(), buf.len() as c_int, &mut required2) == 0
            {
                return Ok(String::new());
            }

            let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            Ok(String::from_utf8_lossy(&buf[..len]).into_owned())
        }
    }

    // ========================================================
    // 性能模式
    // ========================================================

    pub fn apply_profile(&self, profile: i32) -> i32 {
        unsafe { (self.wcf_apply_profile)(profile) }
    }

    pub fn get_current_state(&self) -> NativePerformanceState {
        unsafe {
            let mut state = NativePerformanceState::default();
            let _ = (self.wcf_get_current_state)(&mut state);
            return state;
        }
    }

    // ========================================================
    // 电源计划
    // ========================================================

    pub fn set_power_plan(&self, mode: i32) -> i32 {
        unsafe { (self.wcf_set_power_plan)(mode) }
    }

    // ========================================================
    // Benchmark 模式
    // ========================================================

    pub fn apply_benchmark_mode(&self, enable: i32) -> i32 {
        unsafe { (self.wcf_apply_benchmark_mode)(enable) }
    }

    // ========================================================
    // 被动散热
    // ========================================================

    // pub fn disable_passive_cooling(&self, enable: i32) -> i32 {
    //     unsafe { (self.wcf_disable_passive_cooling)(enable) }
    // }

    // ========================================================
    // 获取支持的功能数量
    // ========================================================

    // pub fn get_supported_features_count(&self) -> i32 {
    //     unsafe { (self.wcf_get_supported_features_count)() }
    // }

    // ========================================================
    // 电池设置
    // ========================================================

    pub fn battery_get_charging_level(&self) -> i32 {
        unsafe { (self.wcf_get_battery_charging_level)() }
    }

    pub fn battery_set_charging_level(&self, value: i32) -> i32 {
        unsafe { (self.wcf_set_battery_charging_level)(value) }
    }

    // ========================================================
    // 显示设置
    // ========================================================

    pub fn display_enable_mode_mgmt(&self, enabled: i32) -> i32 {
        unsafe { (self.wcf_enable_display_mode_mgmt)(enabled) }
    }

    pub fn display_set_mode(&self, mode: i32) -> i32 {
        unsafe { (self.wcf_set_display_mode)(mode) }
    }

    // ========================================================
    // 键盘设置
    // ========================================================

    pub fn keyboard_set_leds_power(&self, enabled: i32) -> i32 {
        unsafe { (self.wcf_enable_keyboard_leds)(enabled) }
    }

    pub fn keyboard_get_leds_power(&self) -> i32 {
        unsafe { (self.wcf_get_keyboard_leds_power)() }
    }

    pub fn keyboard_set_brightness(&self, value: i32, ac: i32) -> i32 {
        unsafe { (self.wcf_set_keyboard_brightness)(value, ac) }
    }

    /// NativeKeyboardLedProfileSingleColor {
    ///     ac_color: NativeLimitedRgb {
    ///         blue: 48,
    ///         green: 46,
    ///         red: 94,
    ///     },
    ///     brightness_ac: 4,
    ///     brightness_dc: 4,
    ///     dc_color: NativeLimitedRgb {
    ///         blue: 100,
    ///         green: 0,
    ///         red: 0,
    ///     },
    ///     effect: 0,
    /// }
    pub fn keyboard_get_profile_single(&self) -> NativeKeyboardLedProfileSingleColor {
        let mut single_color = NativeKeyboardLedProfileSingleColor::default();
        let ret = unsafe { (self.wcf_get_current_keyboard_led_profile_single_color)(&mut single_color) };
        if ret == 0 {
            println!("failed: {}", self.last_error().expect("keyboard_get_profile_single error"));
        }
        single_color
    }

    pub fn keyboard_get_profile(&self) -> NativeKeyboardLedProfile {
        let mut profile = NativeKeyboardLedProfile::default();
        let ret = unsafe { (self.wcf_get_current_keyboard_led_profile)(&mut profile) };
        if ret == 0 {
            println!("failed: {}", self.last_error().expect("keyboard_get_profile error"));
        }
        profile
    }

    pub fn keyboard_get_profile_key_color(&self, profile: NativeKeyboardLedProfile) {}

    // ========================================================
    // 灯条设置
    // ========================================================

    pub fn lightbar_get_profile(&self) -> NativeLightbarProfile {
        let mut profile = NativeLightbarProfile::default();
        let ret = unsafe { (self.wcf_get_current_lightbar_profile)(&mut profile) };
        if ret == 0 {
            println!("lightbar_get_profile 读取失败: {}", self.last_error().expect("lightbar_get_profile error"));
        }
        profile
    }

    pub fn lightbar_set_profile(&self, profile: NativeLightbarProfile) -> i32 {
        let mut profile = profile;
        let ret = unsafe { (self.wcf_set_lightbar_profile)(&mut profile) };
        println!("set lightbar profile ret = {ret}");

        if ret == 0 {
            println!("lightbar_set_profile 写入失败: {}", self.last_error().expect("lightbar_set_profile error"));
        }
        ret
    }

}
