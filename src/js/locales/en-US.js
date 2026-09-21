const enUS = {
  // ==========================================================
  // App
  // ==========================================================
  app: {
    name: "NUCtool",
    systemControl: "System Control",
  },

  // ==========================================================
  // Navigation
  // ==========================================================
  nav: {
    monitor: "Performance",
    tuning: "Performance Tuning",
    fan: "Fan Control",
    display: "Display",
    keyboard: "Keyboard",
    settings: "Settings",
  },

  // ==========================================================
  // Status
  // ==========================================================
  status: {
    connecting: "Connecting...",
    connected: "Connected",
    disconnected: "Connection failed",
    running: "Running",
    stopped: "Stopped",
    checking: "Checking...",
  },

  // ==========================================================
  // Performance Monitor
  // ==========================================================
  monitor: {
    title: "Performance Monitor",
    subtitle: "View system temperature and fan status in real time",

    cpuTemperature: "CPU Temperature",
    gpuTemperature: "GPU Temperature",

    mainFan: "Main Fan Speed",
    secondaryFan: "Secondary Fan Speed",

    systemPower: "System Power",
    batteryHealth: "Battery Health",
  },

  // ==========================================================
  // Performance Tuning
  // ==========================================================
  tuning: {
    title: "Performance Tuning",
    subtitle: "Adjust device performance",

    modeSwitch: "Performance Mode",
    modeDescription:
      "Select a performance configuration for the current usage scenario. The displayed mode is for reference only; the selected mode takes precedence.",

    default: "Default",
    defaultDescription: "Do not modify the mode",

    powerSaving: "Power Saving",
    powerSavingDescription:
      "Reduce power consumption and extend battery life",

    balanced: "Balanced",
    balancedDescription:
      "Balance performance and power consumption",

    performance: "Performance",
    performanceDescription: "Provide higher performance",

    benchmark: "Benchmark",
    benchmarkDescription: "Higher performance for benchmarking",

    powerPlan: "Power Plan",
    powerPlanDescription:
      "Select a Windows power plan. The displayed plan is for reference only; the selected plan and mode take precedence. Mode switching may override it.",

    powerPlanDefault: "Default",
    powerPlanDefaultDescription:
      "Do not modify the power plan",

    powerSavingPlan: "Power Saver",
    powerSavingPlanDescription:
      "Reduce power consumption and extend battery life",

    balancedPlan: "Balanced",
    balancedPlanDescription:
      "Balance performance and power consumption",

    highPerformancePlan: "High Performance",
    highPerformancePlanDescription:
      "Provide higher performance",

    benchmarkHighPerformancePlan:
      "Benchmark High Performance",

    benchmarkHighPerformancePlanDescription:
      "For benchmark testing",

    powerLimit: "Power Limits",
    powerLimitDescription:
      "Adjust processor and graphics power limits",

    refreshConfig: "Read Configuration",
    currentValue: "Current Value",
    write: "Apply",

    cpu: "CPU",
    gpuIntelArc: "GPU (Intel ARC)",
    battery: "Battery",

    chargingLimit: "Charging Limit",
  },

  // ==========================================================
  // Fan
  // ==========================================================
  fan: {
    title: "Fan Control",
    subtitle: "Adjust fan curves according to temperature",

    startControl: "Start Control",
    stopControl: "Stop Control",

    loadConfig: "Load Configuration",
    saveConfig: "Save Configuration",

    mainCurve: "Main Fan Curve",
    secondaryCurve: "Secondary Fan Curve",

    dragHint: "Drag points to adjust",
  },

  // ==========================================================
  // Display
  // ==========================================================
  display: {
    title: "Display Settings",
    subtitle: "Adjust display mode",

    displayMode: "Display Mode",
    displayDescription:
      "Select a display configuration for the current usage scenario. The displayed mode is for reference only; the selected mode takes precedence.",

    default: "Default",
    defaultDescription:
      "Do not modify by default; select again to disable",

    standard: "Standard",
    standardDescription: "Standard display mode",

    gaming: "Gaming",
    gamingDescription: "Gaming display mode",

    video: "Video",
    videoDescription: "Video display mode",

    reading: "Reading",
    readingDescription: "Reading display mode",

    custom: "Custom",
    customDescription: "Custom display mode",
  },

  // ==========================================================
  // Keyboard
  // ==========================================================
  keyboard: {
    title: "Keyboard Settings",
    subtitle: "Configure keyboard lighting",

    keyboardLed: "Keyboard LED",
    keyboardLedDescription:
      "Control keyboard LED lighting",

    ledEnabled: "Keyboard LED is on",
    ledDisabled: "Keyboard LED is off",

    reading: "Reading...",
    readFailed: "Read failed",
  },

  // ==========================================================
  // Settings
  // ==========================================================
  settings: {
    title: "Settings",
    subtitle: "Configure NUCtool",

    autostartTitle: "Startup & Automatic Control",
    autostartDescription:
      "Automatically start fan control when NUCtool starts with Windows",

    autostartChecking: "Checking...",

    startup: "Start with Windows",
    startupDescription:
      "Automatically run NUCtool after signing in to Windows",

    startupEnabledDescription:
      "Automatically run NUCtool after signing in to Windows",

    startupDisabledDescription:
      "NUCtool will not start automatically with Windows",

    enabled: "Enabled",
    disabled: "Disabled",

    readFailed: "Read failed",

    languageTitle: "语言 / Language / 言語",
    languageDescription:
      "Select the NUCtool display language",

    languageAuto: "Follow System",
    languageChinese: "简体中文",
    languageEnglish: "English",
    languageJapanese: "日本語",
  },

  // ==========================================================
  // Common
  // ==========================================================
  common: {
    cpu: "CPU",
    gpu: "GPU",
    battery: "Battery",

    watts: "W",
    percent: "%",
    celsius: "°C",
    rpm: "RPM",

    saved: "Saved ✓",
  },
};

export default enUS;