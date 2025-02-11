use crate::modules::{
    struct_set::{
        FanControlState, FanSpeeds, MODEL_ID, R_FAN_L1, R_FAN_L2, R_FAN_R1,
        R_FAN_R2, R_TEMP_L, R_TEMP_R,
    },
    wmi::{wmi_init, wmi_set},
};
use notify_rust::Notification;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{Emitter, State, Window};

/**
* @Author: cyear
* @Create time: 2025-01-30
* @Description: 风扇控制
* @Version: 0.3.5
**/

pub fn fan_init() {
    let (in_cls, svc, obj_path, method_name) = wmi_init();
    let out = wmi_set(
        &in_cls,
        &svc,
        &obj_path,
        &method_name,
        "0x0000010000000751".to_string().as_str(),
    );
    if out == 27664 {
        let _ = wmi_set(
            &in_cls,
            &svc,
            &obj_path,
            &method_name,
            "0x0000000000400751".to_string().as_str(),
        );
    } else if out == 27648 {
        let _ = wmi_set(
            &in_cls,
            &svc,
            &obj_path,
            &method_name,
            "0x0000000000500751".to_string().as_str(),
        );
    }
}

pub fn fan_reset() {
    let (in_cls, svc, obj_path, method_name) = wmi_init();
    let _ = wmi_set(
        &in_cls,
        &svc,
        &obj_path,
        &method_name,
        "0x0000000000A00751".to_string().as_str(),
    );
}

pub fn fan_set(left: i16, right: i16) {
    let (in_cls, svc, obj_path, method_name) = wmi_init();
    let out = wmi_set(
        &in_cls,
        &svc,
        &obj_path,
        &method_name,
        "0x0000010000000751".to_string().as_str(),
    );
    if out == 27664 && out == 27648 {
        fan_init();
        println!("风扇状态异常已尝试恢复");
    }
    let left = left * 2;
    let right = right * 2;
    println!("{} {}", left, right);
    if *MODEL_ID == 1 {
        let _ = wmi_set(
            &in_cls,
            &svc,
            &obj_path,
            &method_name,
            format!("0x000000000{:02x}1809", left).as_str(),
        );
        let _ = wmi_set(
            &in_cls,
            &svc,
            &obj_path,
            &method_name,
            format!("0x000000000{:02x}1804", right).as_str(),
        );
    } else {
        let _ = wmi_set(
            &in_cls,
            &svc,
            &obj_path,
            &method_name,
            format!("0x000000000{:02x}1809", right).as_str(),
        );
        let _ = wmi_set(
            &in_cls,
            &svc,
            &obj_path,
            &method_name,
            format!("0x000000000{:02x}1804", left).as_str(),
        );
    }
}

pub fn speed_c(speed_n: i64, speed_l: i64, temp_n: i64, temp_l: i64, temp: i64) -> i64 {
    println!("{} {} {} {} {}", speed_n, speed_l, temp_n, temp_l, temp);
    speed_l
        + (((speed_n - speed_l) as f64 / ((temp_n - temp_l) as f64 + 0.001))
            * (temp - temp_l) as f64) as i64
}

