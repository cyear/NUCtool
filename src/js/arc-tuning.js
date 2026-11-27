import { t } from "./i18n.js";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let initialized = false;
let watchEnabled = false;
let watchTimer = null;
let windowAvailable = true;
let busy = false;
let currentEcTakeover = false;
let igclAutoSynced = false;

const el = (id) => document.getElementById(id);
const isPageActive = () => el("page-arc-tuning")?.classList.contains("active") ?? false;

const fmtNum = (v, digits = 0, unit = "") =>
  typeof v === "number" && Number.isFinite(v) ? `${v.toFixed(digits)}${unit}` : "--";

const fmtHex = (v, width = 4) =>
  typeof v === "number" ? `0x${v.toString(16).toUpperCase().padStart(width, "0")}` : "--";

const fmtEcVal = (v, unit = " W") => (v === 0xff ? "N/A (0xFF)" : `${v}${unit}`);
const fmtEcPair = (a, b, unit = "W") =>
  a === 0xff && b === 0xff ? "N/A (0xFF)" : `${fmtEcVal(a, unit)}/${fmtEcVal(b, unit)}`;

function renderTelemetry(tel) {
  if (!tel) return;
  el("arc-tel-clock").textContent = fmtNum(tel.gpu_clock_mhz, 0, " MHz");
  const rangeText =
    typeof tel.freq_range_min_mhz === "number" && typeof tel.freq_range_max_mhz === "number"
      ? `${Math.round(tel.freq_range_min_mhz)} ~ ${Math.round(tel.freq_range_max_mhz)} MHz`
      : "--";
  el("arc-tel-freq-range").textContent = rangeText;
  if (rangeText !== "--" && el("arc-freq-current-range")) {
    el("arc-freq-current-range").textContent = `[${rangeText}]`.replace(" ~ ", ", ");
  }
  el("arc-tel-power").textContent = fmtNum(tel.gpu_power_w, 1, " W");
  el("arc-tel-sys-power").textContent = fmtNum(tel.system_power_w, 0, " W");
  el("arc-tel-temp").textContent = `${fmtNum(tel.gpu_temp_c, 1, " °C")} / ${fmtNum(tel.ec_gpu_temp_c, 0, " °C")}`;
  el("arc-tel-volt").textContent = fmtNum(tel.gpu_voltage_v, 3, " V");
  el("arc-tel-vram").textContent = `${fmtNum(tel.vram_clock_mhz, 0, " MHz")} (${fmtNum(tel.vram_bw_mbps, 0, " Mbps")})`;
  el("arc-tel-peci").textContent = `${fmtNum(tel.peci_readback_pl1_w, 0, "W")} / ${fmtNum(tel.peci_readback_pl2_w, 0, "W")}`;

  const setFlag = (id, label, active) => {
    const node = el(id);
    if (!node) return;
    node.textContent = `${label}: ${active === true ? "YES" : active === false ? "NO" : "--"}`;
    node.classList.toggle("active", active === true);
  };
  setFlag("arc-flag-pwr", t("arcTuning.powerLimitShort"), tel.gpu_power_limited);
  setFlag("arc-flag-thm", t("arcTuning.tempLimitShort"), tel.gpu_temp_limited);

  const maskStr = tel.throttle_mask != null ? fmtHex(tel.throttle_mask, 2) : "--";
  const reasons = tel.throttle_reasons?.length ? tel.throttle_reasons.join(", ") : t("arcTuning.none");
  el("arc-tel-throttle").textContent = `${maskStr} (${reasons})`;
}

