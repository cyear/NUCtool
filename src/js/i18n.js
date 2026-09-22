// ============================================================
// NUCtool i18n
// ============================================================

import zhCN from "./locales/zh-CN.js";
import enUS from "./locales/en-US.js";
import jaJP from "./locales/ja-JP.js";
import ruRU from "./locales/ru-RU.js";

const locales = {
  "zh-CN": zhCN,
  "en-US": enUS,
  "ja-JP": jaJP,
  "ru-RU": ruRU,
};

// 默认语言
const DEFAULT_LANGUAGE = "en-US";

// 当前语言
let currentLanguage = DEFAULT_LANGUAGE;

// ------------------------------------------------------------
// 获取嵌套翻译
// 例如：nav.monitor
// ------------------------------------------------------------
function getNestedValue(object, path) {
  return path.split(".").reduce((result, key) => {
    if (result === undefined || result === null) {
      return undefined;
    }

    return result[key];
  }, object);
}

// ------------------------------------------------------------
// 获取系统语言
// ------------------------------------------------------------
function detectSystemLanguage() {
  const language = navigator.language || navigator.userLanguage || "";

  if (language.toLowerCase().startsWith("zh")) {
    return "zh-CN";
  }
  if (language.toLowerCase().startsWith("ja")) {
    return "ja-JP";
  }
  if (language.toLowerCase().startsWith("ru")) {
    return "ru-RU";
  }
  return "en-US";
}
// ------------------------------------------------------------
// 获取保存的语言设置
// ------------------------------------------------------------
function getSavedLanguage() {
  const saved = localStorage.getItem("nuctool-language");

  if (!saved) {
    return null;
  }

  if (saved === "auto") {
    return "auto";
  }

  if (locales[saved]) {
    return saved;
  }

  return null;
}

// ------------------------------------------------------------
// 获取当前实际语言
// ------------------------------------------------------------
export function getLanguage() {
  return currentLanguage;
}

// ------------------------------------------------------------
// 获取当前语言设置
// auto / zh-CN / en-US
// ------------------------------------------------------------
export function getLanguageSetting() {
  return localStorage.getItem("nuctool-language") || "auto";
}

// ------------------------------------------------------------
// 设置语言
// ------------------------------------------------------------
export function setLanguage(language) {
  if (language === "auto") {
    localStorage.setItem("nuctool-language", "auto");

    currentLanguage = detectSystemLanguage();
  } else if (locales[language]) {
    localStorage.setItem("nuctool-language", language);

    currentLanguage = language;
  } else {
    return;
  }

  document.documentElement.lang = currentLanguage;

  updateTranslations();

  window.dispatchEvent(
    new CustomEvent("nuctool-language-changed", {
      detail: {
        language: currentLanguage,
        setting: getLanguageSetting(),
      },
    }),
  );
}

// ------------------------------------------------------------
// 翻译
//
// t("nav.monitor")
// t("performance.temperatureValue", { value: 55 })
// ------------------------------------------------------------
export function t(key, params = {}) {
  let value = getNestedValue(locales[currentLanguage], key);

  // 当前语言没有时回退中文
  if (value === undefined) {
    value = getNestedValue(locales[DEFAULT_LANGUAGE], key);
  }

  // 最终还是没有，直接返回 key
  if (value === undefined || value === null) {
    return key;
  }

  value = String(value);

  // {{xxx}} 参数替换
  value = value.replace(/\{\{(\w+)\}\}/g, (_, name) => {
    return params[name] !== undefined ? params[name] : "";
  });

  return value;
}

// ------------------------------------------------------------
// 设置元素文本
// ------------------------------------------------------------
function translateElement(element) {
  const key = element.dataset.i18n;

  if (!key) {
    return;
  }

  element.textContent = t(key);
}

// ------------------------------------------------------------
// 设置 placeholder
// ------------------------------------------------------------
function translatePlaceholder(element) {
  const key = element.dataset.i18nPlaceholder;

  if (!key) {
    return;
  }

  element.placeholder = t(key);
}

// ------------------------------------------------------------
// 设置 title
// ------------------------------------------------------------
function translateTitle(element) {
  const key = element.dataset.i18nTitle;

  if (!key) {
    return;
  }

  element.title = t(key);
}

// ------------------------------------------------------------
// 更新整个页面翻译
// ------------------------------------------------------------
export function updateTranslations() {
  // 普通文本
  document.querySelectorAll("[data-i18n]").forEach((element) => {
    translateElement(element);
  });

  // placeholder
  document
    .querySelectorAll("[data-i18n-placeholder]")
    .forEach((element) => {
      translatePlaceholder(element);
    });

  // title
  document
    .querySelectorAll("[data-i18n-title]")
    .forEach((element) => {
      translateTitle(element);
    });

  // select 当前语言
  const languageSelect = document.getElementById("language-select");

  if (languageSelect) {
    languageSelect.value = getLanguageSetting();
  }
}

// ------------------------------------------------------------
// 初始化
// ------------------------------------------------------------
export function initI18n() {
  const savedLanguage = getSavedLanguage();

  if (savedLanguage === "auto") {
    currentLanguage = detectSystemLanguage();
  } else if (savedLanguage && locales[savedLanguage]) {
    currentLanguage = savedLanguage;
  } else {
    currentLanguage = detectSystemLanguage();
  }

  document.documentElement.lang = currentLanguage;

  updateTranslations();
}

// ------------------------------------------------------------
// 全局 API
//
// 这样 main.js 不一定必须 import t()
// ------------------------------------------------------------
window.NUCtoolI18n = {
  t,
  setLanguage,
  getLanguage,
  getLanguageSetting,
  updateTranslations,
  initI18n,
};

// 兼容你以后在其他 JS 中直接使用 t()
// 例如：window.t("common.save")
window.t = t;