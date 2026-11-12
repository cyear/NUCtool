const enUS = {
  // ==========================================================
  // App
  // ==========================================================
  app: {
    name: "NUCtool",
    systemControl: "System Control",
  },

  // ==========================================================
  // App feartures status
  // ==========================================================
  features: {
    beta: "Beta",
    old: "Old",
  },

  // ==========================================================
  // Navigation
  // ==========================================================
  nav: {
    monitor: "Performance",
    tuning: "Performance Tuning",
    fan: "Fan Control",
    system: "System Config",
    display: "Display",
    audio: "Audio Tuning",
    keyboard: "Keyboard",
    keyboardHid: "Keyboard Lighting",
    lightbar: "Lightbar",
    bios: "BIOS Configuration",
    ecDebug: "EC Debug",
    settings: "Settings",
  },

  ecDebug: {
    description: "Read or write a single register via EC or WMI",
    register: "Register Access",
    channel: "Channel",
    address: "Address (hex)",
    value: "Value to write (hex)",
    unlock: "Unlock writes",
    lock: "Lock writes",
    read: "Read",
    write: "Write",
    writeHint: "Check the channel, address, and value before writing.",
    result: "Result",
    reading: "{{channel}} · {{address}} · Byte {{byte}} ({{decimal}})",
    raw: "Raw WMI value {{value}}",
    readingInProgress: "Reading…",
    writing: "Writing…",
    written: "Write request completed",
    failed: "{{channel}} operation failed. {{error}}",
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
    error: "Error",
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

    powerPlanOther: "Other",
    powerPlanOtherDescription:
      "Unlisted plan (not selectable)",

    powerSavingPlan: "Power Saver",
    powerSavingPlanDescription:
      "Reduce power consumption and extend battery life",

    balancedPlan: "Balanced",
    balancedPlanDescription:
      "Balance performance and power consumption",

    highPerformancePlan: "High Performance",
    highPerformancePlanDescription:
      "Provide higher performance",

    ultimatePerformancePlan:
      "Ultimate Performance",

    ultimatePerformancePlanDescription:
      "Windows Ultimate Performance power plan",

    powerLimit: "Power Limits",
    powerLimitDescription:
      "Adjust processor and graphics power limits",

    refreshConfig: "Read Configuration",
    currentValue: "Current Value",
    write: "Apply",

    cpu: "CPU",
    gpuIntelArc: "GPU (AC ARC)",
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
  // New Fan
  // ==========================================================
  newfan: {
    title: "Fan Control (Beta)",
    subtitle: "EC independent fan curve control",

    status: "Fan Control Status",

    mode: "Mode",
    independent: "Independent",
    fan: "Fan",
    duty: "Duty",

    startControl: "Start Control",
    stopControl: "Stop Control",
    fanMaxControl: "Max Fan(Toggle)",
    loadConfig: "Load Configuration",
    saveConfig: "Save Configuration",

    mainCurve: "Main Fan Curve",
    secondaryCurve: "Secondary Fan Curve",

    dragHint: "Drag nodes to adjust",
  },
  system: {
    batteryDetails: {
      title: "Battery details",
      fullOver24Hours: "Fully charged for over 24 hours",
      ermThresholdReached: "ERM threshold reached",
      windowsPercent: "Windows battery level",
      designCapacity: "Design capacity",
      fullChargeCapacity: "Full charge capacity",
      remainingCapacity: "Remaining capacity",
      designVoltage: "Design voltage",
      voltage: "Live voltage",
      batteryCurrent: "Battery current",
      cycleCount: "Battery cycle count",
      warning: "Battery warning",
      chargeLimit: "Charge limit",
      yes: "Yes",
      no: "No",
      normal: "Normal",
      warningActive: "Warning",
      capacityEstimate: "mAh estimated using design voltage",
    },
    title: "System Configuration",
    subtitle: "Adjust system-related settings",

    // Battery
    battery: "Battery",
    batteryDescription: "Adjust battery charging settings",

    // Battery Health Optimizer
    batteryHealthOptimizer: "Battery Health Optimizer",
    batteryHealthDescription: "Choose a charging strategy",
    batteryHealthFull: "Charge to 100%",
    batteryHealthCustom: "Custom",
    batteryHealthBest: "Best Battery Health",

    // Battery Charging Limit
    chargingLimit: "Charging Limit",
    currentValue: "Current Value",
    write: "Apply",
    
    // ==========================================================
    // GSC
    // ==========================================================
    gsc: {
      title: "Stutter-free Driver Check",
      description: "Verifies the Graphics System Controller Firmware Interface driver version",
      installed: "Installed",
      recommended: "Recommended",
      howToFix: "How to fix",
      ok: "Driver is fine",
      warning: "Audio/video stutter may occur",
      notFound: "Device not found",
      error: "Failed to read driver version",
    },
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
    title: "Keyboard",
    subtitle: "Configure keyboard lighting",

    keyboardLed: "Keyboard LED",
    keyboardLedDescription: "Control keyboard LED lighting",

    ledEnabled: "Keyboard LED is enabled",
    ledDisabled: "Keyboard LED is disabled",

    brightness: "Brightness",
    brightnessDescription: "Adjust keyboard backlight brightness",

    mode: "Lighting Mode",
    modeDescription: "Choose between static and rainbow lighting",

    static: "Static",
    rainbow: "Rainbow",

    color: "Keyboard Color",
    colorDescription: "Adjust the RGB color of the keyboard LED",

    ac: "AC Mode",
    acDescription: "Configure keyboard lighting when connected to AC power",

    dc: "DC Mode",
    dcDescription: "Configure keyboard lighting when running on battery",

    reading: "Reading...",
    readFailed: "Failed to read",
  },
  keyboardHid: {
    title: "Keyboard Lighting (HID)", subtitle: "Independent HID control · device status refreshes every 4 seconds", device: "Keyboard Controller",
    firmware: "Firmware", layout: "Layout", refresh: "Refresh", effectNow: "Current Effect", refreshNow: "Read Now",
    powerTitle: "Lighting Power", power: "Keyboard Light", powerHint: "Send the switch command directly to the keyboard controller", brightness: "Brightness",
    timeout: "Timed Shutdown", timeoutEnabled: "Enable firmware timed shutdown", rawTime: "Firmware time value", timeoutHint: "The protocol confirms a raw time value from 0–255; its unit is defined by firmware.",
    applyTimeout: "Write Shutdown Setting", builtinTitle: "Built-in Effects", effect: "Effect", direction: "Direction", speed: "Speed",
    effectBrightness: "Brightness", palette: "Palette (up to 7 colors)", paletteHint: "Used by multicolor dynamic effects", save: "Save to device profile",
    writeBuiltin: "Write Built-in Effect", extendedTitle: "Extended Effect", matrixHint: "Select an individual key to edit its color",
    frameColor: "Selected Key Color", frameBrightness: "Frame Brightness", extendedHint: "Set colors for each of the 126 key slots, or fill every key at once.", writeExtended: "Write Extended Effect",
    fillKeys: "Fill All Keys",
  },
  lightbar: {
    title: "Light Bar Settings",
    subtitle: "Configure the device light bar",
    ac: "AC Mode",
    acDescription: "Configure the light bar effect when connected to power",
    dc: "DC Mode",
    dcDescription: "Configure the light bar effect when running on battery",
    red: "Red",
    green: "Green",
    blue: "Blue",
    effect: "Effect",
    monocolor: "Monochrome",
    rainbow: "Rainbow",
    breathing: "Breathing Mode (Sleep)",
    breathingDescription: "",
    quickOff: "Quick Off"
  },
  // ==========================================================
  // BIOS Configuration
  // ==========================================================
  bios: {
    eyebrow: "NUCTOOL / FIRMWARE",
    title: "BIOS Configuration",
    subtitle: "Read and view BIOS NVRAM configuration data.",

    readBios: "Read BIOS",
    exportNvram: "Export NVRAM",

    readOnlyTitle: "Read-only Mode",
    readOnlyDescription:
      "Changes made here are for local preview only and will not be written to BIOS or NVRAM.",

    statusReady: "Ready. Click \"Read BIOS\" to retrieve configuration data.",

    totalEntries: "Total Entries",
    totalEntriesDescription: "Parsed configuration entries",

    entriesWithOptions: "Entries with Options",
    entriesWithOptionsDescription: "Configuration entries with available options",

    modifiedLocally: "Locally Modified",
    modifiedLocallyDescription: "Preview changes not yet applied",

    visibleEntries: "Visible Entries",
    visibleEntriesDescription: "Entries matching the current filters",

    searchConfiguration: "Search Configuration",
    searchPlaceholder: "Search questions, tokens, offsets, values...",
    clearSearch: "Clear Search",

    entryType: "Entry Type",
    filterAll: "All Entries",
    filterOptions: "With Options",
    filterNoOptions: "Without Options",
    filterModified: "Locally Modified",

    resetEdits: "Reset Changes",
    exportRecords: "Export Entries",

    noConfigurationLoaded: "No Configuration Loaded",
    selectEntryHint: "Select an entry to view its details",

    columnIndex: "Index",
    columnQuestion: "Configuration Item",
    columnOffset: "Offset",
    columnWidth: "Width",
    columnCurrentValue: "Current Value",
    columnState: "Status",

    noBiosData: "No BIOS Data Loaded",
    noBiosDataDescription: "Click \"Read BIOS\" to retrieve the current NVRAM configuration.",

    recordCountZero: "0 entries",
    recordCount: "{count} entries",
    showAllEntries: "Show All Entries",

    configuration: "CONFIGURATION",
    entryDetails: "Entry Details",
    noSelection: "No Entry Selected",
    detailEmpty: "Select an entry from the table to view its metadata and local editing options.",

    modifiedBadge: "Locally Modified",

    offset: "Offset",
    width: "Width",
    biosDefault: "BIOS Default",

    currentValuePreview: "Current Value Preview",
    editorHint: "Changes affect the local preview only.",

    restoreOriginalValue: "Restore Original Value",
    copyDetails: "Copy Details",

    availableOptions: "Available Options",
    noOptionList: "No options are available for this entry.",

    rawEntryText: "Raw Entry Text",

    sourceData: "SOURCE DATA",
    nvramText: "NVRAM Text",
    showSource: "Show Source Data",
    hideSource: "Hide Source Data",
    noSourceLoaded: "No Source Data Loaded",
    copySource: "Copy Source Data",

    stateNormal: "Unmodified",
    stateModified: "Locally Modified",
    stateUnknown: "Unknown",

    searchNoResults: "No matching entries found",
    searchResults: "Found {count} entries",
    readSuccess: "BIOS configuration loaded successfully",
    readFailed: "Failed to read BIOS configuration",
    exportSuccess: "Export successful",
    exportFailed: "Export failed",
    copySuccess: "Copied successfully",
    copyFailed: "Copy failed",
    resetSuccess: "Original value restored",
    writeBios: "Write BIOS",

    writeWarningTitle: "BIOS Writing Risk",
    writeWarningDescription:
      "Writing BIOS NVRAM may prevent the system from booting, cause configuration issues, or disable hardware functions. Verify that the configuration is correct and ensure a stable power supply. Only proceed if you fully understand the risks. Back up your NVRAM before use. This application does not provide a safe NVRAM backup mechanism.",

    passwordPrompt: "Enter the BIOS administrator password:",
    passwordRequired: "The BIOS administrator password cannot be empty.",
    writeConfirmTitle: "Confirm BIOS Write",
    writeConfirmDescription:
      "This operation will modify BIOS NVRAM and may prevent the system from booting or cause hardware malfunctions. Do you want to continue?",
    writeSuccess: "BIOS NVRAM write command completed",
    writeFailed: "Failed to write BIOS NVRAM",

    writing: "Writing BIOS NVRAM...",
    writeCancelled: "BIOS write cancelled",
  },
  // ==========================================================
  // Settings
  // ==========================================================
  settings: {
    title: "Settings",
    subtitle: "Configure NUCtool",

    autostartTitle: "Startup",
    autostartDescription:
      "Automatically run NUCtool after signing in to Windows",
    autoFanTitle: "Fan control at startup",
    autoFanDescription: "Fan control mode used when NUCtool starts with Windows or is launched manually with --fan-control",
    autoFanOff: "Off",
    autoFanLegacy: "Legacy fan control",
    autoFanNew: "New fan control",
    autoFanSaveFailed: "Failed to save the fan startup setting",

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

    languageTitle: "语言 / Language / Язык / 言語",
    languageDescription:
      "Select the NUCtool display language",

    languageAuto: "Follow System",
    languageChinese: "简体中文",
    languageEnglish: "English",
    languageJapanese: "日本語",
    languageRussian: "Русский",

    fanModeTitle: "Fan Mode",
    fanModeDescription: "Configure how the two fans are controlled",
    independent: "Independent",
    mainPriority: "Main Fan Priority",
    secondaryPriority: "Secondary Fan Priority",

  },

  // ==========================================================
  // Common
  // ==========================================================
  audio: {
    title: "Audio Tuning",
    description: "Adjust playback and recording effects, equalizer, and volume",
    loading: "Loading...",
    refresh: "Refresh",
    ready: "Ready",
    unavailable: "Unavailable",
    partial: "Partially Available",
    apoUnavailable: "Audio processing is unavailable; system volume can still be adjusted:",
    eqPresetUnavailable: "This playback device did not provide all equalizer band values, so presets are unavailable.",
    eqPresetEnableFirst: "Turn on the equalizer before applying a preset.",
    applying: "Applying...",
    saved: "Applied",
    outputDevice: "Playback Device",
    inputDevice: "Recording Device",
    noDevices: "No devices found",
    defaultDevice: "Default",
    volume: "Master Volume",
    output: "Output",
    input: "Input",
    enhancement: "Audio Enhancement",
    enhancementDescription: "Enable playback audio processing",
    profile: "Profile",
    defaultProfile: "Default Profile",
    mode: "Sound Mode",
    modeChoose: "Choose a mode",
    modeMusic: "Music",
    modeVoice: "Voice",
    modeMovie: "Movie",
    modeGaming: "Gaming",
    preset: "Equalizer Preset",
    choosePreset: "Choose a preset",
    presetCustom: "Custom",
    presetDefault: "Default",
    presetAcoustic: "Acoustic",
    presetBlues: "Blues",
    presetClassical: "Classical",
    presetCountry: "Country",
    presetDance: "Dance",
    presetElectronic: "Electronic",
    presetHipHop: "Hip-hop",
    presetJazz: "Jazz",
    presetMetal: "Metal",
    presetPop: "Pop",
    presetRock: "Rock",
    profileNamePlaceholder: "New profile name",
    createProfile: "Create Profile",
    deleteProfile: "Delete Profile",
    resetProfile: "Restore Profile Defaults",
    profileNameRequired: "Enter a profile name",
    confirmDeleteProfile: "Delete this audio profile?",
    confirmResetProfile: "Restore the current profile's default audio settings?",
    tone: "Sound Controls",
    toneDescription: "Adjust bass, detail, voice, and surround effects",
    bass: "Bass",
    detail: "Detail",
    voice: "Voice",
    surround: "Surround",
    equalizer: "Equalizer",
    equalizerDescription: "Adjust the 10-band frequency response",
    inputEnhancement: "Microphone Enhancement",
    inputEnhancementDescription: "Enable audio enhancement for the recording device",
    inputMode: "Microphone Mode",
    inputHeadset: "Headset",
    inputConference: "Conference",
    noiseReduction: "Noise Reduction",
    noiseReductionDescription: "Control microphone noise reduction",
  },
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