function renderSnapshot(snap) {
  if (!snap) return;

  const banner = el("arc-errors-banner");
  if (snap.errors?.length) {
    banner.hidden = false;
    banner.textContent = snap.errors.join(" | ");
  } else {
    banner.hidden = true;
    banner.textContent = "";
  }

  if (snap.device) {
    const igclVer = snap.device.igcl_version != null ? ` · IGCL ${fmtHex(snap.device.igcl_version, 4)}` : "";
    el("arc-device-meta").textContent =
      `${snap.device.name || "Intel Arc"} · VID=${fmtHex(snap.device.vendor_id, 4)} DID=${fmtHex(snap.device.device_id, 4)}${igclVer}`;
  } else {
    el("arc-device-meta").textContent = t("arcTuning.noSysmanDevice");
  }

  renderTelemetry(snap.telemetry);

  // Frequency card
  const f = snap.freq || {};
  const rngText =
    f.range_min_mhz != null && f.range_max_mhz != null
      ? `[${Math.round(f.range_min_mhz)}, ${Math.round(f.range_max_mhz)}] MHz`
      : (f.range_error || "--");
  const hwText = f.hw_max_mhz > f.hw_min_mhz ? `[${Math.round(f.hw_min_mhz)}, ${Math.round(f.hw_max_mhz)}] MHz` : "--";
  el("arc-freq-summary").textContent = `canControl=${Boolean(f.can_control)} · ${f.available_clocks_mhz?.length || 0} steps`;
  el("arc-freq-current-range").textContent = rngText;
  el("arc-freq-hw-range").textContent = `HW ${hwText}`;

  if (f.range_min_mhz != null && !el("arc-freq-min").matches(":focus")) {
    el("arc-freq-min").value = Math.round(f.range_min_mhz);
  }
  if (f.range_max_mhz != null && !el("arc-freq-max").matches(":focus")) {
    el("arc-freq-max").value = Math.round(f.range_max_mhz);
    if (!el("arc-freq-single").value) {
      el("arc-freq-single").value = Math.round(f.range_max_mhz);
    }
  }

  const stepsGrid = el("arc-freq-steps-grid");
  stepsGrid.replaceChildren();
  for (const mhz of f.available_clocks_mhz || []) {
    if (mhz <= 0) continue;
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "arc-chip-btn";
    btn.textContent = `${Math.round(mhz)}`;
    btn.addEventListener("click", () => {
      const val = Math.round(mhz);
      el("arc-freq-single").value = val;
      el("arc-freq-max").value = val;
    });
    stepsGrid.appendChild(btn);
  }

  // Power locations
  const p = snap.power_limits || {};
  const ec = p.ec;
  if (ec) {
    currentEcTakeover = ec.host_takeover;
    const tkBtn = el("arc-ec-takeover-btn");
    tkBtn.textContent = `${t("arcTuning.hostTakeover")}: ${ec.host_takeover ? "ON" : "OFF"}`;
    tkBtn.classList.toggle("active", ec.host_takeover);

    el("arc-loc-ec-info").innerHTML = [
      `Host(0x072D/2E): <b>${ec.host_gpu_pl1_w} / ${ec.host_gpu_pl2_w} W</b> · CPU PL1(0x0783): <b>${ec.cpu_pl1_w} W</b>`,
      `仲裁(0x086B/0x09C0): <b>${fmtEcPair(ec.arb_gpu_pl1_w, ec.arb_gpu_pl2_w)}</b> → <b>${fmtEcPair(ec.final_peci_pl1_w, ec.final_peci_pl2_w)}</b>`,
      `PECI(0x0EF0~3) 发送/回读: <b>${ec.peci_sent_pl1_w}→${ec.peci_readback_pl1_w}W</b> | <b>${ec.peci_sent_pl2_w}→${ec.peci_readback_pl2_w}W</b>`,
      `策略表=${fmtEcPair(ec.policy_gpu_pl1_w, ec.policy_gpu_pl2_w)} · 降额=${fmtEcPair(ec.thermal_derate_pl1_w, ec.hyst_derate_pl1_w)} · Mode=${ec.policy_mode} · PECI=${ec.peci_ready ? "OK" : "NO"}`,
    ].join("<br>");

    if (!el("arc-ec-pl1").value) el("arc-ec-pl1").value = ec.host_gpu_pl1_w || ec.final_peci_pl1_w;
    if (!el("arc-ec-pl2").value) el("arc-ec-pl2").value = ec.host_gpu_pl2_w || ec.final_peci_pl2_w;

    if (ec.regs?.length) {
      const tbody = el("arc-ec-regs-tbody");
      tbody.replaceChildren();
      for (const reg of ec.regs) {
        const tr = document.createElement("tr");
        tr.innerHTML = `<td>${reg.address}</td><td>${reg.value}</td><td>${reg.hex}</td><td></td>`;
        tr.children[3].textContent = reg.desc;
        tbody.appendChild(tr);
      }
    }
  } else {
    el("arc-loc-ec-info").textContent = "--";
  }

  const sys = p.sysman;
  if (sys) {
    el("arc-loc-sysman-ctrl").textContent = `canControl=${sys.can_control}`;
    const extText = sys.ext_limits?.length
      ? sys.ext_limits.map((x) => `L${x.level}:${x.limit}(lock:${x.limit_locked ? 1 : 0})`).join(" ")
      : "None";
    el("arc-loc-sysman-info").innerHTML = [
      `Sustained(PL1): <b>${sys.sustained_w.toFixed(1)} W</b> (tau=${sys.sustained_tau_ms}ms, en=${sys.sustained_enabled ? 1 : 0})`,
      `Burst(PL2): <b>${sys.burst_w.toFixed(1)} W</b> · Peak AC/DC: <b>${sys.peak_ac_w.toFixed(1)}/${sys.peak_dc_w.toFixed(1)} W</b>`,
      `Default/Range: <b>${sys.default_w.toFixed(1)} W</b> [${sys.min_w.toFixed(1)}, ${sys.max_w.toFixed(1)}]`,
      `ExtLimits: ${extText}`,
    ].join("<br>");
    if (!el("arc-sys-pl1").value) el("arc-sys-pl1").value = Math.round(sys.sustained_w);
    if (!el("arc-sys-pl2").value) el("arc-sys-pl2").value = Math.round(sys.burst_w);
    if (!el("arc-sys-tau").value) el("arc-sys-tau").value = sys.sustained_tau_ms || 64000;
    if (!el("arc-all-pl1").value) el("arc-all-pl1").value = Math.round(sys.sustained_w);
    if (!el("arc-all-pl2").value) el("arc-all-pl2").value = Math.round(sys.burst_w);
  } else {
    el("arc-loc-sysman-ctrl").textContent = "--";
    el("arc-loc-sysman-info").textContent = "--";
  }

  const igcl = p.igcl;
  if (igcl) {
    el("arc-loc-igcl-ctrl").textContent = igcl.oc_power_supported
      ? "OC=true"
      : t("arcTuning.igclReadOnly");
    if (!igclAutoSynced) {
      igclAutoSynced = true;
      if (!igcl.oc_power_supported && el("arc-chk-igcl")) {
        el("arc-chk-igcl").checked = false;
      }
    }
    el("arc-loc-igcl-info").innerHTML = [
      `V2: <b>${igcl.power_limit_v2_w != null ? `${igcl.power_limit_v2_w.toFixed(1)} W` : igcl.v2_error || "--"}</b>`,
      `V1: <b>${igcl.power_limit_v1_w != null ? `${igcl.power_limit_v1_w.toFixed(1)} W` : igcl.v1_error || "--"}</b>`,
      `OC Range: [${igcl.oc_min.toFixed(1)}, ${igcl.oc_max.toFixed(1)}] step=${igcl.oc_step.toFixed(1)} def=${igcl.oc_default.toFixed(1)}`,
      `FreqOffset: ${igcl.freq_offset_v2_mhz ?? igcl.freq_offset_v1_mhz ?? "--"} MHz · VF Lock: ${igcl.gpu_lock_mhz ?? "--"} MHz`,
    ].join("<br>");
    if (!el("arc-igcl-pl").value && igcl.power_limit_v2_w != null) {
      el("arc-igcl-pl").value = Math.round(igcl.power_limit_v2_w);
    }
  } else {
    el("arc-loc-igcl-ctrl").textContent = "--";
    el("arc-loc-igcl-info").textContent = "--";
  }

  // Multi-domain table
  const domBody = el("arc-domains-tbody");
  domBody.replaceChildren();
  for (const d of f.domains || []) {
    const tr = document.createElement("tr");
    const rng = d.range_min_mhz != null && d.range_max_mhz != null
      ? `${Math.round(d.range_min_mhz)}~${Math.round(d.range_max_mhz)}`
      : (d.range_error || "--");
    tr.innerHTML = [
      `<td>[${d.index}] ${d.type_name}</td>`,
      `<td>${d.can_control}</td>`,
      `<td>${Math.round(d.hw_min_mhz)}~${Math.round(d.hw_max_mhz)}</td>`,
      `<td>${rng}</td>`,
      `<td>${fmtNum(d.actual_mhz)} / ${fmtNum(d.request_mhz)} / ${fmtNum(d.tdp_mhz)}</td>`,
      `<td>${fmtNum(d.voltage_v, 3, "V")}</td>`,
      `<td>${d.throttle_mask != null ? fmtHex(d.throttle_mask, 2) : "--"}</td>`,
    ].join("");
    domBody.appendChild(tr);
  }
}

