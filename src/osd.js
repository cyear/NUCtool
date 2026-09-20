const tauri = window.__TAURI__;

if (!tauri) {
    throw new Error("Tauri global API is unavailable");
}

const { invoke } = tauri.core;
const { listen } = tauri.event;
const { getCurrentWindow } = tauri.window;

const appWindow = getCurrentWindow();

const osd = document.getElementById("osd");
const titleElement = document.getElementById("osd-title");
const subtitleElement = document.getElementById("osd-subtitle");

let hideTimer = null;

async function hideOsd() {
    osd.classList.remove("visible");

    // 等待 CSS 淡出动画完成
    await new Promise((resolve) => {
        setTimeout(resolve, 300);
    });

    await appWindow.hide();
}

function showOsd(title, subtitle = "") {
    titleElement.textContent = title ?? "";
    subtitleElement.textContent = subtitle ?? "";

    if (hideTimer !== null) {
        clearTimeout(hideTimer);
        hideTimer = null;
    }

    // 窗口已经被隐藏时，重新显示
    void appWindow.show();

    // 强制重新计算布局，确保动画重新触发
    void osd.offsetWidth;

    osd.classList.add("visible");

    hideTimer = setTimeout(() => {
        hideTimer = null;
        void hideOsd();
    }, 1800);
}

await listen("osd-show", (event) => {
    const payload = event.payload;

    if (!payload) {
        return;
    }

    showOsd(
        payload.title ?? "",
        payload.subtitle ?? ""
    );
});

await invoke("osd_ready");