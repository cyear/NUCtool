use serde::{Deserialize, Serialize};
use tauri::{async_runtime::spawn_blocking, WebviewWindow};

use crate::acpi::{acpi_ec_worker, UniwillWmiEc};

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Ec,
    Wmi,
}

#[derive(Serialize)]
pub struct Reading {
    value: u8,
    raw: Option<u32>,
}

fn require_main_window(window: &WebviewWindow) -> Result<(), String> {
    if window.label() != "main" {
        return Err("EC debug requires the main window".into());
    }
    Ok(())
}

#[tauri::command]
pub async fn ec_debug_read(
    window: WebviewWindow,
    channel: Channel,
    address: u16,
) -> Result<Reading, String> {
    require_main_window(&window)?;
    spawn_blocking(move || match channel {
        Channel::Ec => {
            let value = acpi_ec_worker()
                .call(move |ec| ec.read_u8(address))?
                .map_err(|error| error.to_string())?;
            Ok(Reading { value, raw: None })
        }
        Channel::Wmi => {
            let wmi = UniwillWmiEc::new().map_err(|error| error.to_string())?;
            let raw = wmi
                .get_set((0x0100_u64 << 32) | u64::from(address))
                .map_err(|error| error.to_string())?;
            Ok(Reading {
                value: (raw & 0xFF) as u8,
                raw: Some(raw),
            })
        }
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn ec_debug_write(
    window: WebviewWindow,
    channel: Channel,
    address: u16,
    value: u8,
) -> Result<(), String> {
    require_main_window(&window)?;
    spawn_blocking(move || match channel {
        Channel::Ec => acpi_ec_worker()
            .call(move |ec| ec.write_u8(address, value))?
            .map_err(|error| error.to_string()),
        Channel::Wmi => {
            let wmi = UniwillWmiEc::new().map_err(|error| error.to_string())?;
            wmi.get_set((u64::from(value) << 16) | u64::from(address))
                .map_err(|error| error.to_string())?;
            Ok(())
        }
    })
    .await
    .map_err(|error| error.to_string())?
}