function showFeedback(id, state, text) {
  const box = el(id);
  if (!box) return;
  const cls = state === true ? "ok" : state === false ? "err" : state;
  box.hidden = false;
  box.className = `arc-feedback ${cls}`;
  box.textContent = text;
}

async function refreshSnapshot(includeEcRegs = false) {
  if (busy) return;
  busy = true;
  try {
    const snap = await invoke("arc_get_snapshot", { includeEcRegs });
    renderSnapshot(snap);
  } catch (error) {
    showFeedback("arc-power-feedback", false, String(error));
  } finally {
    busy = false;
  }
}

async function pollTelemetry() {
  if (!watchEnabled || !windowAvailable || !isPageActive() || busy) return;
  try {
    const tel = await invoke("arc_get_telemetry");
    renderTelemetry(tel);
  } catch (error) {
    console.error("[ARC] telemetry poll failed:", error);
  }
}

export function syncArcTuningRefresh() {
  const shouldPoll = watchEnabled && windowAvailable && isPageActive();
  if (shouldPoll && watchTimer === null) {
    pollTelemetry();
    watchTimer = setInterval(pollTelemetry, 1000);
  } else if (!shouldPoll && watchTimer !== null) {
    clearInterval(watchTimer);
    watchTimer = null;
  }
}

