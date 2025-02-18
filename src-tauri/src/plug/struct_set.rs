#[cfg(unix)]
use std::path::PathBuf;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::{
    {Arc, Mutex},
};

#[cfg(windows)]
use crate::win_plug::wmi::{
    wmi_init, wmi_set, get_model
};
#[cfg(windows)]
use windows::{
    core::BSTR,
    Win32::System::Wmi::{
        IWbemClassObject, IWbemServices
    }
};
#[cfg(unix)]
use crate::{
    linux_plug::sysfs::{get_sys, set_sys, get_kernel_version, get_model_id},
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
    pub static ref MODEL_ID: i64 = get_model_id();
}
#[cfg(unix)]
lazy_static! {
    pub static ref KERNEL_ID: i64 = get_kernel_version();
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

#[cfg(windows)]
pub struct ApiFan {
    in_cls: IWbemClassObject,
    svc: IWbemServices,
    obj_path: BSTR,
    method_name: BSTR,
    r_fan_l1: &'static str,
    r_fan_l2: &'static str,
    r_fan_r1: &'static str,
    r_fan_r2: &'static str,
}

#[cfg(windows)]
impl ApiFan {
    pub fn init() -> Self {
        let (in_cls, svc, obj_path, method_name) = wmi_init();
        let (r_fan_l1, r_fan_l2, r_fan_r1, r_fan_r2) = if *MODEL_ID == 1 {
            (R_FAN_L1, R_FAN_L2, R_FAN_R1, R_FAN_R2)
        } else {
            (R_FAN_R1, R_FAN_R2, R_FAN_L1, R_FAN_L2)
        };
        ApiFan {
            in_cls, svc, obj_path, method_name,
            r_fan_l1, r_fan_l2, r_fan_r1, r_fan_r2,
        }
    }
    pub fn get_cpu_temp(&self) -> i64 {
        wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, R_TEMP_L)
    }
    pub fn get_gpu_temp(&self) -> i64 {
        wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, R_TEMP_R) & 0xFF
    }
    pub fn get_fan_l(&self) -> i64 {
        (wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, &self.r_fan_l1) & 0xFF) << 8 |
            wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, &self.r_fan_l2)
    }
    pub fn get_fan_r(&self) -> i64 {
        (wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, &self.r_fan_r1) & 0xFF) << 8 |
            wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, &self.r_fan_r2)
    }
    fn _set_fan(&self, l: i64, r: i64) {
        let _ = wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, format!("0x000000000{:02x}1809", l).as_str());
        let _ = wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, format!("0x000000000{:02x}1804", r).as_str());
    }
    /// MAX speed 200
    pub fn set_fan(&self, l: i64, r: i64) -> bool {
        let _ = if *MODEL_ID == 1 { &self._set_fan(l ,r) } else { &self._set_fan(r, l) };
        true
    }
    pub fn set_fan_auto(&self) -> bool {
        let _ = wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, W_FAN_RESET);
        true
    }
    pub fn set_fan_control(&self) -> bool {
        if *MODEL_ID == 1 {
            wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, W_FAN_AC71H_TURBO);
        } else {
            wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, W_FAN_KC71F_TURBO);
        }
        true
    }
    /// 1 - control, 2 - auto
    pub fn get_fan_mode(&self) -> i64 {
        let out = wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, R_FAN_MODE);
        if out == 27664 || out == 27648 { 2 } else { 1 }
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

#[cfg(unix)]
pub struct ApiFan {
    pub cpu: PathBuf,
    pub gpu: PathBuf,
    pub r_fan_l: PathBuf,
    pub r_fan_r: PathBuf,
    pub w_fan_l: PathBuf,
    pub w_fan_r: PathBuf,
    pub mode: PathBuf
}

#[cfg(unix)]
impl ApiFan {
    pub fn init() -> Self {
        let (r_fan_l, r_fan_r, w_fan_l, w_fan_r) = if *MODEL_ID == 1 {
            (DRIVER_PATH.join("fan2_input"), DRIVER_PATH.join("fan1_input"), DRIVER_PATH.join("pwm2"), DRIVER_PATH.join("pwm1"))
        } else {
            (DRIVER_PATH.join("fan1_input"), DRIVER_PATH.join("fan2_input"), DRIVER_PATH.join("pwm1"), DRIVER_PATH.join("pwm2"))
        };
        ApiFan {
            cpu: DRIVER_PATH.join("temp1_input"),
            gpu: DRIVER_PATH.join("temp2_input"),
            r_fan_l, r_fan_r, w_fan_l, w_fan_r,
            mode: DRIVER_PATH.join("pwm1_enable"),
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
    /// MAX speed 255
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