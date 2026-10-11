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
    lightbar: "灯条设置",
    bios: "BIOS配置",
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
    error: "异常",
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

    chargingLimit: "充电限制",
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
  // New Fan
  // ==========================================================
  newfan: {
    title: "新风扇控制",
    subtitle: "EC 独立风扇曲线控制",

    status: "风扇控制状态",

    mode: "模式",
    independent: "独立",
    fan: "风扇",
    duty: "占空比",

    startControl: "启动控制",
    stopControl: "停止控制",
    fanMaxControl: "一键强冷(切换)",
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

    // 电池
    battery: "电池",
    batteryDescription: "调整电池充电相关设置",

    // 电池健康优化器
    batteryHealthOptimizer: "电池健康优化器",
    batteryHealthDescription: "选择电池充电策略",
    batteryHealthFull: "充电达到 100% 电量",
    batteryHealthCustom: "定制",
    batteryHealthBest: "最佳电池健康",

    // 电池充电限制
    chargingLimit: "充电限制",
    currentValue: "当前值",
    write: "写入",
    
    gsc: {
      title: "检查驱动程序是否存在卡顿问题",
      description: "验证图形系统控制器固件接口驱动程序版本",
      installed: "已安装",
      recommended: "推荐版本",
      howToFix: "如何修复",
      ok: "驱动正常",
      warning: "可能会出现音频/视频卡顿",
      notFound: "未找到设备",
      error: "读取驱动版本失败",
    },
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

    brightness: "亮度",
    brightnessDescription: "调整键盘背光亮度",

    mode: "灯光模式",
    modeDescription: "选择静态或彩虹模式",

    static: "静态",
    rainbow: "彩虹",

    color: "键盘颜色",
    colorDescription: "调整键盘 LED 的 RGB 颜色",

    ac: "AC 模式",
    acDescription: "配置接通电源时的键盘灯光",

    dc: "DC 模式",
    dcDescription: "配置使用电池时的键盘灯光",

    reading: "读取中...",
    readFailed: "读取失败",
  },
  // ==========================================================
  // 灯条设置
  // ==========================================================
  lightbar: {
    title: "灯条设置",
    subtitle: "配置设备灯条灯光",
    ac: "AC 模式",
    acDescription: "配置接通电源时的灯条效果",
    dc: "DC 模式",
    dcDescription: "配置使用电池时的灯条效果",
    red: "红色",
    green: "绿色",
    blue: "蓝色",
    effect: "灯效",
    monocolor: "单色",
    rainbow: "彩虹",
    breathing: "闪烁模式(睡眠)",
    breathingDescription: "",
    quickOff: "快速关闭"
  },

  // ==========================================================
  // BIOS 配置
  // ==========================================================
  bios: {
    eyebrow: "NUCTOOL / FIRMWARE",
    title: "BIOS 配置",
    subtitle: "读取并查看 BIOS NVRAM 配置数据。",

    readBios: "读取 BIOS",
    exportNvram: "导出 NVRAM",

    readOnlyTitle: "只读模式",
    readOnlyDescription:
      "此处的修改仅作为本地预览，不会写入 BIOS 或 NVRAM。",

    statusReady: "就绪。点击“读取 BIOS”以获取配置数据。",

    totalEntries: "配置项总数",
    totalEntriesDescription: "已解析的配置记录",

    entriesWithOptions: "包含可选值",
    entriesWithOptionsDescription: "具有可选值的配置项",

    modifiedLocally: "本地已修改",
    modifiedLocallyDescription: "尚未应用的预览修改",

    visibleEntries: "当前显示",
    visibleEntriesDescription: "符合当前筛选条件的配置项",

    searchConfiguration: "搜索配置项",
    searchPlaceholder: "搜索问题、Token、偏移量、数值...",
    clearSearch: "清除搜索",

    entryType: "配置项类型",
    filterAll: "全部配置项",
    filterOptions: "包含可选值",
    filterNoOptions: "不包含可选值",
    filterModified: "本地已修改",

    resetEdits: "重置修改",
    exportRecords: "导出记录",

    noConfigurationLoaded: "尚未加载配置",
    selectEntryHint: "选择一个配置项以查看详细信息",

    columnIndex: "序号",
    columnQuestion: "配置项",
    columnOffset: "偏移量",
    columnWidth: "宽度",
    columnCurrentValue: "当前值",
    columnState: "状态",

    noBiosData: "尚未加载 BIOS 数据",
    noBiosDataDescription: "点击“读取 BIOS”以获取当前 NVRAM 配置。",

    recordCountZero: "0 条记录",
    recordCount: "{count} 条记录",
    showAllEntries: "显示全部配置项",

    configuration: "CONFIGURATION",
    entryDetails: "配置项详情",
    noSelection: "未选择配置项",
    detailEmpty: "从表格中选择一个配置项，以查看其元数据和本地编辑选项。",

    modifiedBadge: "本地已修改",

    offset: "偏移量",
    width: "宽度",
    biosDefault: "BIOS 默认值",

    currentValuePreview: "当前值预览",
    editorHint: "修改仅影响本地预览。",

    restoreOriginalValue: "恢复原始值",
    copyDetails: "复制详情",

    availableOptions: "可用选项",
    noOptionList: "没有可用的选项列表。",

    rawEntryText: "原始配置项文本",

    sourceData: "SOURCE DATA",
    nvramText: "NVRAM 文本",
    showSource: "显示源数据",
    hideSource: "隐藏源数据",
    noSourceLoaded: "尚未加载源数据",
    copySource: "复制源数据",

    stateNormal: "未修改",
    stateModified: "本地已修改",
    stateUnknown: "未知",

    searchNoResults: "没有符合条件的配置项",
    searchResults: "找到 {count} 条配置项",
    readSuccess: "BIOS 配置读取完成",
    readFailed: "读取 BIOS 配置失败",
    exportSuccess: "导出成功",
    exportFailed: "导出失败",
    copySuccess: "复制成功",
    copyFailed: "复制失败",
    resetSuccess: "已恢复原始值",
    
    writeBios: "写入 BIOS",
    writeWarningTitle: "BIOS 写入风险",
    writeWarningDescription:
      "写入 BIOS NVRAM 可能导致系统无法启动、配置异常或硬件功能失效。请确认配置内容正确，并确保设备供电稳定。写入操作仅应在明确了解风险后执行。使用前请自行备份 NVRAM，本程序不提供安全备份功能。",

    passwordPrompt: "请输入 BIOS 管理员密码：",
    passwordRequired: "BIOS 管理员密码不能为空。",
    writeConfirmTitle: "确认写入 BIOS",
    writeConfirmDescription:
      "此操作将修改 BIOS NVRAM，可能导致系统无法启动或硬件功能异常。确定继续吗？",
    writeSuccess: "BIOS NVRAM 写入命令执行完成",
    writeFailed: "BIOS NVRAM 写入失败",
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
    secondaryPriority: "分风扇优先",
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