async function applyFreq(mode, minMhz = null, maxMhz = null) {
  try {
    const res = await invoke("arc_set_frequency", { mode, minMhz, maxMhz });
    const rb = res.readback_min_mhz != null && res.readback_max_mhz != null
      ? ` -> [${Math.round(res.readback_min_mhz)}, ${Math.round(res.readback_max_mhz)}] MHz`
      : "";
    showFeedback("arc-freq-feedback", res.ok, `${res.message}${rb}`);
    await refreshSnapshot(false);
  } catch (error) {
    showFeedback("arc-freq-feedback", false, String(error));
  }
}

async function applyPower(request) {
  try {
    const res = await invoke("arc_set_power_limits", { request });
    const lines = [];
    let okCount = 0;
    let failCount = 0;
    for (const [label, item] of [["EC", res.ec], ["Sysman", res.sysman], ["IGCL", res.igcl]]) {
      if (!item) continue;
      if (item.ok) okCount++;
      else failCount++;
      lines.push(`[${label}] ${item.ok ? "OK" : "FAIL"}: ${item.message}${item.readback ? ` (${item.readback})` : ""}`);
    }
    const state = okCount > 0 && failCount > 0 ? "warn" : okCount > 0;
    showFeedback("arc-power-feedback", state, lines.join(" | ") || "未选择写入目标");
    await refreshSnapshot(false);
  } catch (error) {
    showFeedback("arc-power-feedback", false, String(error));
  }
}

