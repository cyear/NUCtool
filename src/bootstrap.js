const tauri = window.__TAURI__;
const { invoke } = tauri.core;

const pages = [
  "monitor",
  "tuning",
  "arc-tuning",
  "fan",
  "newfan",
  "system",
  "display",
  "audio",
  "keyboard",
  "keyboard-hid",
  "lightbar",
  "bios",
  "ec-debug",
  "settings",
];

async function loadPage(page) {
  const response = await fetch(`pages/${page}.html`);

  if (!response.ok) {
    throw new Error(`Failed to load page: ${page}`);
  }

  return await response.text();
}

async function loadPages() {
  const content = document.querySelector(".content");

  if (!content) {
    throw new Error("Main content container not found");
  }
  const model = await invoke("get_sys_model");
  window.__NUCTOOL_MODEL__ = String(model ?? "");
  const modelUpper = window.__NUCTOOL_MODEL__.toUpperCase();
  const isLapkc = modelUpper.includes("LAPKC");
  const isLapac = modelUpper.includes("LAPAC");

  const keyboardNav = document.querySelector('.nav-btn[data-page="keyboard"]');
  const keyboardHidNav = document.querySelector('.nav-btn[data-page="keyboard-hid"]');
  const lightbarNav = document.querySelector('.nav-btn[data-page="lightbar"]');
  const arcTuningNav = document.querySelector('.nav-btn[data-page="arc-tuning"]');
  (isLapkc ? keyboardNav : keyboardHidNav)?.remove();
  if (!isLapkc) lightbarNav?.remove();
  if (!isLapac) arcTuningNav?.remove();

  for (const page of pages) {
    if ((page === "keyboard" && isLapkc) || (page === "keyboard-hid" && !isLapkc)) {
      continue;
    }
    if (page === "lightbar" && !isLapkc) {
      continue;
    }
    if (page === "arc-tuning" && !isLapac) {
      continue;
    }
    const html = await loadPage(page);

    content.insertAdjacentHTML("beforeend", html);
  }
  if (typeof window.initBiosPage === "function") {
    window.initBiosPage();
  } else {
    console.warn(
      "BIOS page initializer not found. Check whether js/bios.js loaded successfully."
    );
  }
}

async function bootstrap() {
  try {
    await loadPages();

    await import("./js/main.js");
  } catch (error) {
    console.error("NUCtool bootstrap failed:", error);
  }
}

bootstrap();
