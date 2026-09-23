use libloading::{Library, Symbol};
use std::{
    fs,
    ffi::CString,
    os::raw::c_int
};
use sha2::{Digest, Sha256};

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

type WcfConnectFn = unsafe extern "system" fn(key: *const std::os::raw::c_char) -> c_int;
type WcfDisconnectFn = unsafe extern "system" fn();
type WcfIsConnectedFn = unsafe extern "system" fn() -> c_int;
type WcfGetCurrentStateFn = unsafe extern "system" fn(state: *mut NativePerformanceState) -> c_int;
type WcfApplyProfileFn = unsafe extern "system" fn(index: c_int) -> c_int;
type WcfSetPowerPlanFn = unsafe extern "system" fn(mode: c_int) -> c_int;
type WcfApplyBenchmarkModeFn = unsafe extern "system" fn(enable: c_int) -> c_int;
type WcfGetBatteryChargingLevelFn = unsafe extern "system" fn() -> c_int;
type WcfSetBatteryChargingLevelFn = unsafe extern "system" fn(level: c_int) -> c_int;
type WcfEnableDisplayModeMgmtFn  = unsafe extern "system" fn(enable: c_int) -> c_int;
type WcfSetDisplayModeFn         = unsafe extern "system" fn(index: c_int) -> c_int;
type WcfEnableKeyboardLedsFn     = unsafe extern "system" fn(enable: c_int) -> c_int;
type WcfGetKeyboardLedsPowerFn   = unsafe extern "system" fn() -> c_int;
type WcfSetKeyboardBrightnessFn  = unsafe extern "system" fn(brightness: c_int, ac: c_int) -> c_int;

// ============================================================
// Uniwill WCF interface
// ============================================================

pub struct UniwillWcfEc {
    lib: Library,

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
        if !hash.eq_ignore_ascii_case(EXPECTED_HASH_V2) {
            return Err(std::io::Error::other(
                "NUCtool.dll 完整性校验失败，文件可能已被修改或替换",
            ).into());
        } else {
            println!("NUCtool.dll 完整性校验成功");
        }
        let lib = unsafe { Library::new(&dll_path)? };
        // Library 会被结构体持有，所以这里延长 Symbol 生命周期
        let lib_ref: &'static Library = unsafe { std::mem::transmute(&lib) };
        unsafe {
            Ok(Self {
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
    
    pub fn wcf_get_battery_charging_level(&self) -> i32 {
        unsafe { (self.wcf_get_battery_charging_level)() }
    }

    pub fn wcf_set_battery_charging_level(&self, value: i32) -> i32 {
        unsafe { (self.wcf_set_battery_charging_level)(value) }
    }

    // ========================================================
    // 显示设置
    // ======================================================== 

    pub fn wcf_enable_display_mode_mgmt(&self, enabled: i32) -> i32 {
        unsafe { (self.wcf_enable_display_mode_mgmt)(enabled) }
    }

    pub fn wcf_set_display_mode(&self, mode: i32) -> i32 {
        unsafe { (self.wcf_set_display_mode)(mode) }
    }

    // ========================================================
    // 键盘设置
    // ======================================================== 

    pub fn wcf_enable_keyboard_leds(&self, enabled: i32) -> i32 {
        unsafe { (self.wcf_enable_keyboard_leds)(enabled) }
    }

    pub fn wcf_get_keyboard_leds_power(&self) -> i32 {
        unsafe { (self.wcf_get_keyboard_leds_power)() }
    }

    pub fn wcf_set_keyboard_brightness(&self, value: i32, ac: i32) -> i32 {
        unsafe { (self.wcf_set_keyboard_brightness)(value, ac) }
    }

}
