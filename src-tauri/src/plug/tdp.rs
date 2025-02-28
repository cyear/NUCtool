/**
* @Author: cyear
* @Create time: 2025-02-28
* @Description: 
* @Version: 1
**/
use crate::plug::struct_set::{ApiFan, Tdp};

#[tauri::command]
pub fn get_tdp() -> (i64, i64, i64, i64, i64) {
    ApiFan::init().get_tdp()
}

#[tauri::command]
pub fn set_tdp(t: Tdp) {
    ApiFan::init().set_tdp(t);
}