pub fn cpu_temp(left: &Option<&serde_json::Value>, right: &Option<&serde_json::Value>) {
    let (in_cls, svc, obj_path, method_name) = wmi_init();
    let cpu_out = wmi_set(
        &in_cls,
        &svc,
        &obj_path,
        &method_name,
        R_TEMP_L.to_string().as_str(),
    );
    let gpu_out = wmi_set(
        &in_cls,
        &svc,
        &obj_path,
        &method_name,
        R_TEMP_R.to_string().as_str(),
    ) & 0xFF;
    println!("CPU Temp: {:?}, GPU Temp: {:?}", &cpu_out, &gpu_out);
    if cpu_out > 95 || gpu_out > 95 {
        fan_set(100, 100);
        return;
    }
    let (mut l_c, mut s_c, mut r_c, mut s_c_) = (0i64, 0i64, 0i64, 0i64);
    if let (Some(left), Some(right)) = (left.expect("l").as_array(), right.expect("r").as_array()) {
        for l_ in left {
            if let (Some(l), Some(s)) = (
                l_.get("temperature").expect("转换错误").as_i64(),
                l_.get("speed").expect("转换错误").as_i64(),
            ) {
                if l >= cpu_out {
                    for r_ in right {
                        if let (Some(r), Some(s_)) = (
                            r_.get("temperature").expect("转换错误").as_i64(),
                            r_.get("speed").expect("转换错误").as_i64(),
                        ) {
                            if r >= gpu_out {
                                let s = speed_c(s, s_c, l, l_c, cpu_out);
                                let s_ = speed_c(s_, s_c_, r, r_c, gpu_out);
                                fan_set(s as i16, s_ as i16);
                                println!(
                                    "cpu_t: {:?} l_fan: {:?} gpu_t: {:?} r_fan: {:?}",
                                    cpu_out, s, gpu_out, s_
                                );
                                return;
                            } else {
                                (l_c, s_c, r_c, s_c_) = (l, s, r, s_);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[tauri::command]
pub fn get_fan_speeds(window: Window) {
    thread::spawn(move || {
        println!("get fan loop...");
        let (in_cls, svc, obj_path, method_name) = wmi_init();
        loop {
            let l_fan_1 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_FAN_L1);
            let l_fan_2 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_FAN_L2);
            let r_fan_1 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_FAN_R1);
            let r_fan_2 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_FAN_R2);
            let l_temp = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_TEMP_L);
            let r_temp = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_TEMP_R);
            if *MODEL_ID == 1 {
                window
                    .emit(
                        "get-fan-speeds",
                        FanSpeeds {
                            left_fan_speed: (l_fan_1 & 0xFF) << 8 | l_fan_2,
                            right_fan_speed: (r_fan_1 & 0xFF) << 8 | r_fan_2,
                            left_temp: l_temp,
                            right_temp: r_temp & 0xFF,
                        },
                    )
                    .unwrap()
            } else {
                window
                    .emit(
                        "get-fan-speeds",
                        FanSpeeds {
                            left_fan_speed: (r_fan_1 & 0xFF) << 8 | r_fan_2,
                            right_fan_speed: (l_fan_1 & 0xFF) << 8 | l_fan_2,
                            left_temp: l_temp,
                            right_temp: r_temp & 0xFF,
                        },
                    )
                    .unwrap()
            }
            thread::sleep(Duration::from_secs_f64(2.5));
        }
    });
}

#[tauri::command]
pub fn start_fan_control(
    fan_data: serde_json::Value,
    state: State<FanControlState>,
) {
    // Arc::clone(&tx.tx)
    //     .lock()
    //     .unwrap()
    //     .send("0x000001000000044F".to_string())
    //     .unwrap();
    let is_running = Arc::clone(&state.is_running);
    Notification::new()
        .summary("NUC X15 Fan Control")
        .body("正在运行")
        .icon("firefox")
        .show()
        .unwrap();
    // 打印接收到的风扇数据
    // println!("left fan data: {:?}", fan_data.get("left_fan"));
    // println!("right fan data: {:?}", fan_data.get("right_fan"));
    // 如果已经在运行，跳过启动
    if *is_running.lock().unwrap() {
        println!("Fan control is already running.");
        return;
    }
    fan_init();
    println!("接受风扇配置信息");
    // 启动新的控制线程
    *is_running.lock().unwrap() = true;
    thread::spawn(move || {
        while *is_running.lock().unwrap() {
            // 模拟执行时间
            // println!("Fan control loop running...");
            cpu_temp(&fan_data.get("left_fan"), &fan_data.get("right_fan"));
            thread::sleep(Duration::from_secs(2));
            // println!("TEMP: {}", cpu_temp());
        }
        println!("Fan control stopped.");
    });
}

#[tauri::command]
pub fn stop_fan_control(state: State<FanControlState>) {
    let mut is_running = state.is_running.lock().unwrap();
    // fan_init();
    *is_running = false; // 停止风扇控制
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        fan_reset();
        Notification::new()
            .summary("NUC X15 Fan Control")
            .body("停止运行")
            .icon("firefox")
            .show()
            .unwrap();
    });
}
