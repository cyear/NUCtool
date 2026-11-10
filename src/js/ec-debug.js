import { t } from "./i18n.js";

function parseHex(text, max) {
  const hex = text.trim();
  if (!/^(?:0x)?[0-9a-f]+$/i.test(hex)) return null;
  const number = Number.parseInt(hex, 16);
  return number <= max ? number : null;
}

function formatHex(number, width) {
  return "0x" + number.toString(16).toUpperCase().padStart(width, "0");
}

export function initEcDebug() {
  const page = document.getElementById("page-ec-debug");
  const nav = document.querySelector('[data-page="ec-debug"]');
  const field = (name) => page.querySelector(`#ec-debug-${name}`);
  const channel = field("channel");
  const address = field("address");
  const value = field("value");
  const lock = field("lock");
  const read = field("read");
  const write = field("write");
  const reading = field("reading");
  const raw = field("raw");
  const status = field("status");
  const { invoke } = window.__TAURI__.core;
  const label = (name, params) => t(`ecDebug.${name}`, params);
  const isActive = () => page.classList.contains("active");

  let unlocked = false;
  let busy = false;
  let result = null;
  let message = null;

  function renderControls() {
    const addressInvalid = parseHex(address.value, 0xFFFF) === null;
    const valueInvalid = parseHex(value.value, 0xFF) === null;
    address.setAttribute("aria-invalid", String(addressInvalid && address.value.trim() !== ""));
    value.setAttribute("aria-invalid", String(valueInvalid && value.value.trim() !== ""));
    lock.textContent = label(unlocked ? "lock" : "unlock");
    channel.disabled = address.disabled = value.disabled = lock.disabled = busy;
    read.disabled = busy || addressInvalid;
    write.disabled = busy || !unlocked || addressInvalid || valueInvalid;
  }

  function renderResult() {
    reading.textContent = result
      ? label("reading", {
        channel: result.channel,
        address: formatHex(result.address, 4),
        byte: formatHex(result.value, 2),
        decimal: result.value,
      })
      : "—";
    raw.hidden = result?.raw == null;
    raw.textContent = raw.hidden ? "" : label("raw", { value: formatHex(result.raw, 8) });
    status.hidden = !message;
    status.textContent = message ? label(message.key, message.params) : "";
    status.classList.toggle("ec-debug-error", message?.key === "failed");
  }

  function clearResult() {
    result = message = null;
    renderResult();
    renderControls();
  }

  async function run(writing) {
    if (busy || !isActive() || (writing && !unlocked)) return;
    const parsedAddress = parseHex(address.value, 0xFFFF);
    const parsedValue = parseHex(value.value, 0xFF);
    if (parsedAddress === null || (writing && parsedValue === null)) return;

    const args = { channel: channel.value, address: parsedAddress };
    const channelName = args.channel.toUpperCase();
    busy = true;
    result = null;
    message = { key: writing ? "writing" : "readingInProgress" };
    renderControls();
    renderResult();

    try {
      if (writing) {
        await invoke("ec_debug_write", { ...args, value: parsedValue });
        message = { key: "written" };
      } else {
        const data = await invoke("ec_debug_read", args);
        result = { ...data, channel: channelName, address: parsedAddress };
        message = null;
      }
    } catch (error) {
      message = { key: "failed", params: { channel: channelName, error: String(error) } };
    } finally {
      busy = false;
      renderControls();
      renderResult();
    }
  }

  lock.addEventListener("click", () => {
    if (busy || !isActive()) return;
    unlocked = !unlocked;
    renderControls();
  });
  channel.addEventListener("change", clearResult);
  address.addEventListener("input", clearResult);
  value.addEventListener("input", renderControls);
  read.addEventListener("click", () => run(false));
  write.addEventListener("click", () => run(true));
  page.addEventListener("selectstart", (event) => event.stopPropagation());

  const observer = new MutationObserver(() => {
    if (isActive() || !unlocked) return;
    unlocked = false;
    renderControls();
  });
  observer.observe(page, { attributes: true, attributeFilter: ["class"] });

  function renderLanguage() {
    nav.setAttribute("aria-label", t("nav.ecDebug"));
    renderControls();
    renderResult();
  }

  window.addEventListener("nuctool-language-changed", renderLanguage);
  renderLanguage();
}
