#[cfg(unix)]
use std::path::PathBuf;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::{
    {Arc, Mutex},
};
use std::thread;
use std::time::Duration;
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

pub static R_TDP_GPU1: &str = "0x000001000000072d";
pub static R_TDP_GPU2: &str = "0x000001000000072e";
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
pub static R_AC_LED_COLOR: &str = "0x00000100000007EA";
pub static R_DC_LED_COLOR: &str = "0x00000100000007EB";
pub static W_AC_LED_COLOR_Y: &str = "0x00000000002A07EA";
pub static W_AC_LED_COLOR_N: &str = "0x00000000000A07EA";
pub static W_DC_LED_COLOR_Y: &str = "0x00000000002A07EB";
pub static W_DC_LED_COLOR_N: &str = "0x00000000000A07EB";
// RGB: i64 = 标准RGB值 / 51 * 10
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FanSpeeds {
    pub left_fan_speed: i64,
    pub right_fan_speed: i64,
    pub left_temp: i64,
    pub right_temp: i64,
}

#[derive(Serialize, Deserialize)]
pub struct TDP {
    pub cpu1: i64,
    pub cpu2: i64,
    pub gpu1: i64,
    pub gpu2: i64,
    pub tcc: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RGB {
    pub(crate) r: i64,
    pub(crate) g: i64,
    pub(crate) b: i64
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
        println!("MODE: {}", out);
        if out <= 0 { return 1 } // 异常不管了
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
    /// (gpu1, gpu2, cpu1, cpu2, tcc)
    pub fn get_tdp(&self) -> (i64, i64, i64, i64, i64) {
        (
            wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, R_TDP_CPU1) & 0xFF,
            wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, R_TDP_CPU2) & 0xFF,
            wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, R_TDP_GPU1) & 0xFF,
            wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, R_TDP_GPU2) & 0xFF,
            wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, R_TDP_TCC)
        )
    }
    pub fn set_tdp(&self, t: TDP) -> bool {
        let _ = wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, format!("0x000000000{:02x}0783", t.cpu1).as_str());
        thread::sleep(Duration::from_secs_f64(0.5));
        let _ = wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, format!("0x000000000{:02x}0784", t.cpu2).as_str());
        thread::sleep(Duration::from_secs_f64(0.5));
        let _ = wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, format!("0x000000000{:02x}072d", t.gpu1).as_str());
        thread::sleep(Duration::from_secs_f64(0.5));
        let _ = wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, format!("0x000000000{:02x}072e", t.gpu2).as_str());
        thread::sleep(Duration::from_secs_f64(0.5));
        let _ = wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, format!("0x000000000{:02x}0786", t.tcc).as_str());
        true
    }
    pub fn set_ac_led_color_y(&self) -> bool {
        let _ = wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, W_AC_LED_COLOR_Y);
        true
    }
    pub fn set_ac_led_color_n(&self) -> bool {
        let _ = wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, W_AC_LED_COLOR_N);
        true
    }
    pub fn set_dc_led_color_y(&self) -> bool {
        let _ = wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, W_DC_LED_COLOR_Y);
        true
    }
    pub fn set_dc_led_color_n(&self) -> bool {
        let _ = wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, W_DC_LED_COLOR_N);
        true
    }
    /// 0 - Error, 1 - off, 2 - on
    pub fn get_ac_led_color(&self) -> i64 {
        let out = wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, R_AC_LED_COLOR);
        match out & 0xff {
            2 => 1,
            4 => 1,
            34 => 2,
            36 => 2,
            _ => {
                println!("COLOR AC ERROR: {}", out);
                0
            },
        }
    }
    pub fn get_dc_led_color(&self) -> i64 {
        let out =wmi_set(&self.in_cls, &self.svc, &self.obj_path, &self.method_name, R_DC_LED_COLOR); 
        match out {
            10 => 1,
            64 => 2,
            _ => { 
                println!("COLOR DC ERROR: {}", out);
                0
            },
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
    pub mode: PathBuf,
    pub cpu_pl1: PathBuf,
    pub cpu_pl2: PathBuf,
    pub gpu_pl1: PathBuf,
    pub gpu_pl2: PathBuf,
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
            cpu_pl1: DRIVER_PATH.join("power1_input"),
            cpu_pl2: DRIVER_PATH.join("power2_input"),
            gpu_pl1: DRIVER_PATH.join("power3_input"),
            gpu_pl2: DRIVER_PATH.join("power4_input"),
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
    pub fn get_tdp(&self) -> (i64, i64, i64, i64, i64) {
        (
            get_sys(&self.cpu_pl1) / 1000,
            get_sys(&self.cpu_pl2) / 1000,
            get_sys(&self.gpu_pl1) / 1000,
            get_sys(&self.gpu_pl2) / 1000,
            0
        )
    }
    pub fn set_tdp(&self, t: TDP) -> bool {
        set_sys(&self.cpu_pl1, t.cpu1 * 1000)
            && set_sys(&self.cpu_pl2, t.cpu2 * 1000)
            && set_sys(&self.gpu_pl1, t.gpu1 * 1000)
            && set_sys(&self.gpu_pl2, t.gpu2 * 1000)
    }
}