export async function initArcTuning() {
  if (initialized) return;
  const page = el("page-arc-tuning");
  if (!page) return;
  initialized = true;

  el("arc-refresh-btn")?.addEventListener("click", () => {
    const includeRegs = Boolean(el("arc-diag-ec-details")?.open);
    refreshSnapshot(includeRegs);
  });

  el("arc-watch-btn")?.addEventListener("click", () => {
    watchEnabled = !watchEnabled;
    el("arc-watch-btn").textContent = t(watchEnabled ? "arcTuning.watchOn" : "arcTuning.watchOff");
    el("arc-watch-btn").classList.toggle("primary", watchEnabled);
    syncArcTuningRefresh();
  });

  el("arc-freq-range-btn")?.addEventListener("click", () => {
    const lo = Number(el("arc-freq-min").value);
    const hi = Number(el("arc-freq-max").value);
    applyFreq("range", lo, hi);
  });
  el("arc-freq-cap-btn")?.addEventListener("click", () => {
    const hi = Number(el("arc-freq-single").value);
    applyFreq("max", null, hi);
  });
  el("arc-freq-lock-btn")?.addEventListener("click", () => {
    const mhz = Number(el("arc-freq-single").value);
    applyFreq("lock", mhz, mhz);
  });
  el("arc-freq-reset-btn")?.addEventListener("click", () => applyFreq("reset"));

  document.querySelectorAll("[data-arc-watts]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const pl1 = Number(btn.dataset.arcWatts);
      const pl2 = Number(btn.dataset.arcPl2) || Math.max(pl1, 200);
      el("arc-all-pl1").value = pl1;
      el("arc-all-pl2").value = pl2;
    });
  });

  el("arc-all-apply-btn")?.addEventListener("click", () => {
    const pl1 = Number(el("arc-all-pl1").value);
    const pl2 = Number(el("arc-all-pl2").value) || Math.max(pl1, 200);
    if (!Number.isFinite(pl1) || pl1 <= 0) {
      showFeedback("arc-power-feedback", false, t("arcTuning.invalidWatts"));
      return;
    }
    applyPower({
      write_ec: el("arc-chk-ec").checked,
      write_sysman: el("arc-chk-sysman").checked,
      write_igcl: el("arc-chk-igcl").checked,
      pl1_w: Math.round(pl1),
      pl2_w: Math.round(pl2),
      tau_ms: Number(el("arc-sys-tau").value) || 64000,
      ensure_ec_takeover: true,
    });
  });

  el("arc-ec-apply-btn")?.addEventListener("click", () => {
    const pl1 = Number(el("arc-ec-pl1").value);
    const pl2 = Number(el("arc-ec-pl2").value) || pl1;
    if (!Number.isFinite(pl1) || pl1 <= 0) {
      showFeedback("arc-power-feedback", false, t("arcTuning.invalidWatts"));
      return;
    }
    applyPower({ write_ec: true, pl1_w: Math.round(pl1), pl2_w: Math.round(pl2), ensure_ec_takeover: true });
  });

  el("arc-sys-apply-btn")?.addEventListener("click", () => {
    const pl1 = Number(el("arc-sys-pl1").value);
    const pl2 = Number(el("arc-sys-pl2").value) || Math.max(pl1, 200);
    const tau = Number(el("arc-sys-tau").value) || 64000;
    if (!Number.isFinite(pl1) || pl1 <= 0) {
      showFeedback("arc-power-feedback", false, t("arcTuning.invalidWatts"));
      return;
    }
    applyPower({ write_sysman: true, pl1_w: Math.round(pl1), pl2_w: Math.round(pl2), tau_ms: Math.round(tau) });
  });

  el("arc-igcl-apply-btn")?.addEventListener("click", () => {
    const pl1 = Number(el("arc-igcl-pl").value);
    if (!Number.isFinite(pl1) || pl1 <= 0) {
      showFeedback("arc-power-feedback", false, t("arcTuning.invalidWatts"));
      return;
    }
    applyPower({ write_igcl: true, pl1_w: Math.round(pl1) });
  });

  el("arc-ec-takeover-btn")?.addEventListener("click", async () => {
    try {
      const reg = await invoke("arc_set_ec_takeover", { enabled: !currentEcTakeover });
      showFeedback("arc-power-feedback", true, `EC 0x0741 = ${fmtHex(reg, 2)}`);
      await refreshSnapshot(false);
    } catch (error) {
      showFeedback("arc-power-feedback", false, String(error));
    }
  });

  el("arc-ec-dump-btn")?.addEventListener("click", () => refreshSnapshot(true));
  el("arc-diag-ec-details")?.addEventListener("toggle", (ev) => {
    if (ev.target.open && el("arc-ec-regs-tbody").children.length === 0) {
      refreshSnapshot(true);
    }
  });

  document.querySelector('.nav-btn[data-page="arc-tuning"]')?.addEventListener("click", () => {
    if (el("arc-device-meta").textContent === "--") {
      refreshSnapshot(false);
    }
    syncArcTuningRefresh();
  });

  await listen("window-push-changed", ({ payload }) => {
    windowAvailable = Boolean(payload);
    syncArcTuningRefresh();
  }, { target: "main" });

  window.addEventListener("nuctool-language-changed", () => {
    const watchBtn = el("arc-watch-btn");
    if (watchBtn) {
      watchBtn.textContent = t(watchEnabled ? "arcTuning.watchOn" : "arcTuning.watchOff");
    }
  });

  windowAvailable = await invoke("get_window_push_enabled").catch(() => true);
}
