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

> [!CAUTION]
> This document may be out of date, please check the latest Chinese document.
> 
> This document was last updated on: Mar. 4, 2025.

# NUCtool

> Intel NUC X15 series tool for models like LAPAC71H, LAPKC71F, etc.

> [!NOTE]
> Updates are not guaranteed. If you find it useful, please give a ⭐.
>
> [English Document](./README_English.md)
>
> [Detailed Update Notes](./NUCtoolChange.md)

> [!WARNING]
> There is a risk of hardware damage. By using this program, you accept the risk. Any issues arising are at your own responsibility.
>
> This program may have serious defects. Please ensure safety before using it.
>
> Before submitting feedback, please read [Essential Information](./分析.md).

| Windows/Linux Support |   LAPAC71H   |   LAPKC71F   |
|:----------------:|:------------:|:------------:|
|   Fan Control    |      ✓ ✓      |    ✓ ☐     |
|   Recovery Mode  |      ✓ ✓      |    ✓ ☐     |
|    CPU L1       |      ✓ ✓      |    ☐ ☐     |
|    CPU L2       |      ✓ ✓      |    ☐ ☐     |
|    GPU L1       |      ✓ ✓      |    ✕ ✕     |
|    GPU L2       |      ✓ ✓      |    ✕ ✕     |
|   GPU MAX       |     100 85     |   ✕ ✕     |
|   Thermal Limit |      ✓ ?      |    ☐ ?     |
|   RGB Keyboard  |      * ?       |    * ?     |
|   Custom LED    |      * ?       |    * ?     |
|   Auto Update   |      ✓ ✓       |    ✓ ✓    |
|   Auto Start    |      ✓ ☐      |    ✓ ☐     |

`✓`: Supported `✕`: Not Supported 

`☐`: Not Tested `*`: Pending Update 

`?`: Possibly Supported but Unnecessary

> [!WARNING]
> There is a risk of hardware damage. By using this program, you accept the risk. Any issues arising are at your own responsibility.
>
> Platform Support:
>
> 1. Windows 11 / Windows 10 (Requires WebView2)
> 2. Linux (Requires Make)
>
> Windows Usage Guide (Features depend on actual implementation):
> 1. On first use, you need to **adjust the fan curve** and click **Save Configuration**.
> 2. Configuration path: `%AppData%\com.nuc.x15.fan.cyear.app`
>
> Linux Usage Guide (Features depend on actual implementation):
> 1. Kernel requirement: `>=6.13` (or `>=6.10`)
> 2. See **Linux Essential Steps** below, usage is similar to Windows (Steps 2-3).

> [!CAUTION]
> **Linux Essential Steps**
> ```shell
> # If kernel version >=6.13, run:
> git clone https://github.com/cyear/uniwill-laptop --branch kernel-6.13
> # If kernel version >=6.10, run:
> git clone https://github.com/cyear/uniwill-laptop
> # The following steps apply to all:
> cd uniwill-laptop
> # Compile, install, and load module:
> sudo sh u.sh
> # If a serious issue occurs, immediately power off or uninstall the module (do not set it to auto-load at startup).
> ```

#### Star History

[![Star History Chart](https://api.star-history.com/svg?repos=cyear/NUCtool&type=Timeline)](https://star-history.com/#cyear/NUCtool&Timeline)

#### Thanks

> Special thanks to [Carey Evans (Rust)](https://users.rust-lang.org/u/carey/summary)
> 
> Special thanks to [Wer-Wolf (uniwill-laptop)](https://github.com/Wer-Wolf/uniwill-laptop)
