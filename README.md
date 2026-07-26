![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/cyear/NUCtool/total?style=for-the-badge)
![GitHub Issues or Pull Requests](https://img.shields.io/github/issues/cyear/NUCtool?style=for-the-badge)
![GitHub Issues or Pull Requests](https://img.shields.io/github/issues-closed/cyear/NUCtool?style=for-the-badge)
![GitHub last commit](https://img.shields.io/github/last-commit/cyear/NUCtool?style=for-the-badge)
![GitHub top language](https://img.shields.io/github/languages/top/cyear/NUCtool?style=for-the-badge)
![GitHub License](https://img.shields.io/github/license/cyear/NUCtool?style=for-the-badge)
![GitHub repo size](https://img.shields.io/github/repo-size/cyear/NUCtool?style=for-the-badge)
![GitHub Repo stars](https://img.shields.io/github/stars/cyear/NUCtool?style=for-the-badge)
![GitHub Release](https://img.shields.io/github/v/release/cyear/NUCtool?style=for-the-badge)
![GitHub commits since latest release](https://img.shields.io/github/commits-since/cyear/NUCtool/latest?style=for-the-badge)

# NUCtool

> Intel NUC X15 系列工具用于 LAPAC71H, LAPKC71F 等机型 (仅支持 Windows)

> [!NOTE]
> v0.5.0 起项目仅保留 Windows 支持, Linux 用户请使用
> [tuxedo-control-center](https://github.com/tuxedocomputers/tuxedo-control-center)
> 或历史版本
>
> 交流群以及及时反馈: https://discord.gg/8geBWer34C

> [!NOTE]
> 随缘更新, 如果觉得有用请给个⭐
>
> [English Document](./assets/README_English.md)
>
> [详细更新说明](./assets/NUCtoolChange.md)
>
> 这里有话说了:
> 
> 你: 主播主播有没有比这个还好用的软件
>
> 主播: 兄弟有的有的, 这种软件还有两种
>
> [机械革命控制台 windows](http://mechrevo.com/)/[tuxedo-control-center Linux](https://github.com/tuxedocomputers/tuxedo-control-center)

> [!WARNING]
> 有损坏硬件风险, 使用此程序即认为接受风险, 出现问题概不负责
>
> 本程序可能有严重缺陷, 请在保障安全情况下使用此程序
>
> 反馈前请看 [必要说明](assets/分析.md)

| Windows 支持  |   LAPAC71H    |  LAPKC71F   |
|:------------:|:-------------:|:-----------:|
|   风扇控制    |       ✓       |      ✓      |
|   异常恢复    |       ✓       |      ✓      |
|    CPU L1    |       ✓       |      ☐      |
|    CPU L2    |       ✓       |      ☐      |
|    GPU L1    |       ✓       |      ✕      |
|    GPU L2    |       ✓       |      ✕      |
|   GPU MAX    |      100      |      ✕      |
|    温度墙     |       ✓       |      ☐      |
|  键盘彩色LED  |       ✓       |      ☐      |
| 键盘自定义LED |       *       |      *      |
|   自动更新    |       ✓       |      ✓      |
|   开机自启    |       ✓       |      ✓      |

`✓`: 支持 `✕`: 不支持 

`☐`: 未测试 `*`: 等待更新 

> [!WARNING]
> 平台支持
>
> 1. Windows 11 / Windows 10(依赖 webview2)
> 2. 需要管理员权限运行(程序会自动请求 UAC 提权)
>
> 使用教程(支持功能以实际为准)
> 1. 首次使用需`调整风扇曲线`并点击`保存配置`
> 2. 配置路径: `%AppData%\com.nuc.x15.fan.cyear.app`
> 3. 开机自启: 将配置目录下 `beta.config` 内容改为 `1`

#### 开发

```shell
# 依赖: Rust stable + Node.js
npm install
# 调试运行
npm run tauri dev
# 构建
npm run tauri build
```

#### Star History

[![Star History Chart](https://api.star-history.com/svg?repos=cyear/NUCtool&type=Timeline)](https://star-history.com/#cyear/NUCtool&Timeline)

#### Thank

> 致谢 [Carey Evans(Rust)](https://users.rust-lang.org/u/carey/summary) 大佬
> 
> 致谢 [Wer-Wolf(uniwill-laptop)](https://github.com/Wer-Wolf/uniwill-laptop) 大佬
