use crate::modules::wmi::get_model;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::{
    // mpsc::Sender,
    {Arc, Mutex},
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

lazy_static! {
    pub static ref MODEL_ID: i64 = get_model();
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

// #[derive(Clone)]
// pub struct ChannelControlState {
//     pub tx: Arc<Mutex<Sender<String>>>,
// }
