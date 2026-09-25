const jaJP = {
  // ==========================================================
  // アプリ
  // ==========================================================
  app: {
    name: "NUCtool",
    systemControl: "System Control",
  },

  // ==========================================================
  // ナビゲーション
  // ==========================================================
  nav: {
    monitor: "パフォーマンス監視",
    tuning: "パフォーマンス調整",
    fan: "ファン制御",
    system: "システム設定",
    display: "ディスプレイ設定",
    keyboard: "キーボード設定",
    lightbar: "ライト設定",
    settings: "設定",
  },

  // ==========================================================
  // ステータス
  // ==========================================================
  status: {
    connecting: "接続中...",
    connected: "接続済み",
    disconnected: "接続に失敗しました",
    running: "実行中",
    stopped: "停止中",
    checking: "確認中...",
  },

  // ==========================================================
  // パフォーマンス監視
  // ==========================================================
  monitor: {
    title: "パフォーマンス監視",
    subtitle: "システム温度とファンの状態をリアルタイムで確認",

    cpuTemperature: "CPU 温度",
    gpuTemperature: "GPU 温度",

    mainFan: "Main ファン回転数",
    secondaryFan: "Secondary ファン回転数",

    systemPower: "システム消費電力",
    batteryHealth: "バッテリー健康度",
  },

  // ==========================================================
  // パフォーマンス調整
  // ==========================================================
  tuning: {
    title: "パフォーマンス調整",
    subtitle: "デバイスのパフォーマンスを調整します",

    modeSwitch: "モード切替",
    modeDescription:
      "現在の使用環境に適したパフォーマンス設定を選択します。表示されているモードは参考値であり、選択したモードが適用されます。",

    default: "デフォルト",
    defaultDescription: "モードを変更しない",

    powerSaving: "省電力モード",
    powerSavingDescription:
      "消費電力を抑えてバッテリー駆動時間を延長",

    balanced: "バランスモード",
    balancedDescription:
      "パフォーマンスと消費電力のバランスを調整",

    performance: "パフォーマンスモード",
    performanceDescription:
      "より高いパフォーマンスを提供",

    benchmark: "ベンチマークモード",
    benchmarkDescription:
      "ベンチマーク向けの高性能設定",

    powerPlan: "電源プラン",
    powerPlanDescription:
      "Windows の電源プランを選択します。表示されているプランは参考値であり、選択したプランとモードが適用されます。モード切替によって上書きされる場合があります。",

    powerPlanDefault: "デフォルト",
    powerPlanDefaultDescription:
      "電源プランを変更しない",

    powerSavingPlan: "省電力",
    powerSavingPlanDescription:
      "消費電力を抑えてバッテリー駆動時間を延長",

    balancedPlan: "バランス",
    balancedPlanDescription:
      "パフォーマンスと消費電力のバランスを調整",

    highPerformancePlan: "高パフォーマンス",
    highPerformancePlanDescription:
      "より高いパフォーマンスを提供",

    benchmarkHighPerformancePlan:
      "ベンチマーク高パフォーマンス",

    benchmarkHighPerformancePlanDescription:
      "ベンチマークテスト用",

    powerLimit: "電力制限",
    powerLimitDescription:
      "CPU と GPU の電力制限を調整します",

    refreshConfig: "設定を読み込む",
    currentValue: "現在値",
    write: "適用",

    cpu: "CPU",
    gpuIntelArc: "GPU (Intel ARC)",
    battery: "Battery",

    chargingLimit: "充電上限",
  },

  // ==========================================================
  // ファン
  // ==========================================================
  fan: {
    title: "ファン制御",
    subtitle: "温度に応じてファンカーブを調整します",

    startControl: "制御を開始",
    stopControl: "制御を停止",

    loadConfig: "設定を読み込む",
    saveConfig: "設定を保存",

    mainCurve: "Main ファンカーブ",
    secondaryCurve: "Secondary ファンカーブ",

    dragHint: "ノードをドラッグして調整",
  },

  system: {
    title: "システム設定",
    subtitle: "システム関連の設定を調整します",

    // バッテリー
    battery: "バッテリー",
    batteryDescription: "バッテリーの充電設定を調整します",

    // バッテリー健康最適化
    batteryHealthOptimizer: "バッテリー健康最適化",
    batteryHealthDescription: "充電方式を選択します",
    batteryHealthFull: "100%まで充電",
    batteryHealthCustom: "カスタム",
    batteryHealthBest: "バッテリー健康を優先",

    // 充電上限
    chargingLimit: "充電上限",
    currentValue: "現在値",
    write: "書き込み",
    
    gsc: {
      title: "カクつき防止ドライバーの確認",
      description: "グラフィックスシステムコントローラー・ファームウェアインターフェースドライバーのバージョンを確認します",
      installed: "インストール済み",
      recommended: "推奨",
      howToFix: "修正方法",
      ok: "ドライバーは正常です",
      warning: "音声や映像にカクつきが発生する可能性があります",
      notFound: "デバイスが見つかりません",
      error: "ドライバーのバージョンの読み取りに失敗しました",
    }
  },

  // ==========================================================
  // ディスプレイ
  // ==========================================================
  display: {
    title: "ディスプレイ設定",
    subtitle: "ディスプレイモードを調整します",

    displayMode: "ディスプレイモード",
    displayDescription:
      "現在の使用環境に適したディスプレイ設定を選択します。表示されているモードは参考値であり、選択したモードが適用されます。",

    default: "デフォルト",
    defaultDescription:
      "デフォルトでは変更しません。もう一度選択すると無効になります",

    standard: "Standard",
    standardDescription: "標準表示モード",

    gaming: "Gaming",
    gamingDescription: "ゲーム表示モード",

    video: "Video",
    videoDescription: "動画表示モード",

    reading: "Reading",
    readingDescription: "読書表示モード",

    custom: "Custom",
    customDescription: "カスタム表示モード",
  },
  lightbar: {
    title: "ライトバー設定",
    subtitle: "デバイスのライトバーを設定",
    ac: "ACモード",
    acDescription: "電源接続時のライトバー効果を設定",
    dc: "DCモード",
    dcDescription: "バッテリー使用時のライトバー効果を設定",
    red: "赤",
    green: "緑",
    blue: "青",
    effect: "ライト効果",
    monocolor: "単色",
    rainbow: "レインボー",
    breathing: "ブリージングモード（スリープ）",
    breathingDescription: "",
    quickOff: "クイックオフ"
  },
  // ==========================================================
  // キーボード
  // ==========================================================
  keyboard: {
    title: "キーボード設定",
    subtitle: "キーボードライトを設定します",

    keyboardLed: "キーボード LED",
    keyboardLedDescription:
      "キーボード LED ライトを制御します",

    ledEnabled: "キーボード LED はオンです",
    ledDisabled: "キーボード LED はオフです",

    reading: "読み込み中...",
    readFailed: "読み込みに失敗しました",
  },

  // ==========================================================
  // 設定
  // ==========================================================
  settings: {
    title: "設定",
    subtitle: "NUCtool を設定します",

    autostartTitle: "スタートアップと自動制御",
    autostartDescription:
      "Windows 起動時に NUCtool のファン制御を自動的に開始します",

    autostartChecking: "確認中...",

    startup: "Windows 起動時に実行",
    startupDescription:
      "Windows へのサインイン後に NUCtool を自動的に起動します",

    startupEnabledDescription:
      "Windows へのサインイン後に NUCtool を自動的に起動します",

    startupDisabledDescription:
      "Windows 起動時に NUCtool を自動的に起動しません",

    enabled: "有効",
    disabled: "無効",

    readFailed: "読み込みに失敗しました",

    languageTitle: "语言 / Language / Язык / 言語",
    languageDescription:
      "NUCtool の表示言語を選択します",

    languageAuto: "システムに合わせる",
    languageChinese: "简体中文",
    languageEnglish: "English",
    languageJapanese: "日本語",
    languageRussian: "Русский",

    fanModeTitle: "ファンモード",
    fanModeDescription: "2つのファンの制御方法を設定します",
    independent: "独立",
    mainPriority: "メインファン優先",
    secondaryPriority: "個別ファン優先"
  
  },

  // ==========================================================
  // 共通
  // ==========================================================
  common: {
    cpu: "CPU",
    gpu: "GPU",
    battery: "Battery",

    watts: "W",
    percent: "%",
    celsius: "°C",
    rpm: "RPM",

    saved: "保存しました ✓",
  }
};

export default jaJP;