import { t } from "./i18n.js";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const translation = (key) => t(`system.batteryDetails.${key}`);

function formatReading(field, data) {
  const value = data[field];
  if (value == null) return "—";

  switch (field) {
    case "full_over_24_hours":
    case "erm_threshold_reached":
      return translation(value ? "yes" : "no");
    case "windows_percent":
    case "charge_limit":
      return `${value}%`;
    case "design_capacity":
    case "full_charge_capacity":
    case "remaining_capacity": {
      if (!data.capacity_unit) return "—";
      const reading = `${value} ${data.capacity_unit}`;
      if (field === "remaining_capacity" && data.capacity_unit === "mWh" && data.design_voltage_mv) {
        const estimatedMah = Math.round(value * 1000 / data.design_voltage_mv);
        return `${reading} (≈ ${estimatedMah} mAh)`;
      }
      return reading;
    }
    case "design_voltage_mv":
    case "voltage_mv":
      return `${(value / 1000).toFixed(3)} V`;
    case "battery_current_ma":
      return `${value > 0 ? "+" : ""}${value} mA`;
    case "battery_warning":
      return value === 0
        ? translation("normal")
        : `${translation("warningActive")} (0x${value.toString(16).toUpperCase().padStart(2, "0")})`;
    default:
      return String(value);
  }
}

export async function initBatteryDetails() {
  const page = document.getElementById("page-system");
  if (!page) return;

  const readings = page.querySelectorAll("[data-battery-field]");
  const remaining = page.querySelector('[data-battery-field="remaining_capacity"]');
  const pageIsActive = () => page.classList.contains("active");
  let data = {};

  function render() {
    for (const reading of readings) {
      reading.textContent = formatReading(reading.dataset.batteryField, data);
    }
    remaining.title = data.remaining_capacity != null && data.capacity_unit === "mWh" && data.design_voltage_mv
      ? translation("capacityEstimate")
      : "";
  }

  await listen("battery-details-update", ({ payload }) => {
    if (!pageIsActive()) return;
    data = payload;
    render();
  });
  window.addEventListener("nuctool-language-changed", render);

  let previousActive;
  function syncPageState() {
    const active = pageIsActive();
    if (active === previousActive) return;
    previousActive = active;
    invoke("set_battery_details_active", { active }).catch((error) => {
      console.error("Battery details page state:", error);
    });
  }
  const observer = new MutationObserver(syncPageState);
  observer.observe(page, { attributes: true, attributeFilter: ["class"] });
  syncPageState();
}
