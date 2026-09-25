const tauri = window.__TAURI__;
const { invoke } = tauri.core;

const pages = [
  "monitor",
  "tuning",
  "fan",
  "system",
  "display",
  "keyboard",
  "lightbar",
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
  for (const page of pages) {
    if (page === "lightbar" && (model === "LAPAC71H" || model == "LAPAC71G")) {
      continue;
    }
    const html = await loadPage(page);

    content.insertAdjacentHTML("beforeend", html);
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