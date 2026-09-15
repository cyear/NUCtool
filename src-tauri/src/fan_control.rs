use crate::config::FanPoint;
use std::sync::{
    atomic::AtomicBool,
    Arc, Mutex,
};
use std::thread::JoinHandle;

pub struct FanControlState {
    pub running: Arc<AtomicBool>,
    pub thread: Mutex<Option<JoinHandle<()>>>,
}

impl FanControlState {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            thread: Mutex::new(None),
        }
    }
}
pub fn calculate_speed(
    points: &Vec<FanPoint>,
    temperature: u8,
) -> u8 {
    if points.is_empty() {
        return 100;
    }

    let mut points = points.to_vec();

    points.sort_by_key(|p| p.temperature);

    // 低于最低温度
    if temperature <= points[0].temperature {
        return points[0].speed.clamp(0, 100);
    }

    // 高于最高温度
    if temperature >= points[points.len() - 1].temperature {
        return points[points.len() - 1]
            .speed
            .clamp(0, 100);
    }

    // 找到温度所在区间
    for window in points.windows(2) {
        let p1 = &window[0];
        let p2 = &window[1];

        if temperature >= p1.temperature
            && temperature <= p2.temperature
        {
            let temp_range =
                (p2.temperature - p1.temperature) as f32;

            let temp_offset =
                (temperature - p1.temperature) as f32;

            let ratio =
                temp_offset / temp_range;

            let speed1 =
                p1.speed as f32;

            let speed2 =
                p2.speed as f32;

            let speed =
                speed1
                    + (speed2 - speed1) * ratio;

            return speed
                .round()
                .clamp(0.0, 100.0) as u8;
        }
    }

    100
}