#![allow(non_snake_case)]

use std::{
    ffi::{c_void, CStr},
    mem::size_of,
    path::{Path, PathBuf},
    ptr,
    sync::{Arc, Mutex, Once},
    thread,
    time::Duration,
};

use libloading::Library;
use serde::{Deserialize, Serialize};

use crate::acpi::acpi_ec_worker;

// ============================================================================
// Public SDK Models
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct ArcDeviceInfo {
    pub name: String,
    pub vendor_id: u32,
    pub device_id: u32,
    pub igcl_version: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ArcTelemetry {
    pub gpu_clock_mhz: Option<f64>,
    pub gpu_request_mhz: Option<f64>,
    pub gpu_tdp_mhz: Option<f64>,
    pub gpu_efficient_mhz: Option<f64>,
    pub freq_range_min_mhz: Option<f64>,
    pub freq_range_max_mhz: Option<f64>,
    pub gpu_voltage_v: Option<f64>,
    pub gpu_temp_c: Option<f64>,
    pub temps_c: Vec<f64>,
    pub ec_gpu_temp_c: Option<u8>,
    pub gpu_power_w: Option<f64>,
    pub system_power_w: Option<u8>,
    pub vram_clock_mhz: Option<f64>,
    pub vram_bw_mbps: Option<f64>,
    pub gpu_power_limited: Option<bool>,
    pub gpu_temp_limited: Option<bool>,
    pub throttle_mask: Option<u32>,
    pub throttle_reasons: Vec<String>,
    pub peci_readback_pl1_w: Option<u8>,
    pub peci_readback_pl2_w: Option<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArcFreqDomainInfo {
    pub index: usize,
    pub freq_type: u32,
    pub type_name: String,
    pub can_control: bool,
    pub is_throttle_supported: bool,
    pub hw_min_mhz: f64,
    pub hw_max_mhz: f64,
    pub range_min_mhz: Option<f64>,
    pub range_max_mhz: Option<f64>,
    pub range_error: Option<String>,
    pub actual_mhz: Option<f64>,
    pub request_mhz: Option<f64>,
    pub tdp_mhz: Option<f64>,
    pub efficient_mhz: Option<f64>,
    pub voltage_v: Option<f64>,
    pub throttle_mask: Option<u32>,
    pub step_count: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ArcFreqInfo {
    pub can_control: bool,
    pub hw_min_mhz: f64,
    pub hw_max_mhz: f64,
    pub range_min_mhz: Option<f64>,
    pub range_max_mhz: Option<f64>,
    pub range_error: Option<String>,
    pub available_clocks_mhz: Vec<f64>,
    pub domains: Vec<ArcFreqDomainInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArcPowerExtLimit {
    pub level: i32,
    pub source: i32,
    pub limit_unit: i32,
    pub limit: i64,
    pub interval: i64,
    pub enabled_locked: bool,
    pub interval_locked: bool,
    pub limit_locked: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArcSysmanPowerLimits {
    pub can_control: bool,
    pub sustained_enabled: bool,
    pub sustained_w: f64,
    pub sustained_tau_ms: i32,
    pub burst_enabled: bool,
    pub burst_w: f64,
    pub peak_ac_w: f64,
    pub peak_dc_w: f64,
    pub default_w: f64,
    pub min_w: f64,
    pub max_w: f64,
    pub ext_limits: Vec<ArcPowerExtLimit>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArcIgclPowerLimits {
    pub power_limit_v2_w: Option<f64>,
    pub v2_error: Option<String>,
    pub power_limit_v1_w: Option<f64>,
    pub v1_error: Option<String>,
    pub oc_power_supported: bool,
    pub oc_min: f64,
    pub oc_max: f64,
    pub oc_step: f64,
    pub oc_default: f64,
    pub freq_offset_v1_mhz: Option<f64>,
    pub freq_offset_v2_mhz: Option<f64>,
    pub gpu_lock_mv: Option<f64>,
    pub gpu_lock_mhz: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArcEcReg {
    pub address: String,
    pub value: u8,
    pub hex: String,
    pub desc: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArcEcPowerSnapshot {
    pub ctrl_0741: u8,
    pub host_takeover: bool,
    pub cpu_pl1_w: u8,
    pub host_gpu_pl1_w: u8,
    pub host_gpu_pl2_w: u8,
    pub arb_gpu_pl1_w: u8,
    pub arb_gpu_pl2_w: u8,
    pub final_peci_pl1_w: u8,
    pub final_peci_pl2_w: u8,
    pub policy_gpu_pl1_w: u8,
    pub policy_gpu_pl2_w: u8,
    pub thermal_derate_pl1_w: u8,
    pub hyst_derate_pl1_w: u8,
    pub peci_sent_pl1_w: u8,
    pub peci_readback_pl1_w: u8,
    pub peci_sent_pl2_w: u8,
    pub peci_readback_pl2_w: u8,
    pub policy_mode: u8,
    pub table_sel: u8,
    pub peci_ready: bool,
    pub regs: Vec<ArcEcReg>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ArcPowerLimits {
    pub sysman: Option<ArcSysmanPowerLimits>,
    pub igcl: Option<ArcIgclPowerLimits>,
    pub ec: Option<ArcEcPowerSnapshot>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ArcSnapshot {
    pub device: Option<ArcDeviceInfo>,
    pub telemetry: ArcTelemetry,
    pub freq: ArcFreqInfo,
    pub power_limits: ArcPowerLimits,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArcFreqWriteResult {
    pub ok: bool,
    pub code: u32,
    pub message: String,
    pub can_control: bool,
    pub hw_min_mhz: f64,
    pub hw_max_mhz: f64,
    pub readback_min_mhz: Option<f64>,
    pub readback_max_mhz: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArcPowerWriteRequest {
    #[serde(default)]
    pub write_ec: bool,
    #[serde(default)]
    pub write_sysman: bool,
    #[serde(default)]
    pub write_igcl: bool,
    pub pl1_w: u32,
    pub pl2_w: Option<u32>,
    pub tau_ms: Option<i32>,
    #[serde(default = "default_true")]
    pub ensure_ec_takeover: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize)]
pub struct ArcWriteItemResult {
    pub ok: bool,
    pub message: String,
    pub readback: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ArcPowerWriteResult {
    pub ec: Option<ArcWriteItemResult>,
    pub sysman: Option<ArcWriteItemResult>,
    pub igcl: Option<ArcWriteItemResult>,
}

// ============================================================================
// Helpers & Error Explanations
// ============================================================================

fn cstr_lossy(buf: &[u8]) -> String {
    let nul = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    CStr::from_bytes_with_nul(&[&buf[..nul], &[0]].concat())
        .ok()
        .and_then(|c| c.to_str().ok())
        .unwrap_or("")
        .trim()
        .to_string()
}

fn read_f64_le(buf: &[u8], off: usize) -> f64 {
    f64::from_le_bytes(buf[off..off + 8].try_into().unwrap())
}

fn read_u32_le(buf: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(buf[off..off + 4].try_into().unwrap())
}

fn read_i32_le(buf: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(buf[off..off + 4].try_into().unwrap())
}

fn find_driver_store_dll(dll_name: &str) -> Option<PathBuf> {
    let sys32 = PathBuf::from(r"C:\Windows\System32").join(dll_name);
    if sys32.exists() {
        return Some(sys32);
    }
    let repo = Path::new(r"C:\Windows\System32\DriverStore\FileRepository");
    let mut candidates: Vec<_> = std::fs::read_dir(repo)
        .ok()?
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("iigd_dch_d.inf_amd64_"))
        .filter_map(|e| {
            let p = e.path().join(dll_name);
            let m = std::fs::metadata(&p).ok()?;
            p.exists().then_some((m.modified().ok(), p))
        })
        .collect();
    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    candidates.into_iter().next().map(|x| x.1)
}

unsafe fn load_sym<T: Copy>(lib: &Library, name: &[u8]) -> Result<T, String> {
    let sym = lib
        .get::<T>(name)
        .map_err(|e| format!("missing symbol {}: {e}", String::from_utf8_lossy(name)))?;
    Ok(*sym)
}

pub fn explain_ze_error(code: u32) -> &'static str {
    match code {
        0x0000_0000 => "ZE_RESULT_SUCCESS",
        0x7800_0001 => "ZE_RESULT_ERROR_UNINITIALIZED",
        0x7800_0002 => "ZE_RESULT_ERROR_UNSUPPORTED_VERSION",
        0x7800_0003 => "ZE_RESULT_ERROR_UNSUPPORTED_FEATURE (驱动/PCODE 禁止修改)",
        0x7800_0004 => "ZE_RESULT_ERROR_INVALID_NULL_POINTER / INVALID_RANGE",
        0x7800_0005 => "ZE_RESULT_ERROR_INVALID_ARGUMENT",
        0x7800_0007 => "ZE_RESULT_ERROR_INSUFFICIENT_PERMISSIONS (需管理员权限)",
        _ => "OTHER_ZE_ERROR",
    }
}

pub fn explain_ctl_error(code: u32) -> &'static str {
    match code {
        0x0000_0000 => "SUCCESS",
        0x4000_0001 => "NOT_INITIALIZED",
        0x4000_0009 => "UNSUPPORTED_VERSION",
        0x4000_000d => "INVALID_OPERATION_TYPE",
        0x4000_000e => "INVALID_NULL_POINTER / NOT_FOUND",
        0x4000_000f => "INVALID_SIZE",
        0x4000_0013 => "KMD_CALL_ERROR / UNKNOWN",
        0x4000_0027 => "RESET_REQUIRED / KMD_UNSUPPORTED",
        0x4000_0028 => "UNSUPPORTED_FEATURE",
        0x4400_0001 => "OC_NOT_SUPPORTED (KMD/PCODE 锁定或不支持)",
        0x4400_0003 => "OC_INVALID_ARGUMENT",
        0x4400_0004 => "OC_INVALID_RANGE (超出 PCODE 允许范围)",
        0x4400_0006 => "OC_LOCKED",
        0x4400_0008 => "OC_WAIVER_NOT_SET",
        0x4400_0009 => "OC_UNSUPPORTED_VERSION (当前硬件/KmdOCVersion 不支持)",
        _ => "OTHER_IGCL_ERROR",
    }
}

fn decode_throttle_reasons(mask: u32) -> Vec<String> {
    const FLAGS: &[(u32, &str)] = &[
        (1 << 0, "AVE_PWR_CAP (PL1)"),
        (1 << 1, "BURST_PWR_CAP (PL2)"),
        (1 << 2, "CURRENT_LIMIT (ICCMax)"),
        (1 << 3, "THERMAL_LIMIT"),
        (1 << 4, "PSU_ALERT"),
        (1 << 5, "SW_RANGE"),
        (1 << 6, "HW_RANGE"),
    ];
    let mut reasons = Vec::new();
    let mut rem = mask;
    for &(bit, label) in FLAGS {
        if (mask & bit) != 0 {
            reasons.push(label.to_string());
            rem &= !bit;
        }
    }
    if rem != 0 {
        reasons.push(format!("OTHER({rem:#x})"));
    }
    reasons
}

fn freq_type_name(t: u32) -> &'static str {
    match t {
        0 => "GPU",
        1 => "VRAM",
        2 => "Media",
        _ => "Other",
    }
}

// ============================================================================
// Level Zero Sysman (ze_loader.dll)
// ============================================================================

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct ZesPowerSustainedLimit {
    enabled: i32,
    power_mw: i32,
    interval_ms: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct ZesPowerBurstLimit {
    enabled: i32,
    power_mw: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct ZesPowerPeakLimit {
    power_ac_mw: i32,
    power_dc_mw: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct ZesPowerEnergyCounter {
    energy_uj: u64,
    timestamp_us: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct ZesFreqRange {
    min: f64,
    max: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ZesPowerLimitExtDesc {
    stype: u32,
    p_next: *mut c_void,
    level: i32,
    source: i32,
    limit_unit: i32,
    enabled_state_locked: u8,
    interval_value_locked: u8,
    limit_value_locked: u8,
    _pad0: u8,
    interval: i64,
    limit: i64,
}

impl Default for ZesPowerLimitExtDesc {
    fn default() -> Self {
        Self {
            stype: 0x1002b,
            p_next: ptr::null_mut(),
            level: 0,
            source: 0,
            limit_unit: 0,
            enabled_state_locked: 0,
            interval_value_locked: 0,
            limit_value_locked: 0,
            _pad0: 0,
            interval: 0,
            limit: 0,
        }
    }
}

type FnZeInit = unsafe extern "C" fn(u32) -> u32;
type FnZeDriverGet = unsafe extern "C" fn(*mut u32, *mut *mut c_void) -> u32;
type FnZeDeviceGet = unsafe extern "C" fn(*mut c_void, *mut u32, *mut *mut c_void) -> u32;
type FnZeDeviceGetProps = unsafe extern "C" fn(*mut c_void, *mut u8) -> u32;
type FnZesEnumHandles = unsafe extern "C" fn(*mut c_void, *mut u32, *mut *mut c_void) -> u32;
type FnZesPowerGetProps = unsafe extern "C" fn(*mut c_void, *mut u8) -> u32;
type FnZesPowerGetLimits = unsafe extern "C" fn(
    *mut c_void,
    *mut ZesPowerSustainedLimit,
    *mut ZesPowerBurstLimit,
    *mut ZesPowerPeakLimit,
) -> u32;
type FnZesPowerSetLimits = unsafe extern "C" fn(
    *mut c_void,
    *const ZesPowerSustainedLimit,
    *const ZesPowerBurstLimit,
    *const ZesPowerPeakLimit,
) -> u32;
type FnZesPowerGetLimitsExt =
    unsafe extern "C" fn(*mut c_void, *mut u32, *mut ZesPowerLimitExtDesc) -> u32;
type FnZesPowerSetLimitsExt =
    unsafe extern "C" fn(*mut c_void, *mut u32, *mut ZesPowerLimitExtDesc) -> u32;
type FnZesPowerEnergy = unsafe extern "C" fn(*mut c_void, *mut ZesPowerEnergyCounter) -> u32;
type FnZesFreqProps = unsafe extern "C" fn(*mut c_void, *mut u8) -> u32;
type FnZesFreqGetRange = unsafe extern "C" fn(*mut c_void, *mut ZesFreqRange) -> u32;
type FnZesFreqSetRange = unsafe extern "C" fn(*mut c_void, *const ZesFreqRange) -> u32;
type FnZesFreqAvailClocks = unsafe extern "C" fn(*mut c_void, *mut u32, *mut f64) -> u32;
type FnZesFreqState = unsafe extern "C" fn(*mut c_void, *mut u8) -> u32;
type FnZesTempState = unsafe extern "C" fn(*mut c_void, *mut f64) -> u32;

struct SysmanDevice {
    name: String,
    vendor_id: u32,
    device_id: u32,
    power_domains: Vec<*mut c_void>,
    freq_domains: Vec<*mut c_void>,
    temp_sensors: Vec<*mut c_void>,
}

struct SysmanSession {
    _lib: Library,
    devices: Vec<SysmanDevice>,
    arc_idx: usize,
    fn_power_props: FnZesPowerGetProps,
    fn_power_get: FnZesPowerGetLimits,
    fn_power_set: FnZesPowerSetLimits,
    fn_power_get_ext: FnZesPowerGetLimitsExt,
    fn_power_set_ext: FnZesPowerSetLimitsExt,
    fn_power_energy: FnZesPowerEnergy,
    fn_freq_props: FnZesFreqProps,
    fn_freq_get_range: FnZesFreqGetRange,
    fn_freq_set_range: FnZesFreqSetRange,
    fn_freq_avail_clocks: FnZesFreqAvailClocks,
    fn_freq_state: FnZesFreqState,
    fn_temp_state: FnZesTempState,
}

unsafe impl Send for SysmanSession {}

impl SysmanSession {
    fn open() -> Result<Self, String> {
        static ENV_ONCE: Once = Once::new();
        ENV_ONCE.call_once(|| std::env::set_var("ZES_ENABLE_SYSMAN", "1"));

        let sys32 = Path::new(r"C:\Windows\System32\ze_loader.dll");
        let lib = unsafe {
            if sys32.exists() {
                Library::new(sys32)
            } else {
                Library::new("ze_loader.dll")
            }
        }
        .map_err(|e| format!("加载 ze_loader.dll 失败: {e}"))?;

        unsafe {
            let fn_init: FnZeInit = load_sym(&lib, b"zeInit\0")?;
            let fn_drv_get: FnZeDriverGet = load_sym(&lib, b"zeDriverGet\0")?;
            let fn_dev_get: FnZeDeviceGet = load_sym(&lib, b"zeDeviceGet\0")?;
            let fn_dev_props: FnZeDeviceGetProps = load_sym(&lib, b"zeDeviceGetProperties\0")?;
            let fn_enum_pwr: FnZesEnumHandles = load_sym(&lib, b"zesDeviceEnumPowerDomains\0")?;
            let fn_enum_freq: FnZesEnumHandles = load_sym(&lib, b"zesDeviceEnumFrequencyDomains\0")?;
            let fn_enum_temp: FnZesEnumHandles = load_sym(&lib, b"zesDeviceEnumTemperatureSensors\0")?;
            let fn_power_props: FnZesPowerGetProps = load_sym(&lib, b"zesPowerGetProperties\0")?;
            let fn_power_get: FnZesPowerGetLimits = load_sym(&lib, b"zesPowerGetLimits\0")?;
            let fn_power_set: FnZesPowerSetLimits = load_sym(&lib, b"zesPowerSetLimits\0")?;
            let fn_power_get_ext: FnZesPowerGetLimitsExt = load_sym(&lib, b"zesPowerGetLimitsExt\0")?;
            let fn_power_set_ext: FnZesPowerSetLimitsExt = load_sym(&lib, b"zesPowerSetLimitsExt\0")?;
            let fn_power_energy: FnZesPowerEnergy = load_sym(&lib, b"zesPowerGetEnergyCounter\0")?;
            let fn_freq_props: FnZesFreqProps = load_sym(&lib, b"zesFrequencyGetProperties\0")?;
            let fn_freq_get_range: FnZesFreqGetRange = load_sym(&lib, b"zesFrequencyGetRange\0")?;
            let fn_freq_set_range: FnZesFreqSetRange = load_sym(&lib, b"zesFrequencySetRange\0")?;
            let fn_freq_avail_clocks: FnZesFreqAvailClocks =
                load_sym(&lib, b"zesFrequencyGetAvailableClocks\0")?;
            let fn_freq_state: FnZesFreqState = load_sym(&lib, b"zesFrequencyGetState\0")?;
            let fn_temp_state: FnZesTempState = load_sym(&lib, b"zesTemperatureGetState\0")?;

            let r_init = fn_init(0);
            if r_init != 0 {
                return Err(format!("zeInit(0) failed: {r_init:#010x} ({})", explain_ze_error(r_init)));
            }

            let mut dcnt = 0u32;
            fn_drv_get(&mut dcnt, ptr::null_mut());
            let mut drivers = vec![ptr::null_mut(); dcnt as usize];
            if dcnt > 0 {
                fn_drv_get(&mut dcnt, drivers.as_mut_ptr());
            }

            let mut devices = Vec::new();
            for drv in drivers {
                let mut cnt = 0u32;
                fn_dev_get(drv, &mut cnt, ptr::null_mut());
                let mut devs = vec![ptr::null_mut(); cnt as usize];
                if cnt > 0 {
                    fn_dev_get(drv, &mut cnt, devs.as_mut_ptr());
                }
                for dev in devs {
                    let mut props = [0u8; 512];
                    props[0..4].copy_from_slice(&3u32.to_le_bytes());
                    let _ = fn_dev_props(dev, props.as_mut_ptr());
                    let vendor_id = read_u32_le(&props, 20);
                    let device_id = read_u32_le(&props, 24);
                    let name = cstr_lossy(&props[112..368]);

                    let enum_handles = |f: FnZesEnumHandles| -> Vec<*mut c_void> {
                        let mut n = 0u32;
                        let _ = f(dev, &mut n, ptr::null_mut());
                        let mut v = vec![ptr::null_mut(); n as usize];
                        if n > 0 {
                            let _ = f(dev, &mut n, v.as_mut_ptr());
                        }
                        v
                    };

                    devices.push(SysmanDevice {
                        name,
                        vendor_id,
                        device_id,
                        power_domains: enum_handles(fn_enum_pwr),
                        freq_domains: enum_handles(fn_enum_freq),
                        temp_sensors: enum_handles(fn_enum_temp),
                    });
                }
            }

            let arc_idx = devices
                .iter()
                .position(|d| {
                    !d.power_domains.is_empty()
                        && (matches!(d.device_id, 0x5691 | 0x5692) || d.name.contains("Arc"))
                })
                .or_else(|| devices.iter().position(|d| !d.power_domains.is_empty()))
                .ok_or_else(|| "未找到暴露 Sysman PowerDomain 的 Arc 独显".to_string())?;

            Ok(Self {
                _lib: lib,
                devices,
                arc_idx,
                fn_power_props,
                fn_power_get,
                fn_power_set,
                fn_power_get_ext,
                fn_power_set_ext,
                fn_power_energy,
                fn_freq_props,
                fn_freq_get_range,
                fn_freq_set_range,
                fn_freq_avail_clocks,
                fn_freq_state,
                fn_temp_state,
            })
        }
    }

    fn device(&self) -> &SysmanDevice {
        &self.devices[self.arc_idx]
    }

    fn read_energy(&self) -> Option<(u64, u64)> {
        let pdom = *self.device().power_domains.first()?;
        let mut energy = ZesPowerEnergyCounter::default();
        (unsafe { (self.fn_power_energy)(pdom, &mut energy) } == 0)
            .then_some((energy.energy_uj, energy.timestamp_us))
    }

    fn read_temps(&self) -> Vec<f64> {
        self.device()
            .temp_sensors
            .iter()
            .filter_map(|&ts| {
                let mut t = 0.0f64;
                (unsafe { (self.fn_temp_state)(ts, &mut t) } == 0).then_some(t)
            })
            .collect()
    }

    fn read_freq_domains(&self, include_clocks: bool) -> (Vec<ArcFreqDomainInfo>, Vec<f64>) {
        let mut out = Vec::with_capacity(self.device().freq_domains.len());
        let mut gpu_clocks = Vec::new();
        for (idx, &fdom) in self.device().freq_domains.iter().enumerate() {
            let mut props = [0u8; 64];
            props[0..4].copy_from_slice(&0x10006u32.to_le_bytes());
            let r_props = unsafe { (self.fn_freq_props)(fdom, props.as_mut_ptr()) };
            let (freq_type, can_control, is_throttle_supported, hw_min_mhz, hw_max_mhz) =
                if r_props == 0 {
                    (
                        read_u32_le(&props, 16),
                        props[28] != 0,
                        props[29] != 0,
                        read_f64_le(&props, 32),
                        read_f64_le(&props, 40),
                    )
                } else {
                    (0, false, false, 0.0, 0.0)
                };

            let mut range = ZesFreqRange::default();
            let r_rng = unsafe { (self.fn_freq_get_range)(fdom, &mut range) };
            let (range_min_mhz, range_max_mhz, range_error) = if r_rng == 0 {
                (Some(range.min), Some(range.max), None)
            } else {
                (None, None, Some(format!("{r_rng:#010x} ({})", explain_ze_error(r_rng))))
            };

            let mut cnt = 0u32;
            let mut avail = Vec::new();
            if (include_clocks || freq_type == 0)
                && unsafe { (self.fn_freq_avail_clocks)(fdom, &mut cnt, ptr::null_mut()) } == 0
                && cnt > 0
            {
                if include_clocks && freq_type == 0 {
                    avail.resize(cnt as usize, 0.0);
                    let _ = unsafe { (self.fn_freq_avail_clocks)(fdom, &mut cnt, avail.as_mut_ptr()) };
                }
            }
            let step_count = if !avail.is_empty() { avail.len() } else { cnt as usize };
            if freq_type == 0 && !avail.is_empty() {
                gpu_clocks = avail;
            }

            let mut fs = [0u8; 64];
            fs[0..4].copy_from_slice(&0x10007u32.to_le_bytes());
            let r_st = unsafe { (self.fn_freq_state)(fdom, fs.as_mut_ptr()) };
            let (voltage_v, request_mhz, tdp_mhz, efficient_mhz, actual_mhz, throttle_mask) =
                if r_st == 0 {
                    let v = read_f64_le(&fs, 16);
                    let req = read_f64_le(&fs, 24);
                    let tdp = read_f64_le(&fs, 32);
                    let eff = read_f64_le(&fs, 40);
                    let act = read_f64_le(&fs, 48);
                    let thr = read_u32_le(&fs, 56);
                    (
                        (v > 0.0).then_some(v),
                        (req >= 0.0).then_some(req),
                        (tdp > 0.0).then_some(tdp),
                        (eff >= 0.0).then_some(eff),
                        (act >= 0.0).then_some(act),
                        Some(thr),
                    )
                } else {
                    (None, None, None, None, None, None)
                };

            out.push(ArcFreqDomainInfo {
                index: idx,
                freq_type,
                type_name: freq_type_name(freq_type).to_string(),
                can_control,
                is_throttle_supported,
                hw_min_mhz,
                hw_max_mhz,
                range_min_mhz,
                range_max_mhz,
                range_error,
                actual_mhz,
                request_mhz,
                tdp_mhz,
                efficient_mhz,
                voltage_v,
                throttle_mask,
                step_count,
            });
        }
        (out, gpu_clocks)
    }

    fn read_power_limits(&self) -> Result<ArcSysmanPowerLimits, String> {
        let pdom = *self
            .device()
            .power_domains
            .first()
            .ok_or_else(|| "无 Sysman PowerDomain".to_string())?;
        let mut sust = ZesPowerSustainedLimit::default();
        let mut burst = ZesPowerBurstLimit::default();
        let mut peak = ZesPowerPeakLimit::default();
        let r = unsafe { (self.fn_power_get)(pdom, &mut sust, &mut burst, &mut peak) };
        if r != 0 {
            return Err(format!("zesPowerGetLimits failed: {r:#010x} ({})", explain_ze_error(r)));
        }

        let mut cnt = 0u32;
        let mut ext_raw = Vec::new();
        if unsafe { (self.fn_power_get_ext)(pdom, &mut cnt, ptr::null_mut()) } == 0 && cnt > 0 {
            ext_raw.resize(cnt as usize, ZesPowerLimitExtDesc::default());
            let _ = unsafe { (self.fn_power_get_ext)(pdom, &mut cnt, ext_raw.as_mut_ptr()) };
        }

        let mut pprops = [0u8; 64];
        pprops[0..4].copy_from_slice(&0x10018u32.to_le_bytes());
        let _ = unsafe { (self.fn_power_props)(pdom, pprops.as_mut_ptr()) };

        Ok(ArcSysmanPowerLimits {
            can_control: pprops[24] != 0,
            sustained_enabled: sust.enabled != 0,
            sustained_w: f64::from(sust.power_mw) / 1000.0,
            sustained_tau_ms: sust.interval_ms,
            burst_enabled: burst.enabled != 0,
            burst_w: f64::from(burst.power_mw) / 1000.0,
            peak_ac_w: f64::from(peak.power_ac_mw) / 1000.0,
            peak_dc_w: f64::from(peak.power_dc_mw) / 1000.0,
            default_w: f64::from(read_i32_le(&pprops, 28)) / 1000.0,
            min_w: f64::from(read_i32_le(&pprops, 32)) / 1000.0,
            max_w: f64::from(read_i32_le(&pprops, 36)) / 1000.0,
            ext_limits: ext_raw
                .into_iter()
                .map(|e| ArcPowerExtLimit {
                    level: e.level,
                    source: e.source,
                    limit_unit: e.limit_unit,
                    limit: e.limit,
                    interval: e.interval,
                    enabled_locked: e.enabled_state_locked != 0,
                    interval_locked: e.interval_value_locked != 0,
                    limit_locked: e.limit_value_locked != 0,
                })
                .collect(),
        })
    }

    fn set_frequency_range(&self, min_mhz: f64, max_mhz: f64, is_reset: bool) -> Result<ArcFreqWriteResult, String> {
        let (domains, _) = self.read_freq_domains(false);
        let gpu_dom = domains
            .iter()
            .find(|d| d.freq_type == 0)
            .or_else(|| domains.first())
            .ok_or_else(|| "该设备没有 Sysman FrequencyDomain".to_string())?;
        let fdom = self.device().freq_domains[gpu_dom.index];
        let (can_control, hw_min, hw_max) = (gpu_dom.can_control, gpu_dom.hw_min_mhz, gpu_dom.hw_max_mhz);

        let (req_min, req_max) = if is_reset {
            if hw_max > hw_min {
                (hw_min, hw_max)
            } else {
                (-1.0, -1.0)
            }
        } else {
            if !min_mhz.is_finite() || !max_mhz.is_finite() || min_mhz <= 0.0 || max_mhz <= 0.0 {
                return Err("频率必须为大于 0 的有效数值".to_string());
            }
            if min_mhz > max_mhz {
                return Err(format!("最低频率 {min_mhz:.0} MHz 不能大于最高频率 {max_mhz:.0} MHz"));
            }
            if hw_max > hw_min && (min_mhz < hw_min || max_mhz > hw_max) {
                return Err(format!(
                    "目标范围 [{min_mhz:.0}, {max_mhz:.0}] MHz 超出硬件区间 [{hw_min:.0}, {hw_max:.0}] MHz"
                ));
            }
            (min_mhz, max_mhz)
        };

        let req = ZesFreqRange { min: req_min, max: req_max };
        let res = unsafe { (self.fn_freq_set_range)(fdom, &req) };

        let mut rb = ZesFreqRange::default();
        let (rb_min, rb_max) = if unsafe { (self.fn_freq_get_range)(fdom, &mut rb) } == 0 {
            (Some(rb.min), Some(rb.max))
        } else {
            (None, None)
        };

        Ok(ArcFreqWriteResult {
            ok: res == 0,
            code: res,
            message: format!("{res:#010x} ({})", explain_ze_error(res)),
            can_control,
            hw_min_mhz: hw_min,
            hw_max_mhz: hw_max,
            readback_min_mhz: rb_min,
            readback_max_mhz: rb_max,
        })
    }

    fn set_power_limits(&self, pl1_w: u32, pl2_w: u32, tau_ms: i32) -> ArcWriteItemResult {
        let Some(&pdom) = self.device().power_domains.first() else {
            return ArcWriteItemResult {
                ok: false,
                message: "无 Sysman PowerDomain".to_string(),
                readback: None,
            };
        };
        let mw1 = (pl1_w as i32).saturating_mul(1000);
        let mw2 = (pl2_w as i32).saturating_mul(1000);
        let sust = ZesPowerSustainedLimit { enabled: 1, power_mw: mw1, interval_ms: tau_ms };
        let burst = ZesPowerBurstLimit { enabled: 1, power_mw: mw2 };
        let peak = ZesPowerPeakLimit { power_ac_mw: mw2, power_dc_mw: mw2 };
        let rb = unsafe { (self.fn_power_set)(pdom, &sust, &burst, &peak) };

        let mut cnt = 3u32;
        let mut descs = [
            ZesPowerLimitExtDesc {
                level: 1,
                source: 0,
                limit_unit: 2,
                interval: i64::from(tau_ms),
                limit: i64::from(mw1),
                ..Default::default()
            },
            ZesPowerLimitExtDesc {
                level: 2,
                source: 0,
                limit_unit: 2,
                interval: 0,
                limit: i64::from(mw2),
                ..Default::default()
            },
            ZesPowerLimitExtDesc {
                level: 3,
                source: 0,
                limit_unit: 2,
                interval: 0,
                limit: i64::from(mw2),
                ..Default::default()
            },
        ];
        let re = unsafe { (self.fn_power_set_ext)(pdom, &mut cnt, descs.as_mut_ptr()) };
        let readback = self.read_power_limits().ok().map(|s| {
            format!(
                "PL1={:.1}W (tau={}ms), PL2={:.1}W, Peak={:.1}W",
                s.sustained_w, s.sustained_tau_ms, s.burst_w, s.peak_ac_w
            )
        });
        ArcWriteItemResult {
            ok: rb == 0 || re == 0,
            message: format!(
                "SetLimits={rb:#010x} ({}), SetLimitsExt={re:#010x} ({})",
                explain_ze_error(rb),
                explain_ze_error(re)
            ),
            readback,
        }
    }
}

// ============================================================================
// Intel Graphics Control Library (IntelControlLib.dll)
// ============================================================================

#[repr(C)]
#[derive(Clone, Copy)]
struct CtlInitArgs {
    size: u32,
    version: u8,
    app_version: u32,
    flags: u32,
    supported_version: u32,
    application_uid: [u8; 16],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct CtlOcVfPair {
    size: u32,
    version: u8,
    _pad: [u8; 3],
    voltage: f64,
    frequency: f64,
}

type FnCtlInit = unsafe extern "C" fn(*mut CtlInitArgs, *mut *mut c_void) -> u32;
type FnCtlClose = unsafe extern "C" fn(*mut c_void) -> u32;
type FnCtlEnumDevs = unsafe extern "C" fn(*mut c_void, *mut u32, *mut *mut c_void) -> u32;
type FnCtlOcProps = unsafe extern "C" fn(*mut c_void, *mut u8) -> u32;
type FnCtlOcF64Get = unsafe extern "C" fn(*mut c_void, *mut f64) -> u32;
type FnCtlOcF64Set = unsafe extern "C" fn(*mut c_void, f64) -> u32;
type FnCtlOcWaiverSet = unsafe extern "C" fn(*mut c_void) -> u32;
type FnCtlOcGpuLockGet = unsafe extern "C" fn(*mut c_void, *mut CtlOcVfPair) -> u32;
type FnCtlTelemetryV2 = unsafe extern "C" fn(*mut c_void, *mut u8) -> u32;

struct IgclTelemetry {
    gpu_voltage_v: Option<f64>,
    gpu_clock_mhz: Option<f64>,
    gpu_temp_c: Option<f64>,
    vram_clock_mhz: Option<f64>,
    vram_bw_mbps: Option<f64>,
    gpu_power_limited: Option<bool>,
    gpu_temp_limited: Option<bool>,
}

struct IgclSession {
    _lib: Library,
    api_handle: *mut c_void,
    devices: Vec<*mut c_void>,
    dev_idx: usize,
    supported_version: u32,
    fn_close: FnCtlClose,
    fn_oc_props: FnCtlOcProps,
    fn_oc_power_get: FnCtlOcF64Get,
    fn_oc_power_get_v2: FnCtlOcF64Get,
    fn_oc_power_set: FnCtlOcF64Set,
    fn_oc_power_set_v2: FnCtlOcF64Set,
    fn_oc_waiver_set: FnCtlOcWaiverSet,
    fn_oc_freq_offset_get: Option<FnCtlOcF64Get>,
    fn_oc_freq_offset_get_v2: Option<FnCtlOcF64Get>,
    fn_oc_gpu_lock_get: Option<FnCtlOcGpuLockGet>,
    fn_telemetry_v2: FnCtlTelemetryV2,
}

unsafe impl Send for IgclSession {}

impl Drop for IgclSession {
    fn drop(&mut self) {
        if !self.api_handle.is_null() {
            unsafe {
                let _ = (self.fn_close)(self.api_handle);
            }
        }
    }
}

impl IgclSession {
    fn open() -> Result<Self, String> {
        let path = find_driver_store_dll("IntelControlLib.dll")
            .ok_or_else(|| "未找到 IntelControlLib.dll".to_string())?;
        let lib = unsafe { Library::new(&path) }
            .map_err(|e| format!("加载 IntelControlLib.dll 失败: {e}"))?;

        unsafe {
            let fn_init: FnCtlInit = load_sym(&lib, b"ctlInit\0")?;
            let fn_close: FnCtlClose = load_sym(&lib, b"ctlClose\0")?;
            let fn_enum: FnCtlEnumDevs = load_sym(&lib, b"ctlEnumerateDevices\0")?;
            let fn_oc_props: FnCtlOcProps = load_sym(&lib, b"ctlOverclockGetProperties\0")?;
            let fn_oc_power_get: FnCtlOcF64Get = load_sym(&lib, b"ctlOverclockPowerLimitGet\0")?;
            let fn_oc_power_get_v2: FnCtlOcF64Get = load_sym(&lib, b"ctlOverclockPowerLimitGetV2\0")?;
            let fn_oc_power_set: FnCtlOcF64Set = load_sym(&lib, b"ctlOverclockPowerLimitSet\0")?;
            let fn_oc_power_set_v2: FnCtlOcF64Set = load_sym(&lib, b"ctlOverclockPowerLimitSetV2\0")?;
            let fn_oc_waiver_set: FnCtlOcWaiverSet = load_sym(&lib, b"ctlOverclockWaiverSet\0")?;
            let fn_oc_freq_offset_get = load_sym(&lib, b"ctlOverclockGpuFrequencyOffsetGet\0").ok();
            let fn_oc_freq_offset_get_v2 = load_sym(&lib, b"ctlOverclockGpuFrequencyOffsetGetV2\0").ok();
            let fn_oc_gpu_lock_get = load_sym(&lib, b"ctlOverclockGpuLockGet\0").ok();
            let fn_telemetry_v2: FnCtlTelemetryV2 = load_sym(&lib, b"ctlPowerTelemetryGetV2\0")?;

            let mut args = CtlInitArgs {
                size: size_of::<CtlInitArgs>() as u32,
                version: 0,
                app_version: (1 << 16) | 1,
                flags: 0,
                supported_version: 0,
                application_uid: [0u8; 16],
            };
            let mut api_handle = ptr::null_mut();
            let res = fn_init(&mut args, &mut api_handle);
            if res != 0 {
                return Err(format!("ctlInit failed: {res:#010x} ({})", explain_ctl_error(res)));
            }

            let mut count = 0u32;
            let r_cnt = fn_enum(api_handle, &mut count, ptr::null_mut());
            if r_cnt != 0 {
                let _ = fn_close(api_handle);
                return Err(format!("ctlEnumerateDevices failed: {r_cnt:#010x}"));
            }
            let mut devices = vec![ptr::null_mut(); count as usize];
            if count > 0 {
                let r_dev = fn_enum(api_handle, &mut count, devices.as_mut_ptr());
                if r_dev != 0 {
                    let _ = fn_close(api_handle);
                    return Err(format!("ctlEnumerateDevices(list) failed: {r_dev:#010x}"));
                }
            }
            if devices.is_empty() {
                let _ = fn_close(api_handle);
                return Err("IGCL 未枚举到显示设备".to_string());
            }

            let mut sess = Self {
                _lib: lib,
                api_handle,
                devices,
                dev_idx: 0,
                supported_version: args.supported_version,
                fn_close,
                fn_oc_props,
                fn_oc_power_get,
                fn_oc_power_get_v2,
                fn_oc_power_set,
                fn_oc_power_set_v2,
                fn_oc_waiver_set,
                fn_oc_freq_offset_get,
                fn_oc_freq_offset_get_v2,
                fn_oc_gpu_lock_get,
                fn_telemetry_v2,
            };
            // Prefer discrete GPU device exposing VRAM telemetry.
            if let Some(idx) = (0..sess.devices.len()).find(|&i| {
                sess.read_telemetry_idx(i)
                    .and_then(|t| t.vram_clock_mhz)
                    .is_some()
            }) {
                sess.dev_idx = idx;
            }
            Ok(sess)
        }
    }

    fn read_telemetry_idx(&self, idx: usize) -> Option<IgclTelemetry> {
        let dev = *self.devices.get(idx)?;
        let mut tel = [0u8; 512];
        tel[0..4].copy_from_slice(&512u32.to_le_bytes());
        tel[4] = 2;
        if unsafe { (self.fn_telemetry_v2)(dev, tel.as_mut_ptr()) } != 0 {
            return None;
        }
        let f64_at = |off: usize| (tel[off] != 0).then(|| read_f64_le(&tel, off + 16));
        let bool_at = |off: usize| (tel[off] != 0).then(|| tel[off + 16] != 0);
        Some(IgclTelemetry {
            gpu_voltage_v: f64_at(56),
            gpu_clock_mhz: f64_at(80),
            gpu_temp_c: f64_at(104),
            gpu_power_limited: bool_at(200),
            gpu_temp_limited: bool_at(224),
            vram_clock_mhz: f64_at(256),
            vram_bw_mbps: f64_at(280),
        })
    }

    fn read_telemetry(&self) -> Option<IgclTelemetry> {
        self.read_telemetry_idx(self.dev_idx)
    }

    fn read_power_limits(&self) -> ArcIgclPowerLimits {
        let dev = self.devices[self.dev_idx];
        let mut w = 0.0f64;
        let r_w = unsafe { (self.fn_oc_power_get_v2)(dev, &mut w) };
        let mut mw = 0.0f64;
        let r_mw = unsafe { (self.fn_oc_power_get)(dev, &mut mw) };

        let mut oc_buf = [0u8; 440];
        oc_buf[0..4].copy_from_slice(&440u32.to_le_bytes());
        oc_buf[4] = 1;
        let r_oc = unsafe { (self.fn_oc_props)(dev, oc_buf.as_mut_ptr()) };
        let (oc_power_supported, oc_min, oc_max, oc_step, oc_default) =
            if r_oc == 0 && oc_buf[8..].iter().any(|&b| b != 0) {
                (
                    oc_buf[136] != 0,
                    read_f64_le(&oc_buf, 144),
                    read_f64_le(&oc_buf, 152),
                    read_f64_le(&oc_buf, 160),
                    read_f64_le(&oc_buf, 168),
                )
            } else {
                (false, 0.0, 0.0, 0.0, 0.0)
            };

        let get_opt_f64 = |f: Option<FnCtlOcF64Get>| -> Option<f64> {
            let mut v = 0.0f64;
            (unsafe { f?(dev, &mut v) } == 0).then_some(v)
        };
        let gpu_lock = self.fn_oc_gpu_lock_get.and_then(|f| {
            let mut pair = CtlOcVfPair {
                size: size_of::<CtlOcVfPair>() as u32,
                version: 0,
                ..Default::default()
            };
            (unsafe { f(dev, &mut pair) } == 0).then_some((pair.voltage, pair.frequency))
        });

        ArcIgclPowerLimits {
            power_limit_v2_w: (r_w == 0).then_some(w),
            v2_error: (r_w != 0).then(|| format!("{r_w:#010x} ({})", explain_ctl_error(r_w))),
            power_limit_v1_w: (r_mw == 0).then_some(mw / 1000.0),
            v1_error: (r_mw != 0).then(|| format!("{r_mw:#010x} ({})", explain_ctl_error(r_mw))),
            oc_power_supported,
            oc_min,
            oc_max,
            oc_step,
            oc_default,
            freq_offset_v1_mhz: get_opt_f64(self.fn_oc_freq_offset_get),
            freq_offset_v2_mhz: get_opt_f64(self.fn_oc_freq_offset_get_v2),
            gpu_lock_mv: gpu_lock.map(|x| x.0),
            gpu_lock_mhz: gpu_lock.map(|x| x.1),
        }
    }

    fn set_power_limit(&self, watts: f64) -> ArcWriteItemResult {
        let dev = self.devices[self.dev_idx];
        let (rw, r2, r1) = unsafe {
            (
                (self.fn_oc_waiver_set)(dev),
                (self.fn_oc_power_set_v2)(dev, watts),
                (self.fn_oc_power_set)(dev, watts * 1000.0),
            )
        };
        let rb = self.read_power_limits();
        ArcWriteItemResult {
            ok: r2 == 0 || r1 == 0,
            message: format!(
                "Waiver={rw:#010x}, SetV2={r2:#010x} ({}), Set={r1:#010x} ({})",
                explain_ctl_error(r2),
                explain_ctl_error(r1)
            ),
            readback: rb
                .power_limit_v2_w
                .or(rb.power_limit_v1_w)
                .map(|v| format!("{v:.1} W")),
        }
    }
}

// ============================================================================
// Uniwill EC -> PECI (0x32) Pipeline (via acpi_ec_worker)
// ============================================================================

pub const EC_GPU_PIPELINE_REGS: &[(u16, &str)] = &[
    (0x044B, "EC 电源策略模式 (1/2/3/4)"),
    (0x044C, "APC 整机功耗读数 (W)"),
    (0x044F, "GPU 温度 (°C)"),
    (0x0490, "AC/电池状态标志 (bit0=AC)"),
    (0x04AB, "电池电量百分比 (%)"),
    (0x0720, "Host 写入 PSYS PL1 高字节"),
    (0x0721, "Host 写入 PSYS PL1 低字节"),
    (0x072D, "Host 写入 GPU PL1 (W)"),
    (0x072E, "Host 写入 GPU PL2 (W)"),
    (0x0741, "Host 使能标志 (bit0=接管)"),
    (0x0783, "Host 写入 CPU PL1 (0=跳过0x072D)"),
    (0x0784, "Host 写入 CPU PL2 (W)"),
    (0x0785, "Host 写入 CPU PL4 (W)"),
    (0x07AC, "APDT (ACPI DTTT 模式字节)"),
    (0x07C7, "OEM_BYTE7 (bit6=Benchmark)"),
    (0x07D3, "GPU Module ID (1/2/3)"),
    (0x0823, "PECI 发送请求位 (bit3=PL1, bit4=PL2)"),
    (0x0824, "PECI 读回确认位 (bit3=PL1, bit4=PL2)"),
    (0x086B, "EC min() 仲裁后 GPU PL1 (W)"),
    (0x086C, "EC min() 仲裁后 GPU PL2 (W)"),
    (0x08E2, "高功耗事件标志1"),
    (0x08EB, "高功耗事件标志2 (bit0/bit6)"),
    (0x09C0, "最终送往 PECI 的 GPU PL1 (W)"),
    (0x09C1, "最终送往 PECI 的 GPU PL2 (W)"),
    (0x09C2, "EC 策略表 GPU PL1 候选"),
    (0x09C3, "EC 策略表 GPU PL2 候选"),
    (0x09C4, "Host 镜像 GPU PL1 候选"),
    (0x09C5, "Host 镜像 GPU PL2 候选"),
    (0x09C6, "策略降额 GPU PL1 候选"),
    (0x09C8, "高温计数降额 GPU PL1 候选"),
    (0x09CA, "温度滞回降额 GPU PL1 候选"),
    (0x0A48, "PWR_DESC 模式索引"),
    (0x0A52, "ACPI 镜像 GPU PL1 A"),
    (0x0A53, "ACPI 镜像 GPU PL2 A"),
    (0x0A54, "ACPI 镜像 GPU PL1 B"),
    (0x0A55, "ACPI 镜像 GPU PL2 B"),
    (0x0EF0, "PECI 0x32/0x1A 上次发送 PL1 (W)"),
    (0x0EF1, "PECI 0x32/0x1A GPU 读回 PL1 (W)"),
    (0x0EF2, "PECI 0x32/0x1B 上次发送 PL2 (W)"),
    (0x0EF3, "PECI 0x32/0x1B GPU 读回 PL2 (W)"),
    (0x1665, "GPIO 状态 (bit2=dGPU PECI 就绪)"),
    (0x912C, "EC 功率表选择值 (0x10/0x20/0x30)"),
];

fn ec_read_fast_telemetry() -> Result<(u8, u8, u8, u8), String> {
    acpi_ec_worker().call(|ec| {
        Ok::<_, std::io::Error>((
            ec.read_u8(0x044C)?,
            ec.read_u8(0x044F)?,
            ec.read_u8(0x0EF1)?,
            ec.read_u8(0x0EF3)?,
        ))
    })?
    .map_err(|e| format!("EC 读取失败: {e}"))
}

fn ec_read_pipeline(include_regs: bool) -> Result<ArcEcPowerSnapshot, String> {
    acpi_ec_worker()
        .call(move |ec| {
            let r = |addr: u16| ec.read_u8(addr);
            let ctrl_0741 = r(0x0741)?;
            let gpio_1665 = r(0x1665)?;
            let snap = ArcEcPowerSnapshot {
                ctrl_0741,
                host_takeover: (ctrl_0741 & 0x01) != 0,
                cpu_pl1_w: r(0x0783)?,
                host_gpu_pl1_w: r(0x072D)?,
                host_gpu_pl2_w: r(0x072E)?,
                arb_gpu_pl1_w: r(0x086B)?,
                arb_gpu_pl2_w: r(0x086C)?,
                final_peci_pl1_w: r(0x09C0)?,
                final_peci_pl2_w: r(0x09C1)?,
                policy_gpu_pl1_w: r(0x09C2)?,
                policy_gpu_pl2_w: r(0x09C3)?,
                thermal_derate_pl1_w: r(0x09C8)?,
                hyst_derate_pl1_w: r(0x09CA)?,
                peci_sent_pl1_w: r(0x0EF0)?,
                peci_readback_pl1_w: r(0x0EF1)?,
                peci_sent_pl2_w: r(0x0EF2)?,
                peci_readback_pl2_w: r(0x0EF3)?,
                policy_mode: r(0x044B)?,
                table_sel: r(0x912C)?,
                peci_ready: (gpio_1665 & 0x04) != 0,
                regs: if include_regs {
                    EC_GPU_PIPELINE_REGS
                        .iter()
                        .filter_map(|&(addr, desc)| {
                            r(addr).ok().map(|v| ArcEcReg {
                                address: format!("{addr:#06X}"),
                                value: v,
                                hex: format!("{v:#04X}"),
                                desc: desc.to_string(),
                            })
                        })
                        .collect()
                } else {
                    Vec::new()
                },
            };
            Ok::<_, std::io::Error>(snap)
        })?
        .map_err(|e| format!("EC 仲裁链读取失败: {e}"))
}

fn ec_write_power_limits(pl1_w: u32, pl2_w: u32, ensure_takeover: bool) -> ArcWriteItemResult {
    let Ok(w1) = u8::try_from(pl1_w) else {
        return ArcWriteItemResult {
            ok: false,
            message: format!("EC PL1={pl1_w} W 超出 u8 范围 (1..=255)"),
            readback: None,
        };
    };
    let Ok(w2) = u8::try_from(pl2_w) else {
        return ArcWriteItemResult {
            ok: false,
            message: format!("EC PL2={pl2_w} W 超出 u8 范围 (1..=255)"),
            readback: None,
        };
    };
    if w1 == 0 || w2 == 0 {
        return ArcWriteItemResult {
            ok: false,
            message: "EC GPU PL1/PL2 需大于 0 W".to_string(),
            readback: None,
        };
    }

    let write_res = acpi_ec_worker().call(move |ec| {
        if ensure_takeover {
            let ctrl = ec.read_u8(0x0741)?;
            if (ctrl & 0x01) == 0 {
                ec.write_u8(0x0741, ctrl | 0x01)?;
            }
            if ec.read_u8(0x0783)? == 0 {
                ec.write_u8(0x0783, 45)?;
            }
        }
        ec.write_u8(0x072D, w1)?;
        ec.write_u8(0x072E, w2)?;
        Ok::<_, std::io::Error>(())
    });

    match write_res {
        Ok(Ok(())) => {
            thread::sleep(Duration::from_millis(350));
            let fmt_ec_pair = |a: u8, b: u8| -> String {
                if a == 0xFF && b == 0xFF {
                    "N/A(0xFF)".to_string()
                } else {
                    let fa = if a == 0xFF { "N/A".to_string() } else { format!("{a}W") };
                    let fb = if b == 0xFF { "N/A".to_string() } else { format!("{b}W") };
                    format!("{fa}/{fb}")
                }
            };
            let rb = ec_read_pipeline(false).ok().map(|s| {
                format!(
                    "Host={}/{}W -> 仲裁={} -> PECI发送={}/{}W -> GPU回读={}/{}W",
                    s.host_gpu_pl1_w,
                    s.host_gpu_pl2_w,
                    fmt_ec_pair(s.final_peci_pl1_w, s.final_peci_pl2_w),
                    s.peci_sent_pl1_w,
                    s.peci_sent_pl2_w,
                    s.peci_readback_pl1_w,
                    s.peci_readback_pl2_w
                )
            });
            ArcWriteItemResult {
                ok: true,
                message: format!("已写入 EC 0x072D={w1}W, 0x072E={w2}W"),
                readback: rb,
            }
        }
        Ok(Err(e)) => ArcWriteItemResult {
            ok: false,
            message: format!("EC 写入失败: {e}"),
            readback: None,
        },
        Err(e) => ArcWriteItemResult {
            ok: false,
            message: format!("EC 写入失败: {e}"),
            readback: None,
        },
    }
}

fn ec_set_host_takeover(enabled: bool) -> Result<u8, String> {
    acpi_ec_worker()
        .call(move |ec| {
            let ctrl = ec.read_u8(0x0741)?;
            let next = if enabled { ctrl | 0x01 } else { ctrl & !0x01 };
            ec.write_u8(0x0741, next)?;
            if enabled && ec.read_u8(0x0783)? == 0 {
                ec.write_u8(0x0783, 45)?;
            }
            ec.read_u8(0x0741)
        })?
        .map_err(|e| format!("EC 写入 0x0741 失败: {e}"))
}

// ============================================================================
// Unified ArcSdk / ArcState
// ============================================================================

#[derive(Default)]
struct ArcRuntime {
    sysman: Option<SysmanSession>,
    igcl: Option<IgclSession>,
    prev_energy: Option<(u64, u64)>,
}

impl ArcRuntime {
    fn ensure_sysman(&mut self) -> Result<&SysmanSession, String> {
        if self.sysman.is_none() {
            self.sysman = Some(SysmanSession::open()?);
        }
        Ok(self.sysman.as_ref().unwrap())
    }

    fn ensure_igcl(&mut self) -> Result<&IgclSession, String> {
        if self.igcl.is_none() {
            self.igcl = Some(IgclSession::open()?);
        }
        Ok(self.igcl.as_ref().unwrap())
    }

    fn compute_power_w(&mut self) -> Option<f64> {
        let cur = self.sysman.as_ref()?.read_energy()?;
        let prev = match self.prev_energy {
            Some(p) if cur.1 > p.1 => p,
            _ => {
                // Initial sample: wait briefly so the first snapshot already displays real wattage.
                thread::sleep(Duration::from_millis(80));
                let next = self.sysman.as_ref()?.read_energy()?;
                self.prev_energy = Some(next);
                let dt = next.1.saturating_sub(cur.1);
                return (dt > 0).then(|| next.0.saturating_sub(cur.0) as f64 / dt as f64);
            }
        };
        self.prev_energy = Some(cur);
        let dt = cur.1.saturating_sub(prev.1);
        (dt > 0).then(|| cur.0.saturating_sub(prev.0) as f64 / dt as f64)
    }

    fn collect_telemetry(&mut self, errors: &mut Vec<String>) -> ArcTelemetry {
        let mut t = ArcTelemetry::default();
        if let Err(e) = self.ensure_sysman() {
            errors.push(format!("Sysman: {e}"));
        }
        if let Err(e) = self.ensure_igcl() {
            errors.push(format!("IGCL: {e}"));
        }

        t.gpu_power_w = self.compute_power_w();

        if let Some(sys) = &self.sysman {
            t.temps_c = sys.read_temps();
            let (domains, _) = sys.read_freq_domains(false);
            if let Some(gpu) = domains.iter().find(|d| d.freq_type == 0).or_else(|| domains.first()) {
                t.gpu_clock_mhz = gpu.actual_mhz;
                t.gpu_request_mhz = gpu.request_mhz;
                t.gpu_tdp_mhz = gpu.tdp_mhz;
                t.gpu_efficient_mhz = gpu.efficient_mhz;
                t.freq_range_min_mhz = gpu.range_min_mhz;
                t.freq_range_max_mhz = gpu.range_max_mhz;
                t.gpu_voltage_v = gpu.voltage_v;
                t.throttle_mask = gpu.throttle_mask;
                t.throttle_reasons = gpu
                    .throttle_mask
                    .map(decode_throttle_reasons)
                    .unwrap_or_default();
            }
            if t.gpu_temp_c.is_none() {
                t.gpu_temp_c = t.temps_c.first().copied();
            }
        }

        if let Some(igcl) = &self.igcl {
            if let Some(gt) = igcl.read_telemetry() {
                t.gpu_clock_mhz = gt.gpu_clock_mhz.or(t.gpu_clock_mhz);
                t.gpu_voltage_v = gt.gpu_voltage_v.or(t.gpu_voltage_v);
                t.gpu_temp_c = gt.gpu_temp_c.or(t.gpu_temp_c);
                t.vram_clock_mhz = gt.vram_clock_mhz;
                t.vram_bw_mbps = gt.vram_bw_mbps;
                t.gpu_power_limited = gt.gpu_power_limited;
                t.gpu_temp_limited = gt.gpu_temp_limited;
            }
        }

        if let Ok((sys_w, ec_temp, pl1, pl2)) = ec_read_fast_telemetry() {
            t.system_power_w = Some(sys_w);
            t.ec_gpu_temp_c = Some(ec_temp);
            t.peci_readback_pl1_w = Some(pl1);
            t.peci_readback_pl2_w = Some(pl2);
        }

        t
    }
}

#[derive(Clone, Default)]
pub struct ArcState {
    runtime: Arc<Mutex<ArcRuntime>>,
}

impl ArcState {
    pub fn snapshot(&self, include_ec_regs: bool) -> ArcSnapshot {
        let mut rt = self.runtime.lock().unwrap_or_else(|e| e.into_inner());
        let mut errors = Vec::new();
        let telemetry = rt.collect_telemetry(&mut errors);

        let igcl_ver = rt.igcl.as_ref().map(|g| g.supported_version);
        let device = rt.sysman.as_ref().map(|s| {
            let d = s.device();
            ArcDeviceInfo {
                name: d.name.clone(),
                vendor_id: d.vendor_id,
                device_id: d.device_id,
                igcl_version: igcl_ver,
            }
        });

        let mut freq = ArcFreqInfo::default();
        let mut power_limits = ArcPowerLimits::default();

        if let Some(sys) = &rt.sysman {
            let (domains, clocks) = sys.read_freq_domains(true);
            if let Some(gpu) = domains.iter().find(|d| d.freq_type == 0).or_else(|| domains.first()) {
                freq.can_control = gpu.can_control;
                freq.hw_min_mhz = gpu.hw_min_mhz;
                freq.hw_max_mhz = gpu.hw_max_mhz;
                freq.range_min_mhz = gpu.range_min_mhz;
                freq.range_max_mhz = gpu.range_max_mhz;
                freq.range_error = gpu.range_error.clone();
            }
            freq.available_clocks_mhz = clocks;
            freq.domains = domains;

            match sys.read_power_limits() {
                Ok(p) => power_limits.sysman = Some(p),
                Err(e) => errors.push(format!("Sysman Power: {e}")),
            }
        }

        if let Some(igcl) = &rt.igcl {
            power_limits.igcl = Some(igcl.read_power_limits());
        }

        match ec_read_pipeline(include_ec_regs) {
            Ok(ec) => power_limits.ec = Some(ec),
            Err(e) => errors.push(e),
        }

        ArcSnapshot {
            device,
            telemetry,
            freq,
            power_limits,
            errors,
        }
    }

    pub fn telemetry(&self) -> ArcTelemetry {
        let mut rt = self.runtime.lock().unwrap_or_else(|e| e.into_inner());
        let mut errors = Vec::new();
        rt.collect_telemetry(&mut errors)
    }

    pub fn set_frequency(
        &self,
        mode: &str,
        min_mhz: Option<f64>,
        max_mhz: Option<f64>,
    ) -> Result<ArcFreqWriteResult, String> {
        let mut rt = self.runtime.lock().unwrap_or_else(|e| e.into_inner());
        let sys = rt.ensure_sysman()?;
        let (domains, _) = sys.read_freq_domains(false);
        let hw_min = domains
            .iter()
            .find(|d| d.freq_type == 0)
            .or_else(|| domains.first())
            .map(|d| d.hw_min_mhz)
            .filter(|&v| v > 0.0)
            .unwrap_or(300.0);

        match mode {
            "reset" => sys.set_frequency_range(-1.0, -1.0, true),
            "max" => {
                let hi = max_mhz.or(min_mhz).ok_or("缺少目标最高频率 (MHz)")?;
                sys.set_frequency_range(hw_min, hi, false)
            }
            "lock" => {
                let mhz = min_mhz.or(max_mhz).ok_or("缺少锁定频率 (MHz)")?;
                sys.set_frequency_range(mhz, mhz, false)
            }
            "range" => {
                let lo = min_mhz.ok_or("缺少最低频率 (MHz)")?;
                let hi = max_mhz.ok_or("缺少最高频率 (MHz)")?;
                sys.set_frequency_range(lo, hi, false)
            }
            other => Err(format!("未知频率设置模式: {other}")),
        }
    }

    pub fn set_power_limits(&self, req: &ArcPowerWriteRequest) -> ArcPowerWriteResult {
        let pl1 = req.pl1_w;
        let pl2 = req.pl2_w.unwrap_or_else(|| pl1.max(200));
        let tau = req.tau_ms.unwrap_or(64000).max(1);
        let mut out = ArcPowerWriteResult::default();

        if req.write_ec {
            out.ec = Some(ec_write_power_limits(pl1, pl2, req.ensure_ec_takeover));
        }

        if req.write_sysman || req.write_igcl {
            let mut rt = self.runtime.lock().unwrap_or_else(|e| e.into_inner());
            if req.write_sysman {
                out.sysman = Some(match rt.ensure_sysman() {
                    Ok(sys) => sys.set_power_limits(pl1, pl2, tau),
                    Err(e) => ArcWriteItemResult { ok: false, message: e, readback: None },
                });
            }
            if req.write_igcl {
                out.igcl = Some(match rt.ensure_igcl() {
                    Ok(igcl) => igcl.set_power_limit(f64::from(pl1)),
                    Err(e) => ArcWriteItemResult { ok: false, message: e, readback: None },
                });
            }
        }

        out
    }

    pub fn set_ec_takeover(&self, enabled: bool) -> Result<u8, String> {
        ec_set_host_takeover(enabled)
    }
}
