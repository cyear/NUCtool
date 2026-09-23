const zhCN = {
  // ==========================================================
  // 应用
  // ==========================================================
  app: {
    name: "NUCtool",
    systemControl: "System Control",
  },

  // ==========================================================
  // 导航
  // ==========================================================
  nav: {
    monitor: "性能监控",
    tuning: "性能调优",
    fan: "风扇控制",
    display: "显示设置",
    system: "系统配置",
    keyboard: "键盘设置",
    settings: "设置",
  },

  // ==========================================================
  // 状态
  // ==========================================================
  status: {
    connecting: "连接中...",
    connected: "已连接",
    disconnected: "连接失败",
    running: "运行中",
    stopped: "未运行",
    checking: "检查中...",
  },

  // ==========================================================
  // 性能监控
  // ==========================================================
  monitor: {
    title: "性能监控",
    subtitle: "实时查看系统温度与风扇状态",

    cpuTemperature: "CPU 温度",
    gpuTemperature: "GPU 温度",

    mainFan: "Main 风扇转速",
    secondaryFan: "Secondary 风扇转速",

    systemPower: "平台功耗",
    batteryHealth: "BAT健康度",
  },

  // ==========================================================
  // 性能调优
  // ==========================================================
  tuning: {
    title: "性能调优",
    subtitle: "对设备性能释放进行调整",

    modeSwitch: "模式切换",
    modeDescription:
      "选择适合当前使用场景的性能配置，显示的模式仅为显示，请以选择后为准",

    default: "默认",
    defaultDescription: "不修改模式",

    powerSaving: "省电模式",
    powerSavingDescription: "降低功耗，延长续航",

    balanced: "平衡模式",
    balancedDescription: "平衡性能与功耗",

    performance: "性能模式",
    performanceDescription: "提供更高的性能",

    benchmark: "基准模式",
    benchmarkDescription: "更强大的性能",

    powerPlan: "电源计划",
    powerPlanDescription:
      "选择 Windows 电源计划，显示的计划仅为显示，请以选择后以及模式为准(模式切换会覆盖)",

    powerPlanDefault: "默认",
    powerPlanDefaultDescription: "不修改电源计划",

    powerSavingPlan: "节能",
    powerSavingPlanDescription: "降低功耗，延长续航",

    balancedPlan: "平衡",
    balancedPlanDescription: "平衡性能与功耗",

    highPerformancePlan: "高性能",
    highPerformancePlanDescription: "提供更高的性能",

    benchmarkHighPerformancePlan: "基准高性能",
    benchmarkHighPerformancePlanDescription: "基准测试使用",

    powerLimit: "功耗限制",
    powerLimitDescription: "调整处理器与显卡的功耗限制",

    refreshConfig: "读取配置",
    currentValue: "当前值",
    write: "写入",

    cpu: "CPU",
    gpuIntelArc: "GPU(Intel ARC)",
    battery: "Battery",

    chargingLimit: "Charging limit",
  },

  // ==========================================================
  // 风扇控制
  // ==========================================================
  fan: {
    title: "风扇控制",
    subtitle: "根据温度调整风扇曲线",

    startControl: "启动控制",
    stopControl: "停止控制",

    loadConfig: "加载配置",
    saveConfig: "保存配置",

    mainCurve: "Main 风扇曲线",
    secondaryCurve: "Secondary 风扇曲线",

    dragHint: "拖动节点调整",
  },

  // ==========================================================
  // 系统配置
  // ==========================================================
  system: {
    title: "系统配置",
    subtitle: "调整系统相关配置",

    battery: "电池",
    batteryDescription: "调整电池充电相关设置",

    chargingLimit: "充电限制",
    currentValue: "当前值",

    write: "写入",
  },

  // ==========================================================
  // 显示设置
  // ==========================================================
  display: {
    title: "显示设置",
    subtitle: "调整设备显示模式",

    displayMode: "显示模式",
    displayDescription:
      "选择适合当前使用场景的显示配置，显示的模式仅为显示，请以选择后为准",

    default: "默认",
    defaultDescription: "默认不修改, 再次选择关闭",

    standard: "Standard",
    standardDescription: "标准显示模式",

    gaming: "Gaming",
    gamingDescription: "游戏显示模式",

    video: "Video",
    videoDescription: "视频显示模式",

    reading: "Reading",
    readingDescription: "阅读显示模式",

    custom: "Custom",
    customDescription: "自定义显示模式",
  },

  // ==========================================================
  // 键盘设置
  // ==========================================================
  keyboard: {
    title: "键盘设置",
    subtitle: "配置键盘灯光",

    keyboardLed: "键盘 LED 灯",
    keyboardLedDescription: "控制键盘 LED 灯",

    ledEnabled: "键盘 LED 灯已开启",
    ledDisabled: "键盘 LED 灯已关闭",

    reading: "读取中...",
    readFailed: "读取失败",
  },

  // ==========================================================
  // 设置
  // ==========================================================
  settings: {
    title: "设置",
    subtitle: "配置 NUCtool",

    autostartTitle: "开机自启动 & 自动控制",
    autostartDescription: "开机后NUCtool自动启动风扇控制",

    autostartChecking: "检查中...",

    startup: "开机启动",
    startupDescription: "登录Windows后自动运行NUCtool",

    startupEnabledDescription:
      "登录 Windows 后自动运行 NUCtool",

    startupDisabledDescription:
      "开机后不会自动运行 NUCtool",

    enabled: "已启用",
    disabled: "未启用",

    readFailed: "读取失败",

    languageTitle: "语言 / Language / Язык / 言語",
    languageDescription: "选择 NUCtool 的显示语言",

    languageAuto: "跟随系统",
    languageChinese: "简体中文",
    languageEnglish: "English",
    languageJapanese: "日本語",
    languageRussian: "Русский",

    fanModeTitle: "风扇模式",
    fanModeDescription: "设置两个风扇的控制方式",
    independent: "独立",
    mainPriority: "主风扇优先",
    splitPriority: "分风扇优先",
  },

  // ==========================================================
  // 通用
  // ==========================================================
  common: {
    cpu: "CPU",
    gpu: "GPU",
    battery: "Battery",

    watts: "W",
    percent: "%",
    celsius: "°C",
    rpm: "RPM",

    saved: "已保存 ✓",
  },
};

export default zhCN;