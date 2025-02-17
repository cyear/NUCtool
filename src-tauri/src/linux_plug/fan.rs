use std::{
    sync::Arc,
    time::Duration,
    thread
};
use std::mem::swap;
use tauri::{Emitter, State, Window};
use colored::Colorize;

use crate::plug::{
    struct_set::{
        FanControlState, ApiFan, MODEL_ID

    },
};

/**
* @Author: cyear
* @Create time: 2025-02-16
* @Description: 风扇控制
* @Version: 0.4.0
**/

pub fn fan_init() {
    ApiFan::init().set_fan_control();
    println!("{}", "风扇初始化成功".green());
}

pub fn fan_reset() {
    ApiFan::init().set_fan_auto();
    println!("{}", "风扇状态重置".red());
}

pub fn fan_set(left: i64, right: i64, driver: &ApiFan) {
    let (mut l, mut r): (i64, i64) = ((2.55 * left as f64) as i64, (2.55 * right as f64) as i64);
    if l >= 254 {
        l = 255
    }
    if r >= 254 {
        r = 255
    }
    println!("FAN_L: {}% / FAN_R: {}% OUT: {} / {} {}", left, right, l, r, driver.set_fan(l, r));
}

/// 计算风扇百分比速度
/// ```
/// temp_old - 上次温度
/// speed_old - temp_old 对应风扇速度
/// temp - 大于等于设备的温度
/// speed - temp 对应风扇速度
/// temp_now - 当前温度
/// ```
pub fn speed_handle(temp_old: i64, speed_old: i64, temp: i64, speed: i64, temp_now: i64) -> i64 {
    println!("temp_old: {:?}, speed_old: {:?}, temp: {:?}, speed: {:?}, temp_now: {:?}", temp_old, speed_old, temp, speed, temp_now);
    (speed_old + ((speed - speed_old) * (temp_now - temp_old) / (temp - temp_old)))
}

pub fn cpu_temp(left: &Option<&serde_json::Value>, right: &Option<&serde_json::Value>, driver: &ApiFan) {
    let cpu_out = driver.get_cpu_temp();
    let gpu_out = driver.get_gpu_temp();
    println!("CPU Temp: {:?}, GPU Temp: {:?}", &cpu_out, &gpu_out);
    if cpu_out > 95 || gpu_out > 95 {
        fan_set(100, 100, driver);
        return;
    } else if cpu_out < 0 || gpu_out < 0 {
        println!("温度读取异常, cpu: {:?}, gpu: {:?}", cpu_out, gpu_out);
        return;
    }
    if driver.get_fan_mode() == 2 {
        println!("风扇异常自动恢复: {}", driver.set_fan_control());
    }
    let (mut temp_old_l, mut speed_old_l) = (0i64, 0i64);
    let (mut temp_old_r, mut speed_old_r) = (0i64, 0i64);
    let (mut handle_left, mut handle_right) = (0i64, 0i64);
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
        fan_set(handle_left, handle_right, driver);
    }
}

#[tauri::command]
pub fn get_fan_speeds(window: Window) {
    thread::spawn(move || {
        println!("{}", "推送风扇信息".green());
        let driver: ApiFan;
        if *MODEL_ID == 1 {
            driver = ApiFan::init();
        } else { 
            driver = ApiFan::init_0();
        }
        loop {

            window
                .emit(
                    "get-fan-speeds",
                    driver.get_fan_speeds()
                )
                .unwrap();
            thread::sleep(Duration::from_secs_f64(2.5));
        }
    });
}

#[tauri::command]
pub fn start_fan_control(fan_data: serde_json::Value, state: State<FanControlState>, ) {
    let is_running = Arc::clone(&state.is_running);
    if *is_running.lock().unwrap() {
        println!("Fan control is already running.");
        return;
    }
    fan_init();
    let driver: ApiFan;
    if *MODEL_ID == 1 {
        driver = ApiFan::init();
    } else { 
        driver = ApiFan::init_0();
    }
    // 启动新的控制线程
    *is_running.lock().unwrap() = true;
    thread::spawn(move || {
        while *is_running.lock().unwrap() {
            println!("---------------------------------------------------------------");
            cpu_temp(&fan_data.get("left_fan"), &fan_data.get("right_fan"), &driver);
            thread::sleep(Duration::from_secs(3));
        }
        println!("---------------------------------------------------------------");
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
