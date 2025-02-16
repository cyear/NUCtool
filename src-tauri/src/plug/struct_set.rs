use std::path::PathBuf;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::{
    {Arc, Mutex},
};

#[cfg(windows)]
use crate::win_plug::wmi::get_model;
#[cfg(unix)]
use crate::{
    linux_plug::sysfs::{get_sys, set_sys},
    plug::config::find_hwmon_with_name
};


pub static R_TDP_GPU1: &str = "0x000001000000073d";
pub static R_TDP_GPU2: &str = "0x0000010000000733";
pub static R_TDP_CPU1: &str = "0x0000010000000783";
pub static R_TDP_CPU2: &str = "0x0000010000000784";
pub static R_TDP_TCC: &str = "0x0000010000000786";
pub static R_FAN_L1: &str = "0x000001000000046C";
pub static R_FAN_L2: &str = "0x000001000000046D";
pub static R_FAN_R1: &str = "0x0000010000000464";
pub static R_FAN_R2: &str = "0x0000010000000465";
pub static R_TEMP_L: &str = "0x000001000000043E";
pub static R_TEMP_R: &str = "0x000001000000044F";
pub static R_FAN_MODE: &str = "0x0000010000000751";
pub static W_FAN_AC71H_TURBO: &str = "0x0000000000400751";
pub static W_FAN_KC71F_TURBO: &str = "0x0000000000500751";
pub static W_FAN_RESET: &str = "0x0000000000A00751";
#[cfg(windows)]
lazy_static! {
    pub static ref MODEL_ID: i64 = get_model();
}
#[cfg(unix)]
lazy_static! {
    pub static ref DRIVER_PATH: PathBuf = find_hwmon_with_name();
}

#[derive(Serialize, Deserialize)]
pub struct FanPoint {
    pub temperature: i32,
    pub speed: i32,
}

#[derive(Serialize, Deserialize)]
pub struct FanData {
    pub left_fan: Vec<FanPoint>,
    pub right_fan: Vec<FanPoint>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FanSpeeds {
    pub left_fan_speed: i64,
    pub right_fan_speed: i64,
    pub left_temp: i64,
    pub right_temp: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Tdp {
    pub cpu1: i64,
    pub cpu2: i64,
    pub gpu1: i64,
    pub gpu2: i64,
    pub tcc: i64,
}

pub struct FanControlState {
    pub is_running: Arc<Mutex<bool>>,
}

#[cfg(unix)]
pub struct ApiFan {
    cpu: PathBuf,
    gpu: PathBuf,
    r_fan_l: PathBuf,
    r_fan_r: PathBuf,
    w_fan_l: PathBuf,
    w_fan_r: PathBuf,
    mode: PathBuf
}

#[cfg(unix)]
impl ApiFan {
    pub fn init() -> Self {
        ApiFan {
            cpu: DRIVER_PATH.join("temp1_input"),
            gpu: DRIVER_PATH.join("temp2_input"),
            r_fan_l: DRIVER_PATH.join("fan2_input"),
            r_fan_r: DRIVER_PATH.join("fan1_input"),
            w_fan_l: DRIVER_PATH.join("pwm2"),
            w_fan_r: DRIVER_PATH.join("pwm1"),
            mode: DRIVER_PATH.join("pwm1_enable")
        }
    }
    pub fn get_cpu_temp(&self) -> i64 {
        get_sys(&self.cpu) / 1000
    }
    pub fn get_gpu_temp(&self) -> i64 {
        get_sys(&self.gpu) / 1000
    }
    pub fn get_fan_l(&self) -> i64 {
        get_sys(&self.r_fan_l)
    }

    pub fn get_fan_r(&self) -> i64 {
        get_sys(&self.r_fan_r)
    }
    pub fn set_fan_l(&self, n: i64) -> bool {
        set_sys(&self.w_fan_l, n)
    }
    pub fn set_fan_r(&self, n: i64) -> bool {
        set_sys(&self.w_fan_r, n)
    }
    pub fn set_fan(&self, l: i64, r: i64) -> bool {
        self.set_fan_l(l) && self.set_fan_r(r)
    }
    pub fn set_fan_auto(&self) -> bool {
        set_sys(&self.mode, 2)
    }
    pub fn set_fan_control(&self) -> bool {
        set_sys(&self.mode, 1)
    }
    pub fn get_fan_mode(&self) -> i64 {
        get_sys(&self.mode)
    }
    pub fn get_fan_speeds(&self) -> FanSpeeds {
        FanSpeeds {
            left_fan_speed: self.get_fan_l(),
            right_fan_speed: self.get_fan_r(),
            left_temp: self.get_cpu_temp(),
            right_temp: self.get_gpu_temp()
        }
    }
}