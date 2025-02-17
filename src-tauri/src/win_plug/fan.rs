use std::{
    mem::swap,
    sync::Arc,
    time::Duration,
    thread
};
use tauri::{Emitter, State, Window};
use windows::{
    core::BSTR,
    Win32::System::Wmi::{
        IWbemClassObject, IWbemServices
    }
};
use colored::Colorize;
// use notify_rust::Notification;

use crate::{
    plug::struct_set::{
        FanControlState, FanSpeeds, MODEL_ID, R_FAN_L1, R_FAN_L2, R_FAN_R1,
        R_FAN_R2, R_TEMP_L, R_TEMP_R, R_FAN_MODE, W_FAN_AC71H_TURBO, W_FAN_KC71F_TURBO,
        W_FAN_RESET
    }, 
    win_plug::wmi::{wmi_init, wmi_set}
};

/**
* @Author: cyear
* @Create time: 2025-01-30
* @Description: 风扇控制
* @Version: 0.3.5
**/

pub fn fan_init() {
    let (in_cls, svc, obj_path, method_name) = wmi_init();
    // let out = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_FAN_MODE);
    // if out == 27664 {
    //     let _ = wmi_set(&in_cls, &svc, &obj_path, &method_name, W_FAN_AC71H_TURBO);
    // } else if out == 27648 {
    //     let _ = wmi_set(&in_cls, &svc, &obj_path, &method_name, W_FAN_KC71F_TURBO);
    // } else {
    //     return;
    // }
    if *MODEL_ID == 1 {
        let _ = wmi_set(&in_cls, &svc, &obj_path, &method_name, W_FAN_AC71H_TURBO);
    } else {
        let _ = wmi_set(&in_cls, &svc, &obj_path, &method_name, W_FAN_KC71F_TURBO);
    }
    println!("{}", "风扇初始化成功".green());
}

pub fn fan_reset() {
    let (in_cls, svc, obj_path, method_name) = wmi_init();
    let _ = wmi_set(&in_cls, &svc, &obj_path, &method_name, W_FAN_RESET);
    println!("{}", "风扇状态重置".red());
}

pub fn fan_set(left: i16, right: i16, (in_cls, svc, obj_path, method_name): (&IWbemClassObject, &IWbemServices, &BSTR, &BSTR)) {
    let out = wmi_set(in_cls, svc, obj_path, method_name, R_FAN_MODE);
    if out == 27664 || out == 27648 {
        println!("{}", "风扇状态异常".red());
        fan_reset();
        fan_init();
    }
    println!("FAN_L: {}% / FAN_R: {}% / OUT: {}", left, right, out);
    let mut left = left * 2;
    let mut right = right * 2;
    if *MODEL_ID != 1 {
        swap(&mut left, &mut right);
    }
    wmi_set(in_cls, svc, obj_path, method_name, format!("0x000000000{:02x}1809", left).as_str());
    wmi_set(in_cls, svc, obj_path, method_name, format!("0x000000000{:02x}1804", right).as_str());
}

/// 计算风扇百分比速度
/// ```
/// temp_old - 上次温度
/// speed_old - temp_old 对应风扇速度
/// temp - 大于等于设备的温度
/// speed - temp 对应风扇速度
/// temp_now - 当前温度
/// ```
pub fn speed_handle(temp_old: i64, speed_old: i64, temp: i64, speed: i64, temp_now: i64) -> i16 {
    println!("temp_old: {:?}, speed_old: {:?}, temp: {:?}, speed: {:?}, temp_now: {:?}", temp_old, speed_old, temp, speed, temp_now);
    (speed_old + ((speed - speed_old) * (temp_now - temp_old) / (temp - temp_old))) as i16
}

