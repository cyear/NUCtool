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
> 随缘更新, 如果觉得有用请给个⭐
>
> [English Document](./assets/README_English.md)
>
> [详细更新说明](./assets/NUCtoolChange.md)

> [!WARNING]
> 有损坏硬件风险, 使用此程序即认为接受风险, 出现问题概不负责
>
> 本程序可能有严重缺陷, 请在保障安全情况下使用此程序
>
> 反馈前请看 [必要说明](assets/分析.md)
# 
|Windows 支持|LAPAC71H|LAPKC71F|
|:--------:|:----:|:----:|
| 风扇控制 | ✓ | ✓ |
| 异常恢复 | ✓ | ✓ |
| CPU L1 | ✓ | ☐ |
| CPU L2 | ✓ | ☐ |
| GPU L1 | ✓ | ✕ |
| GPU L2 | ✓ | ✕ |
| GPU MAX | 100 | ✕ |
| 温度墙 | ✓ | ☐ |
| 自动更新 | ✓ | ✓ |
| 开机自启 | ✓ | ✓ |
|        |   |    |
|        |   |    |
|**Linux 支持**|**LAPAC71H**|**LAPKC71F**|
| 风扇控制 | ✓ | ☐ |
| 异常恢复 | ✓ | ☐ |
| CPU L1 | * | ✕ |
| CPU L2 | * | ✕ |
| GPU L1 | * | ✕ |
| GPU L2 | * | ✕ |
| GPU MAX | 85 | ✕ |
| 温度墙 | * | * |
| 自动更新 | ✓ | ✓ |
| 开机自启 | ☐ | ☐ |

`✓`: 支持 `☐`: 未测试 `✕`: 不支持 `*`: 等待更新
# 
> [!WARNING]
> 有损坏硬件风险, 使用此程序即认为接受风险, 出现问题概不负责
>
> 平台支持
>
> 1. Windows 10
> 2. Windows 11
> 3. Linux
>
> Windows 使用教程(支持功能以实际为准)
> 1. 非WIN11系统需检查是否安装webview2
> 2. 首次使用需`调整风扇曲线`并点击`保存配置`
> 3. 每次重新打开程序需要点击`加载配置`
> 4. 点击`Start`
> 5. 配置路径: `%AppData%\com.nuc.x15.fan.cyear.app`
>
> Linux 使用教程(支持功能以实际为准)
> 1. 有损坏硬件风险, 使用此程序即认为接受风险, 出现问题概不负责
> 2. 内核要求>=6.13(或>=6.10)
> 3. 必要依赖: dmidecode make git
> 4. 需要在`sudo`下运行(如果不赋予root权限将不会控制你的设备)
> 5. 注: 模块不会检查更新，请查看下面PS
> 6. 使用教程同Windows(2-4 step)
>

> 致谢 [Carey Evans(Rust)](https://users.rust-lang.org/u/carey/summary) 大佬
> 
> 致谢 [Wer-Wolf(uniwill-laptop)](https://github.com/Wer-Wolf/uniwill-laptop) 大佬

PS: Linux 如果模块加载有问题等，可以手动编译加载
```shell
# 内核版本>=6.13 执行
git clone https://github.com/cyear/uniwill-laptop --branch kernel-6.13 /root/.config/nuc_model
# 内核版本>=6.10 执行
git clone https://github.com/cyear/uniwill-laptop /root/.config/nuc_model
# 以下均可执行
cd /root/.config/nuc_model
# 检查更新看这里(如果需要更新)
git pull
# 卸载编译安装加载
sudo sh u.sh
# 如出现严重问题，请立刻断电或者卸载模块(请不要改为开机自动加载)
```

# 
### Star History

[![Star History Chart](https://api.star-history.com/svg?repos=cyear/NUCtool&type=Timeline)](https://star-history.com/#cyear/NUCtool&Timeline)
