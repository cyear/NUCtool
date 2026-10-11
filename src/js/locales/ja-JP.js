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
    bios: "BIOS設定",
    settings: "設定",
  },

  // ==========================================================
  // ステータス
  // ==========================================================
  status: {
    connecting: "接続中...",
    connected: "接続済み",
    disconnected: "接続失敗",
    running: "実行中",
    stopped: "未実行",
    checking: "確認中...",
    error: "異常",
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
  // ==========================================================
  // New Fan
  // ==========================================================
  newfan: {
    subtitle: "EC独立ファンカーブ制御",

    status: "ファン制御状態",

    mode: "モード",
    independent: "独立",
    fan: "ファン",
    duty: "デューティ",

    startControl: "制御を開始",
    stopControl: "制御を停止",
    fanMaxControl: "ワンタッチ強力冷房（切替）",
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
    },
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
    subtitle: "キーボードのライティングを設定",

    keyboardLed: "キーボード LED",
    keyboardLedDescription: "キーボード LED ライティングを制御",

    ledEnabled: "キーボード LED はオンです",
    ledDisabled: "キーボード LED はオフです",

    brightness: "明るさ",
    brightnessDescription: "キーボードバックライトの明るさを調整",

    mode: "ライティングモード",
    modeDescription: "静的またはレインボーライティングを選択",

    static: "静的",
    rainbow: "レインボー",

    color: "キーボードカラー",
    colorDescription: "キーボード LED の RGB カラーを調整",

    ac: "AC モード",
    acDescription: "AC 電源接続時のキーボードライティングを設定",

    dc: "DC モード",
    dcDescription: "バッテリー駆動時のキーボードライティングを設定",

    reading: "読み込み中...",
    readFailed: "読み込みに失敗しました",
  },
  // ==========================================================
  // BIOS 設定
  // ==========================================================
  bios: {
    eyebrow: "NUCTOOL / FIRMWARE",
    title: "BIOS 設定",
    subtitle: "BIOS の NVRAM 設定データを読み取り、表示します。",

    readBios: "BIOS を読み取る",
    exportNvram: "NVRAM をエクスポート",

    readOnlyTitle: "読み取り専用モード",
    readOnlyDescription:
      "ここでの変更はローカルプレビューのみで、BIOS や NVRAM には書き込まれません。",

    statusReady: "準備完了。「BIOS を読み取る」をクリックして設定データを取得してください。",

    totalEntries: "設定項目の総数",
    totalEntriesDescription: "解析済みの設定項目",

    entriesWithOptions: "選択肢あり",
    entriesWithOptionsDescription: "選択可能な値を持つ設定項目",

    modifiedLocally: "ローカルで変更済み",
    modifiedLocallyDescription: "まだ適用されていないプレビューの変更",

    visibleEntries: "表示中の項目",
    visibleEntriesDescription: "現在のフィルター条件に一致する項目",

    searchConfiguration: "設定項目を検索",
    searchPlaceholder: "項目名、トークン、オフセット、値を検索...",
    clearSearch: "検索をクリア",

    entryType: "項目の種類",
    filterAll: "すべての項目",
    filterOptions: "選択肢あり",
    filterNoOptions: "選択肢なし",
    filterModified: "ローカルで変更済み",

    resetEdits: "変更をリセット",
    exportRecords: "項目をエクスポート",

    noConfigurationLoaded: "設定データが読み込まれていません",
    selectEntryHint: "項目を選択して詳細を表示してください",

    columnIndex: "番号",
    columnQuestion: "設定項目",
    columnOffset: "オフセット",
    columnWidth: "幅",
    columnCurrentValue: "現在の値",
    columnState: "状態",

    noBiosData: "BIOS データが読み込まれていません",
    noBiosDataDescription: "「BIOS を読み取る」をクリックして現在の NVRAM 設定を取得してください。",

    recordCountZero: "0 件",
    recordCount: "{count} 件",
    showAllEntries: "すべての項目を表示",

    configuration: "CONFIGURATION",
    entryDetails: "項目の詳細",
    noSelection: "項目が選択されていません",
    detailEmpty: "テーブルから項目を選択すると、メタデータとローカル編集オプションが表示されます。",

    modifiedBadge: "ローカルで変更済み",

    offset: "オフセット",
    width: "幅",
    biosDefault: "BIOS の既定値",

    currentValuePreview: "現在値のプレビュー",
    editorHint: "変更はローカルプレビューにのみ反映されます。",

    restoreOriginalValue: "元の値に戻す",
    copyDetails: "詳細をコピー",

    availableOptions: "利用可能な選択肢",
    noOptionList: "利用可能な選択肢はありません。",

    rawEntryText: "設定項目の生テキスト",

    sourceData: "SOURCE DATA",
    nvramText: "NVRAM テキスト",
    showSource: "ソースデータを表示",
    hideSource: "ソースデータを非表示",
    noSourceLoaded: "ソースデータが読み込まれていません",
    copySource: "ソースデータをコピー",

    stateNormal: "未変更",
    stateModified: "ローカルで変更済み",
    stateUnknown: "不明",

    searchNoResults: "一致する設定項目が見つかりません",
    searchResults: "{count} 件の設定項目が見つかりました",
    readSuccess: "BIOS 設定の読み取りが完了しました",
    readFailed: "BIOS 設定の読み取りに失敗しました",
    exportSuccess: "エクスポートが完了しました",
    exportFailed: "エクスポートに失敗しました",
    copySuccess: "コピーしました",
    copyFailed: "コピーに失敗しました",
    resetSuccess: "元の値に戻しました",
    writeBios: "BIOS に書き込む",

    writeWarningTitle: "BIOS 書き込みのリスク",
    writeWarningDescription:
      "BIOS NVRAM への書き込みにより、システムが起動できなくなったり、設定に異常が発生したり、ハードウェア機能が使用できなくなったりする可能性があります。設定内容が正しいことを確認し、電源が安定していることを確保してください。リスクを十分に理解したうえで実行してください。使用前に NVRAM をバックアップしてください。本アプリには安全な NVRAM バックアップ機能はありません。",

    passwordPrompt: "BIOS 管理者パスワードを入力してください：",
    passwordRequired: "BIOS 管理者パスワードを空欄にすることはできません。",
    writeConfirmTitle: "BIOS 書き込みの確認",
    writeConfirmDescription:
      "この操作により BIOS NVRAM が変更され、システムが起動できなくなったり、ハードウェアが正常に動作しなくなったりする可能性があります。続行しますか？",
    writeSuccess: "BIOS NVRAM の書き込みコマンドが完了しました",
    writeFailed: "BIOS NVRAM の書き込みに失敗しました",

    writing: "BIOS NVRAM に書き込み中...",
    writeCancelled: "BIOS の書き込みをキャンセルしました",
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