pub fn cpu_temp(
    left: &Option<&serde_json::Value>,
    right: &Option<&serde_json::Value>,
    (in_cls, svc, obj_path, method_name) : (&IWbemClassObject, &IWbemServices, &BSTR, &BSTR)
) {
    let cpu_out = wmi_set(in_cls, svc, obj_path, method_name, R_TEMP_L.to_string().as_str());
    let gpu_out = wmi_set(in_cls, svc, obj_path, method_name, R_TEMP_R.to_string().as_str()) & 0xFF;
    println!("CPU Temp: {:?}, GPU Temp: {:?}", &cpu_out, &gpu_out);
    if cpu_out > 95 || gpu_out > 95 {
        fan_set(100, 100, (in_cls, svc, obj_path, method_name));
        // fan_init();
        return;
    } else if cpu_out < 0 || gpu_out < 0 {
        println!("温度读取异常, cpu: {:?}, gpu: {:?}", cpu_out, gpu_out);
        return;
    }
    let (mut temp_old_l, mut speed_old_l) = (0i64, 0i64);
    let (mut temp_old_r, mut speed_old_r) = (0i64, 0i64);
    let (mut handle_left, mut handle_right) = (0i16, 0i16);
    if let (Some(left), Some(right)) = (left.unwrap().as_array(), right.unwrap().as_array()) {
        for l_ in left {
            if let (Some(temp_left), Some(speed_left)) = (
                l_.get("temperature").unwrap().as_i64(),
                l_.get("speed").unwrap().as_i64(),
            ) {
                if temp_left >= cpu_out {
                    handle_left = speed_handle(temp_old_l, speed_old_l, temp_left, speed_left, cpu_out);
                    break
                }
                temp_old_l = temp_left;
                speed_old_l = speed_left;
            }
        }
        for r_ in right {
            if let (Some(temp_right), Some(speed_right)) = (
                r_.get("temperature").unwrap().as_i64(),
                r_.get("speed").unwrap().as_i64(),
            ) {
                if temp_right >= gpu_out {
                    handle_right = speed_handle(temp_old_r, speed_old_r, temp_right, speed_right, gpu_out);
                    break
                }
                temp_old_r = temp_right;
                speed_old_r = speed_right;
            }
        }
        fan_set(handle_left, handle_right, (in_cls, svc, obj_path, method_name));
    }
}

#[tauri::command]
pub fn get_fan_speeds(window: Window) {
    thread::spawn(move || {
        let (in_cls, svc, obj_path, method_name) = wmi_init();
        println!("{}", "推送风扇信息".green());
        loop {
            let mut l_fan_1 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_FAN_L1);
            let l_fan_2 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_FAN_L2);
            let mut r_fan_1 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_FAN_R1);
            let r_fan_2 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_FAN_R2);
            let l_temp = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_TEMP_L);
            let r_temp = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_TEMP_R);
            // println!("f_l: {}, f_r: {}, t_l: {}, t_r: {}", l_fan_1, r_fan_1, l_temp, r_temp);
            if *MODEL_ID != 1 {
                swap(&mut l_fan_1, &mut r_fan_1);
            }
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
                .unwrap();
            thread::sleep(Duration::from_secs_f64(2.5));
        }
    });
}

#[tauri::command]
pub fn start_fan_control(fan_data: serde_json::Value, state: State<FanControlState>, ) {
    let is_running = Arc::clone(&state.is_running);
    // Notification::new()
    //     .summary("NUC X15 Fan Control")
    //     .body("正在运行")
    //     .icon("firefox")
    //     .show()
    //     .unwrap();
    // 如果已经在运行，跳过启动
    if *is_running.lock().unwrap() {
        println!("Fan control is already running.");
        return;
    }
    fan_init();
    // 启动新的控制线程
    *is_running.lock().unwrap() = true;
    thread::spawn(move || {
        let (in_cls, svc, obj_path, method_name) = wmi_init();
        while *is_running.lock().unwrap() {
            cpu_temp(&fan_data.get("left_fan"), &fan_data.get("right_fan"), (&in_cls, &svc, &obj_path, &method_name) );
            println!("---------------------------------------------------------------");
            thread::sleep(Duration::from_secs(3));
        }
    });
}

#[tauri::command]
pub fn stop_fan_control(state: State<FanControlState>) {
    let mut is_running = state.is_running.lock().unwrap();
    *is_running = false; // 停止风扇控制
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        fan_reset();
    });
    println!("{}", "Fan control stopped.".green());
}
