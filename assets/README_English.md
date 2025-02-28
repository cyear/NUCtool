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

> Intel NUC X15 Series Tools for LAPAC71H, LAPKC71F and other models

> [!NOTE]  
> Updated casually; if you find it useful, please give it a ⭐.  
>  
> [Detailed Update Notes](./assets/NUCtoolChange.md)
>
> This document is almost entirely translated by chatgpt.

> [!WARNING]  
> There is a risk of hardware damage. Using this program means you accept the risk, and no issues will be held responsible.  
>  
> This program may have serious flaws, so please use it under safe conditions.  
>  
> Please read before giving feedback [Necessary Instructions](assets/分析.md)

---

| **Windows Support** | **LAPAC71H** | **LAPKC71F** |
|:-------------------:|:------------:|:------------:|
| Fan Control         | ✓            | ✓            |
| Exception Recovery  | ✓            | ✓            |
| CPU L1              | ✓            | ☐            |
| CPU L2              | ✓            | ☐            |
| GPU L1              | ✓            | ✕            |
| GPU L2              | ✓            | ✕            |
| GPU MAX             | 100          | ✕            |
| TCC                 | ✓            | ☐            |
| Auto Update         | ✓            | ✓            |
| Auto Start on Boot  | ✓            | ✓            |
|                     |              |              |
|                     |              |              |
| **Linux Support**   | **LAPAC71H** | **LAPKC71F** |
| Fan Control         | ✓            | ☐            |
| Exception Recovery  | ✓            | ☐            |
| CPU L1              | *            | ✕            |
| CPU L2              | *            | ✕            |
| GPU L1              | *            | ✕            |
| GPU L2              | *            | ✕            |
| GPU MAX             | 85           | ✕            |
| TCC                 | *            | *            |
| Auto Update         | ✓            | ✓            |
| Auto Start on Boot  | ☐            | ☐            |

`✓`: Supported  
`☐`: Untested  
`✕`: Not Supported  
`*`: Awaiting Update

---

> [!WARNING]  
> There is a risk of hardware damage. Using this program means you accept the risk, and no issues will be held responsible.
>
> **Supported Platforms:**
>
> 1. Windows 10
> 2. Windows 11
> 3. Linux
>
> **Windows Usage Instructions** (Supported features as per actual performance):
> 1. On non-Windows 11 systems, ensure that webview2 is installed.
> 2. On first use, adjust the **Fan Curve** and click **Save Configuration**.
> 3. Each time you reopen the program, click **Load Configuration**.
> 4. Click **Start**.
> 5. Configuration path: `%AppData%\com.nuc.x15.fan.cyear.app`
>
> **Linux Usage Instructions** (Supported features as per actual performance):
> 1. There is a risk of hardware damage. Using this program means you accept the risk, and no issues will be held responsible.
> 2. Kernel requirement: ≥6.13 (or ≥6.10)
> 3. Required dependencies: `dmidecode`, `make`, `git`
> 4. Must be run under `sudo` (without root privileges, your device will not be controlled)
> 5. Note: The module does not check for updates; please refer to the PS below.
> 6. Usage instructions are the same as for Windows (steps 2–4).

> Special thanks to [Carey Evans (Rust)](https://users.rust-lang.org/u/carey/summary)  
> Special thanks to [Wer-Wolf (uniwill-laptop)](https://github.com/Wer-Wolf/uniwill-laptop)

---

**PS:** If you encounter issues with module loading on Linux, you can compile and load it manually:

```shell
# For kernel version ≥6.13, execute:
git clone https://github.com/cyear/uniwill-laptop --branch kernel-6.13 /root/.config/nuc_model
# For kernel version ≥6.10, execute:
git clone https://github.com/cyear/uniwill-laptop /root/.config/nuc_model
# Then execute:
cd /root/.config/nuc_model
# To check for updates (if needed):
git pull
# To uninstall, compile, install, and load:
sudo sh u.sh
# If a serious issue occurs, immediately cut power or unload the module (please do not set it to auto-load on boot)
```

# 
### Star History

[![Star History Chart](https://api.star-history.com/svg?repos=cyear/NUCtool&type=Timeline)](https://star-history.com/#cyear/NUCtool&Timeline)
