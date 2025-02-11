
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

> intel NUC 系列风扇曲线控制 LAPAC71H, LAPKC71F 等机型

> [!NOTE]
> 随缘更新, 如果觉得有用请给个⭐
>
> [详细更新说明](./src-tauri/NUCtoolChange.md)

> [!WARNING]
> [必要说明](assets/分析.md)

# 

|机型支持(以实际为准)|LAPAC71H|LAPKC71F|
|:--------:|:----:|:----:|
| 风扇控制 | ✓ | ✓ |
| 异常恢复 | ✓ | ✓ |
| CPU L1 | ✓ | ☐ |
| CPU L2 | ✓ | ☐ |
| GPU L1 | ✓ | ✕ |
| GPU L2 | ✓ | ✕ |

# 

### 使用方法({}中为必须步骤，[]可选步骤)
- 打开程序后`{调整风扇曲线}`并点击`{保存配置}`
> [!WARNING]
> 首次使用必须执行上述操作
> 配置路径: `%AppData%\com.nuc.x15.fan.cyear.app`
- 每次重新打开程序需要点击`加载配置`
- 点击`Start`

# 

### UI对比
<details>
  <summary>
    0.1.2版本运行界面
  </summary>

![0.1.2](assets/0.1.2.png)
</details>

<details>
  <summary>
    0.3.0版本运行界面
  </summary>

![0.3.0](assets/0.3.0.png)
</details>
