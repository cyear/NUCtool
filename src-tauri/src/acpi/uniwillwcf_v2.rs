use libloading::Library;
use sha2::{Digest, Sha256};
use std::{
    fs,
    ffi::CString,
    os::raw::c_int,
    path::Path
};

const EXPECTED_HASH_V2: &str = "838E83709F1E2C45A470868F210EEF1A3A48EAE10E13DBA896905E6DAA387497";

/// 状态
/// benchmark_mode 基准
/// passive_cooling 被动冷却
/// gaming_power_saver_mode 不清楚作用
/// selected_profile_index 模式
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NativePerformanceState {
    pub benchmark_mode: c_int,
    pub gaming_power_saver_mode: c_int,
    pub passive_cooling: c_int,
    pub selected_profile_index: c_int,
}