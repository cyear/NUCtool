# 更新日志

## v*.*.*-Beta [----年--月--日]

## v1.0.7-Beta [2026年9月23日]

1. 添加"设置-风扇模式"，默认主风扇优先
2. "NUCtool OSD"添加多语言支持
3. 添加性能模式从系统中获取当前模式
4. 添加灯条设置

风扇模式支持如下:

- 独立(对应使用)
- 主风扇优先(只使用Main风扇)
- 分风扇优先(只使用Secondary风扇)

已知问题:

- "系统配置-电池"没有读取按钮，需点击"性能调优-功耗限制-读取配置"

## v1.0.6-Beta [2026年9月22日]

1. Спасибо [Soapheart](https://github.com/Soapheart) за перевод на русский язык.

## v1.0.5-Beta [2026年9月21日]

1. 添加多语言支持
2. 添加DLL完整性校验
3. 添加参数`--no-osd`关闭OSD
4. 优化连续OSD触发问题

## v1.0.4-Beta [2026年9月21日]

1. 添加"NUCtool OSD"
2. 添加电池健康度支持

## v1.0.3-Beta [2026年9月17日]

1. 支持显示设置
2. 支持键盘LED灯设置

## v1.0.2-Beta [2026年9月16日]

1. 自启动设置进行优化，现在是一键添加任务计划(Beta)，并不推荐使用
2. 添加电源计划支持
3. 修复电池充电限制读取错误

## v1.0.1-Beta [2026年9月16日]

1. 添加了一些BUG
2. 支持性能监控：支持CPU、GPU温度显示，支持Main、Secondary风扇实时转速显示，支持"平台功耗"(原APC_WATT，具体作用未知，与CPU、GPU功耗疑似有关)显示
3. 支持性能调优：支持模式切换、功耗限制(支持以下：CPU：PL1/PL2/PL4、GPU：PL1/PL2、PSYS PL1、Battery Charging limit)
4. 支持风扇控制
5. 支持开机自启动(自启动不建议使用，有需求可用任务计划)
6. 添加FN快捷键支持(仅在后台有效哦，放前台为什么要用快捷键)
7. 添加托盘模式切换
8. 添加启动参数支持

FN快捷键支持如下：

- Fn + 1 => 性能模式
- Fn + 2 => 平衡模式
- Fn + 3 => 省电模式
- Fn + 4 => 基准模式

启动参数支持如下：

- `--hide` 启动只保留托盘
- `--fan-control` 自动启动风扇控制

### 自启动不建议使用(手动脚本)

#### 1.创建

```PowerShell
$action = New-ScheduledTaskAction `
    -Execute 'C:\Program Files\NUCtool\nuctool.exe' `
    -Argument '--hide --fan-control'

$trigger = New-ScheduledTaskTrigger -AtLogOn

$principal = New-ScheduledTaskPrincipal `
    -UserId "$env:USERDOMAIN\$env:USERNAME" `
    -LogonType Interactive `
    -RunLevel Highest

Register-ScheduledTask `
    -TaskName 'NUCtool' `
    -Action $action `
    -Trigger $trigger `
    -Principal $principal `
    -Force
```

#### 2.查询

```PowerShell
Get-ScheduledTask -TaskName 'NUCtool' | Select-Object TaskName,State
```

#### 3.删除

```PowerShell
Unregister-ScheduledTask -TaskName 'NUCtool' -Confirm:$false
```

---