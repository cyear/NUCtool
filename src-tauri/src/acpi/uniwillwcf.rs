use libloading::{Library, Symbol};
use std::os::raw::c_int;

type WcfConnectFn = unsafe extern "system" fn() -> c_int;
type WcfDisconnectFn = unsafe extern "system" fn();
type WcfIsConnectedFn = unsafe extern "system" fn() -> c_int;
type WcfApplyProfileFn = unsafe extern "system" fn(index: c_int) -> c_int;
type WcfSetPowerPlanFn = unsafe extern "system" fn(mode: c_int) -> c_int;
type WcfApplyBenchmarkModeFn = unsafe extern "system" fn(enable: c_int) -> c_int;
type WcfDisablePassiveCoolingFn = unsafe extern "system" fn(disable: c_int) -> c_int;
type WcfGetSupportedFeaturesCountFn = unsafe extern "system" fn() -> c_int;

// ============================================================
// Uniwill WCF interface
// ============================================================

pub struct UniwillWcfEc {
    lib: Library,

    wcf_connect: Symbol<'static, WcfConnectFn>,
    wcf_disconnect: Symbol<'static, WcfDisconnectFn>,
    wcf_is_connected: Symbol<'static, WcfIsConnectedFn>,

    wcf_apply_profile: Symbol<'static, WcfApplyProfileFn>,
    wcf_set_power_plan: Symbol<'static, WcfSetPowerPlanFn>,
    wcf_apply_benchmark_mode: Symbol<'static, WcfApplyBenchmarkModeFn>,
    wcf_disable_passive_cooling: Symbol<'static, WcfDisablePassiveCoolingFn>,

    wcf_get_supported_features_count: Symbol<'static, WcfGetSupportedFeaturesCountFn>,
}

impl UniwillWcfEc {
    // ========================================================
    // 加载 DLL
    // ========================================================

    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let exe_path = std::env::current_exe()?;
        let install_dir = exe_path.parent().unwrap();
        let dll_path = install_dir.join("NUCtool.dll");

        let lib = unsafe { Library::new(&dll_path)? };
        // Library 会被结构体持有，所以这里延长 Symbol 生命周期
        let lib_ref: &'static Library = unsafe { std::mem::transmute(&lib) };

        unsafe {
            Ok(Self {
                wcf_connect: lib_ref.get(b"wcf_connect")?,
                wcf_disconnect: lib_ref.get(b"wcf_disconnect")?,
                wcf_is_connected: lib_ref.get(b"wcf_is_connected")?,

                wcf_apply_profile: lib_ref.get(b"wcf_apply_profile")?,
                wcf_set_power_plan: lib_ref.get(b"wcf_set_power_plan")?,
                wcf_apply_benchmark_mode: lib_ref.get(b"wcf_apply_benchmark_mode")?,
                wcf_disable_passive_cooling: lib_ref.get(b"wcf_disable_passive_cooling")?,

                wcf_get_supported_features_count: lib_ref
                    .get(b"wcf_get_supported_features_count")?,

                lib,
            })
        }
    }

    // ========================================================
    // 连接
    // ========================================================

    pub fn connect(&self) -> i32 {
        unsafe { (self.wcf_connect)() }
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

    pub fn disable_passive_cooling(&self, enable: i32) -> i32 {
        unsafe { (self.wcf_disable_passive_cooling)(enable) }
    }

    // ========================================================
    // 获取支持的功能数量
    // ========================================================

    pub fn get_supported_features_count(&self) -> i32 {
        unsafe { (self.wcf_get_supported_features_count)() }
    }
}
