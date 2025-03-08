/**
* @Author: cyear
* @Create time: 2025-02-28
* @Description: 
* @Version: 1
**/
use crate::plug::struct_set::{RGB, TDP, ApiFan};

#[tauri::command]
pub fn get_tdp() -> (i64, i64, i64, i64, i64) {
    ApiFan::init().get_tdp()
}

#[tauri::command]
pub fn set_tdp(t: TDP) {
    ApiFan::init().set_tdp(t);
}

#[tauri::command]
#[cfg(windows)]
pub fn set_rgb(rgb: RGB) {
    println!("{:#?}", rgb);
}

#[tauri::command]
#[cfg(windows)]
pub fn get_rgb() -> RGB {
    // ApiFan::init()
    RGB {
        r: 0,
        g: 0,
        b: 0
    }
}

#[tauri::command]
#[cfg(unix)]
pub fn set_rgb(rgb: RGB) {
    println!("{:#?}", rgb);
}

#[tauri::command]
#[cfg(unix)]
pub fn get_rgb() -> RGB {
    RGB {
        r: 0,
        g: 0,
        b: 0
    }
}

#[tauri::command]
#[cfg(windows)]
pub fn set_rgb_color_y() {
    ApiFan::init().set_ac_led_color_y();
}

#[tauri::command]
#[cfg(windows)]
pub fn set_rgb_color_n() {
    ApiFan::init().set_ac_led_color_n();
}

#[tauri::command]
#[cfg(windows)]
pub fn get_rgb_color() -> bool {
    ApiFan::init().get_ac_led_color() == 2
}

#[tauri::command]
#[cfg(unix)]
pub fn set_rgb_color_y() {}

#[tauri::command]
#[cfg(unix)]
pub fn set_rgb_color_n() {}

#[tauri::command]
#[cfg(unix)]
pub fn get_rgb_color() -> bool {
    false
}
