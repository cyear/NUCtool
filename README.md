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

> Intel NUC X15 系列工具用于 LAPAC71H, LAPKC71F 等机型

> [!NOTE]
>
> 交流群: https://discord.gg/8geBWer34C

> [!NOTE]
> 随缘更新, 如果觉得有用请给个⭐
>
> [English Document](./assets/README_English.md)
>
> [详细更新说明](./assets/NUCtoolChange.md)
>


# 免责声明（Disclaimer）

在使用 NUCtool（以下简称“本软件”）之前，请您仔细阅读以下条款。**一旦您下载、使用或运行本软件，即表示您已充分理解并同意本声明的全部内容。**

## 1. 风险性质说明
本软件旨在提供对英特尔 NUC笔记本设备的硬件监控、功耗解锁（PL1/PL2）、风扇转速调控及底层等功能。此类操作**涉及对硬件固件（BIOS/EC）及系统内核的直接交互**，与普通应用程序有本质区别。

## 2. 硬件与数据风险
您明确知晓并自愿承担以下潜在风险：

- **物理损伤**：不恰当的功耗配置或风扇停转策略可能导致 CPU、主板供电模块（VRM）或周边元器件**过热、烧毁或寿命缩减**。
- **系统稳定性**：错误的设置可能引发**频繁蓝屏、死机、无法开机（Brick）** 或存储设备（SSD）数据损毁。
- **保修失效**：使用本软件进行操作，**可能导致设备丧失官方保修资格**（具体以厂商政策为准）。

## 3. 免责条款
- **“稳定性**：本软件不附带任何明示或暗示的担保，包括但不限于对特定用途适用性、稳定性的担保。
- **开发者免责**：项目作者（cyear）及贡献者**不对任何因使用或无法使用本软件引起的直接、间接、偶然、特殊或惩戒性损害承担责任**，包括但不限于设备报废、数据丢失、业务中断及经济损失，即使已被告知发生此类情况的可能性。

## 4. 用户义务
- 使用前，请确保您已具备**硬件故障排查与系统恢复**的基础能力。
- 强烈建议在调整参数前，**先行备份硬盘中的重要数据**。

## 5. 最终解释权
本声明的最终解释权及修改权归 NUCtool 项目团队所有。

---

> ⚠️ **警告：** 若您无法接受上述条款，或不确定操作后果，请**立即停止使用**并删除本软件。



> [!WARNING]
> 有损坏硬件风险, 使用此程序即认为接受风险, 出现问题概不负责
>
> 本程序可能有严重缺陷, 请在保障安全情况下使用此程序
>
> 反馈前请看 [必要说明](assets/分析.md)

| Windows/Linux 支持 |   LAPAC71H    |  LAPKC71F   |
|:------------:|:-------------:|:-----------:|
|   风扇控制    |      ✓ ✓      |     ✓ ☐     |
|   异常恢复    |      ✓ ✓      |     ✓ ☐     |
|    CPU L1    |      ✓ ✓      |     ☐ ☐     |
|    CPU L2    |      ✓ ✓      |     ☐ ☐     |
|    GPU L1    |      ✓ ✓      |     ✕ ✕     |
|    GPU L2    |      ✓ ✓      |     ✕ ✕     |
|   GPU MAX    |    100 85     |     ✕ ✕     |
|    温度墙     |      ✓ ?      |     ☐ ?     |
|  键盘彩色LED  |      ✓ ?       |     ☐ ?      |
| 键盘自定义LED |      * ?      |     * ?     |
|   自动更新    |      ✓ ✓      |     ✓ ✓     |
|   开机自启    |      ✓ ☐      |     ✓ ☐     |

`✓`: 支持 `✕`: 不支持 

`☐`: 未测试 `*`: 等待更新 

`?`: 也许支持但没必要支持

> [!WARNING]
> 有损坏硬件风险, 使用此程序即认为接受风险, 出现问题概不负责
>
> 平台支持
>
> 1. Windows 11 / Windows 10(依赖webview2)
> 2. Linux(依赖make)
>
> Windows 使用教程(支持功能以实际为准)
> 1. 首次使用需`调整风扇曲线`并点击`保存配置`
> 2. 配置路径: `%AppData%\com.nuc.x15.fan.cyear.app`
>
> Linux 使用教程(支持功能以实际为准)
> 1. 内核要求>=6.13(或>=6.10)
> 2. 请看下方 Linux 必要步骤，使用同Windows(2-3)

> [!CAUTION]
> Linux 必要步骤
> ```shell
> # 内核版本>=6.13 执行
> git clone https://github.com/cyear/uniwill-laptop --branch kernel-6.13
> # 内核版本>=6.10 执行
> git clone https://github.com/cyear/uniwill-laptop
> # 以下均可执行
> cd uniwill-laptop
> # 编译安装加载
> sudo sh u.sh
> # 如出现严重问题，请立刻断电或者卸载模块(请不要改为开机自动加载)
> ```

#### Star History

[![Star History Chart](https://api.star-history.com/svg?repos=cyear/NUCtool&type=Timeline)](https://star-history.com/#cyear/NUCtool&Timeline)

#### Thank

> 致谢 [Carey Evans(Rust)](https://users.rust-lang.org/u/carey/summary) 大佬
> 
> 致谢 [Wer-Wolf(uniwill-laptop)](https://github.com/Wer-Wolf/uniwill-laptop) 大佬
