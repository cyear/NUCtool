use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

use crate::{
    acpi::{acpi_ec_worker, BatteryEcDetails},
    window_push::WindowPushState,
};

const UPDATE_INTERVAL: Duration = Duration::from_secs(3);

#[derive(Clone, Default)]
pub struct BatteryDetailsPage(Arc<AtomicBool>);

#[tauri::command]
pub fn set_battery_details_active(active: bool, page: State<'_, BatteryDetailsPage>) {
    page.0.store(active, Ordering::Relaxed);
}

#[derive(Clone, Serialize)]
struct BatteryDetails {
    windows_percent: Option<u8>,
    #[serde(flatten)]
    ec: BatteryEcDetails,
}

fn collect(can_push: impl Fn() -> bool + Send + 'static) -> BatteryDetails {
    let mut power = SYSTEM_POWER_STATUS::default();
    let has_status = unsafe { GetSystemPowerStatus(&mut power) }.is_ok();
    let battery_present = has_status && power.BatteryFlag & 0x80 == 0;
    let ec = if can_push() {
        acpi_ec_worker()
            .call(move |ec| {
                if can_push() {
                    ec.read_battery_details(battery_present)
                } else {
                    BatteryEcDetails::default()
                }
            })
            .unwrap_or_default()
    } else {
        BatteryEcDetails::default()
    };
    BatteryDetails {
        windows_percent: (battery_present && power.BatteryLifePercent <= 100)
            .then_some(power.BatteryLifePercent),
        ec,
    }
}

pub fn start(app: AppHandle) {
    let page = app.state::<BatteryDetailsPage>().inner().clone();
    let window_push = app.state::<WindowPushState>().inner().clone();
    thread::spawn(move || {
        let can_push = move || page.0.load(Ordering::Relaxed) && window_push.is_enabled();
        loop {
            let started = Instant::now();
            if can_push() {
                let details = collect(can_push.clone());
                if can_push() {
                    let _ = app.emit_to("main", "battery-details-update", details);
                }
            }
            thread::sleep(UPDATE_INTERVAL.saturating_sub(started.elapsed()));
        }
    });
}
