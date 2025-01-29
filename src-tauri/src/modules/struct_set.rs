use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

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

#[derive(Serialize, Deserialize)]
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

pub struct ChannelControlState {
    pub tx: Arc<Mutex<Sender<String>>>,
}

pub struct ChannelControlState64 {
    pub tx: Arc<Mutex<Sender<i64>>>,
}
