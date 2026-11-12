use kc_led_hid::{
    Direction, Effect, FnBrightnessEvents, HidError, KEY_COUNT, KcKeyboard, KeyLayout, KeySetting,
    NssKeyGroups, NssProfile, Profile, Result, Rgb, Supply, Transport, load_console_profile,
    load_key_setting, paint_nss_group, project_aurora_enabled, project_aurora_installed,
    save_console_profile, save_key_setting, set_project_aurora, watch_profile,
};
use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
};
use windows_sys::Win32::System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT, SetConsoleCtrlHandler};

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let mut args: Vec<_> = env::args().skip(1).collect();
    let dry = args.iter().any(|arg| arg == "--dry-run");
    args.retain(|arg| arg != "--dry-run");
    let args: Vec<_> = args.iter().map(String::as_str).collect();

    match args.as_slice() {
        [] => return interactive_menu(),
        ["help" | "--help" | "-h"] => print_help(),
        ["effects"] => {
            for effect in Effect::ALL {
                println!(
                    "{:<10} HID={:>2} 配置={}",
                    effect.name(),
                    effect.hid_code(),
                    *effect as u32
                );
            }
        }
        ["config", "default"] => {
            Profile::factory_preset(Effect::Monocolor).write_json(io::stdout().lock())?
        }
        ["config", "preset", effect] => {
            Profile::factory_preset(effect.parse()?).write_json(io::stdout().lock())?;
        }
        ["config", "get"] => load_console_profile()?.write_json(io::stdout().lock())?,
        ["config", "import-nss", path, layout] => {
            let nss = NssProfile::read(File::open(path)?)?;
            nss.to_profile(key_layout(layout)?)?
                .write_json(io::stdout().lock())?;
        }
        ["config", "export-nss", profile, template, layout, name] => {
            let nss = NssProfile::read(File::open(template)?)?;
            nss.export(
                &load(profile)?,
                key_layout(layout)?,
                name,
                io::stdout().lock(),
            )?;
        }
        [
            "config",
            "paint-group",
            profile,
            groups,
            layout,
            name,
            color,
        ] => {
            let mut profile = load(profile)?;
            paint_nss_group(
                &mut profile,
                key_layout(layout)?,
                File::open(groups)?,
                name,
                rgb(color)?,
            )?;
            profile.write_json(io::stdout().lock())?;
        }
        ["config", "group-list", path] => {
            for (name, keys) in NssKeyGroups::read(File::open(path)?)?.groups() {
                println!("{name}\t{keys:?}");
            }
        }
        ["config", "group-set", path, layout, name, keys @ ..] => {
            let mut groups = if *path == "-" {
                NssKeyGroups::default()
            } else {
                NssKeyGroups::read(File::open(path)?)?
            };
            groups.set(key_layout(layout)?, name, keys)?;
            groups.write(io::stdout().lock())?;
        }
        ["config", "group-remove", path, name] => {
            let mut groups = NssKeyGroups::read(File::open(path)?)?;
            groups.remove(name)?;
            groups.write(io::stdout().lock())?;
        }
        ["config", "save", path] if !dry => save_console_profile(&load(path)?)?,
        ["config", "cycle", path, source] => {
            let mut profile = load(path)?;
            profile.cycle_brightness(match *source {
                "ac" => Supply::Ac,
                "dc" => Supply::Dc,
                _ => return Err(HidError("电源类型应为 ac 或 dc".into()).into()),
            });
            profile.write_json(io::stdout().lock())?;
        }
        ["project-aurora", "status"] => {
            println!(
                "enabled={} installed={}",
                project_aurora_enabled()?,
                project_aurora_installed()?
            );
        }
        ["project-aurora", status @ ("on" | "off")] if !dry => {
            set_project_aurora(*status == "on")?;
        }
        ["config", "apply", path, options @ ..] => {
            let profile = load(path)?;
            let (supply, save) = apply_options(options)?;
            let supply = supply.map_or_else(Supply::current, Ok)?;
            hardware(dry, |keyboard| {
                keyboard.apply_profile(&profile, supply, save)
            })?;
        }
        ["watch" | "music", path, options @ ..] if !dry => {
            let listen = match options {
                [] => false,
                ["--fn"] => true,
                _ => return Err(HidError("watch 和 music 只接受 --fn 参数".into()).into()),
            };
            let mut profile = load(path)?;
            if args[0] == "music" && profile.effect != Effect::Music {
                return Err(HidError("music 需要 effect 为 music 的配置".into()).into());
            }
            let _guard = StopHandler::register()?;
            let mut keyboard = KcKeyboard::open()?;
            let events = if listen && profile.power {
                Some(FnBrightnessEvents::open()?)
            } else {
                None
            };
            watch_profile(&mut keyboard, &mut profile, events.as_ref(), stopped)?;
        }
        ["fn-probe"] if !dry => {
            FnBrightnessEvents::open()?.poll()?;
            println!("已订阅原厂 Fn 亮度 WMI 事件");
        }
        ["key-setting", "get"] if !dry => {
            let keyboard = KcKeyboard::open()?;
            for key in KeySetting::ALL {
                println!("{}={}", key.name(), keyboard.key_setting(key)?);
            }
        }
        ["key-setting", "get", name] if !dry => {
            let key: KeySetting = name.parse()?;
            println!("{}={}", key.name(), KcKeyboard::open()?.key_setting(key)?);
        }
        ["key-setting", "set", name, state @ ("on" | "off")] => {
            let key: KeySetting = name.parse()?;
            if dry {
                println!("预演 KC 键盘设置 {}={state}", key.name());
            } else {
                KcKeyboard::open()?.set_key_setting(key, *state == "on")?;
            }
        }
        ["key-setting", "get-saved", name] if !dry => {
            let key: KeySetting = name.parse()?;
            println!("{}={}", key.name(), load_key_setting(key)?);
        }
        ["key-setting", "save", name, state @ ("on" | "off")] if !dry => {
            save_key_setting(name.parse()?, *state == "on")?;
        }
        ["keys", layout] => {
            for (name, slots) in key_layout(layout)?.keys() {
                println!("{name:<16} {slots:?}");
            }
        }
        ["config", "paint", path, layout, changes @ ..] if !changes.is_empty() => {
            let mut profile = load(path)?;
            let layout = key_layout(layout)?;
            for change in changes {
                let (key, color) = change
                    .split_once('=')
                    .ok_or_else(|| HidError("按键颜色应为 键名=R,G,B".into()))?;
                profile.paint_key(layout, key, rgb(color)?)?;
            }
            profile.write_json(io::stdout().lock())?;
        }
        ["probe"] if !dry => {
            let _keyboard = KcKeyboard::open()?;
            println!("已匹配 KC HID 048d:6006 FF03/1，Feature=9，Output=65");
        }
        ["layout"] if !dry => println!("{}", KcKeyboard::open()?.keyboard_layout()?),
        ["version"] if !dry => {
            let [high, low, test, customer] = KcKeyboard::open()?.firmware_version()?;
            println!("high={high} low={low} test={test} customer={customer}");
        }
        ["read-effect"] if !dry => {
            let report = KcKeyboard::open()?.read_effect()?;
            println!(
                "控制={} 效果={} 速度={} 亮度={} 色槽={} 方向={}",
                report[2], report[3], report[4], report[5], report[6], report[7],
            );
        }
        ["version" | "read-effect"] => {
            let command = if args[0] == "version" { 0x80 } else { 0x88 };
            Preview.set_feature(&[0, command, 0, 0, 0, 0, 0, 0, 0])?;
        }
        ["brightness", level] => {
            let level = number(level, "亮度", 100)?;
            hardware(dry, |k| k.set_brightness(level))?;
        }
        ["switch-light", "off"] => hardware(dry, |keyboard| keyboard.set_brightness(0))?,
        ["switch-light", "on"] => {
            let profile = load_console_profile()?;
            let level = profile.brightness(Supply::current()?);
            hardware(dry, |keyboard| keyboard.set_brightness(level))?;
        }
        ["switch-light", "on", path] => {
            let profile = load(path)?;
            let level = profile.brightness(Supply::current()?);
            hardware(dry, |keyboard| keyboard.set_brightness(level))?;
        }
        ["timeout", state, time, options @ ..] => {
            let enabled = match *state {
                "on" => true,
                "off" => false,
                _ => return Err(HidError("timeout 参数应为 on 或 off".into()).into()),
            };
            let time = number(time, "HID 原始 timeout 字节", 255)?;
            let save = only_save(options)?;
            hardware(dry, |keyboard| {
                keyboard.set_timeout_with_save(enabled, time, save)
            })?;
        }
        ["power", enabled, options @ ..] => {
            let enabled = match *enabled {
                "on" => true,
                "off" => false,
                _ => return Err(HidError("power 参数应为 on 或 off".into()).into()),
            };
            let save = only_save(options)?;
            hardware(dry, |k| k.set_power_with_save(enabled, save))?;
        }
        ["effect", effect, speed, brightness, direction, options @ ..] => {
            let effect = effect
                .parse::<u8>()
                .or_else(|_| effect.parse::<Effect>().map(Effect::hid_code))?;
            let speed = number(speed, "HID 速度", 10)?;
            let brightness = number(brightness, "亮度", 100)?;
            let direction = direction
                .parse::<u8>()
                .or_else(|_| direction.parse::<Direction>().map(Direction::hid_code))?;
            let mut colors = Vec::new();
            let mut save = true;
            for option in options {
                if *option == "--volatile" {
                    save = false;
                } else if let Some(color) = option.strip_prefix("--color=") {
                    colors.push(rgb(color)?);
                } else {
                    return Err(HidError(format!("未知参数 {option}")).into());
                }
            }
            hardware(dry, |k| {
                k.set_firmware_effect(effect, speed, brightness, direction, &colors, save)
            })?;
        }
        ["user-mode", brightness, path, options @ ..] => {
            let brightness = number(brightness, "亮度", 100)?;
            let save = only_save(options)?;
            let colors = read_key_colors(path)?;
            hardware(dry, |k| {
                k.set_user_mode(brightness, &colors, u8::from(save))
            })?;
        }
        _ => return Err(HidError("参数不正确，请查看 --help".into()).into()),
    }
    Ok(())
}

fn interactive_menu() -> Result<()> {
    loop {
        println!(
            "\nKC 键盘测试菜单\n  1. 读取设备状态（探测、布局、固件）\n  2. 读取当前 OEM 配置\n  3. 选择灯效并应用到键盘\n  4. 设置亮度\n  5. 选择预设并写入 OEM 配置\n  6. 从 JSON 配置应用到键盘\n  7. 列出所有灯效\n  0. 退出"
        );
        let choice = prompt("请选择项目")?;
        if choice.trim().is_empty() {
            break;
        }
        match choice.trim() {
            "0" => break,
            "1" => match KcKeyboard::open() {
                Ok(mut keyboard) => {
                    println!("HID: 048d:6006 FF03/1 已连接");
                    match keyboard.keyboard_layout() {
                        Ok(layout) => println!("键盘布局: {layout}"),
                        Err(error) => println!("布局读取失败: {error}"),
                    }
                    match keyboard.firmware_version() {
                        Ok([high, low, test, customer]) => println!(
                            "固件版本: high={high} low={low} test={test} customer={customer}"
                        ),
                        Err(error) => println!("固件版本读取失败: {error}"),
                    }
                }
                Err(error) => println!("设备未就绪: {error}"),
            },
            "2" => match load_console_profile() {
                Ok(profile) => profile.write_json(io::stdout().lock())?,
                Err(error) => println!("配置读取失败: {error}"),
            },
            "3" => {
                for (index, effect) in Effect::ALL.iter().enumerate() {
                    println!("  {}. {}", index + 1, effect.name());
                }
                let selected = prompt("输入灯效编号")?;
                let Ok(index) = selected.trim().parse::<usize>() else {
                    println!("请输入列表中的编号");
                    continue;
                };
                let Some(effect) = index.checked_sub(1).and_then(|i| Effect::ALL.get(i)) else {
                    println!("灯效编号超出范围");
                    continue;
                };
                if *effect == Effect::Music {
                    println!("音乐灯效需要持续运行，请在命令行运行: kc-led-hid music <配置.json>");
                    continue;
                }
                let save = prompt("保存到键盘? 输入 y 保存，其他输入仅本次应用")?;
                let profile = Profile::factory_preset(*effect);
                match KcKeyboard::open().and_then(|mut keyboard| {
                    keyboard.apply_profile(
                        &profile,
                        Supply::current()?,
                        save.trim().eq_ignore_ascii_case("y"),
                    )
                }) {
                    Ok(()) => println!("已应用 {} 灯效", effect.name()),
                    Err(error) => println!("应用失败: {error}"),
                }
            }
            "4" => {
                let value = prompt("输入亮度 0..100")?;
                let Ok(level) = value.trim().parse::<u8>() else {
                    println!("亮度必须是 0..100 的整数");
                    continue;
                };
                if level > 100 {
                    println!("亮度必须是 0..100");
                    continue;
                }
                match KcKeyboard::open().and_then(|mut keyboard| keyboard.set_brightness(level)) {
                    Ok(()) => println!("亮度已设为 {level}%"),
                    Err(error) => println!("亮度设置失败: {error}"),
                }
            }
            "5" => {
                for (index, effect) in Effect::ALL.iter().enumerate() {
                    println!("  {}. {}", index + 1, effect.name());
                }
                let selected = prompt("选择要保存的预设编号")?;
                let Ok(index) = selected.trim().parse::<usize>() else {
                    println!("请输入列表中的编号");
                    continue;
                };
                let Some(effect) = index.checked_sub(1).and_then(|i| Effect::ALL.get(i)) else {
                    println!("灯效编号超出范围");
                    continue;
                };
                let confirm = prompt("这会写入 Windows OEM 注册表。确认? 输入 y 继续")?;
                if !confirm.trim().eq_ignore_ascii_case("y") {
                    println!("已取消");
                    continue;
                }
                match save_console_profile(&Profile::factory_preset(*effect)) {
                    Ok(()) => println!("已将 {} 预设写入 OEM 注册表", effect.name()),
                    Err(error) => println!("配置写入失败: {error}"),
                }
            }
            "6" => {
                let path = prompt("输入 JSON 配置文件路径")?;
                let profile = match load(path.trim()) {
                    Ok(profile) => profile,
                    Err(error) => {
                        println!("无法读取或解析该 JSON 配置: {error}");
                        continue;
                    }
                };
                let save = prompt("保存到键盘? 输入 y 保存，其他输入仅本次应用")?;
                match KcKeyboard::open().and_then(|mut keyboard| {
                    keyboard.apply_profile(
                        &profile,
                        Supply::current()?,
                        save.trim().eq_ignore_ascii_case("y"),
                    )
                }) {
                    Ok(()) => println!("配置已应用"),
                    Err(error) => println!("配置应用失败: {error}"),
                }
            }
            "7" => {
                for effect in Effect::ALL {
                    println!("{:<10} HID={:>2}", effect.name(), effect.hid_code());
                }
            }
            _ => println!("未知选项，请输入 0..7"),
        }
    }
    Ok(())
}

fn prompt(label: &str) -> Result<String> {
    print!("{label}: ");
    io::stdout().flush()?;
    let mut value = String::new();
    io::stdin().read_line(&mut value)?;
    Ok(value)
}

fn key_layout(value: &str) -> Result<KeyLayout> {
    match value {
        "us" | "85" => Ok(KeyLayout::Us),
        "iso" | "86" => Ok(KeyLayout::Iso),
        "jis" | "87" => Ok(KeyLayout::Jis),
        _ => Err(HidError("布局应为 us、iso 或 jis".into()).into()),
    }
}

fn load(path: &str) -> Result<Profile> {
    Profile::from_json(File::open(path)?)
}

fn number(value: &str, name: &str, max: u8) -> Result<u8> {
    match value.parse::<u8>() {
        Ok(value) if value <= max => Ok(value),
        _ => Err(HidError(format!("{name} 应为 0..={max}")).into()),
    }
}

fn rgb(text: &str) -> Result<Rgb> {
    let components: Vec<_> = text.split(',').map(str::trim).collect();
    let [red, green, blue] = components.as_slice() else {
        return Err(HidError("颜色应为 R,G,B".into()).into());
    };
    Ok([
        number(red, "R", 255)?,
        number(green, "G", 255)?,
        number(blue, "B", 255)?,
    ])
}

fn read_key_colors(path: &str) -> Result<[Rgb; KEY_COUNT]> {
    let colors = fs::read_to_string(path)?
        .trim_start_matches('\u{feff}')
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(rgb)
        .collect::<Result<Vec<_>>>()?;
    colors.try_into().map_err(|v: Vec<Rgb>| {
        HidError(format!("颜色文件需要 {KEY_COUNT} 行，实际 {} 行", v.len())).into()
    })
}

fn only_save(options: &[&str]) -> Result<bool> {
    match options {
        [] => Ok(true),
        ["--volatile"] => Ok(false),
        _ => Err(HidError("只接受 --volatile 参数".into()).into()),
    }
}

fn apply_options(options: &[&str]) -> Result<(Option<Supply>, bool)> {
    let mut supply = None;
    let mut save = true;
    for option in options {
        match *option {
            "--ac" if supply.is_none() => supply = Some(Supply::Ac),
            "--dc" if supply.is_none() => supply = Some(Supply::Dc),
            "--volatile" => save = false,
            _ => return Err(HidError(format!("未知或冲突的参数 {option}")).into()),
        }
    }
    Ok((supply, save))
}

// Pass the same SDK call to both transports using an object-safe reference.
fn hardware(
    dry: bool,
    execute: impl FnOnce(&mut KcKeyboard<&dyn Transport>) -> Result<()>,
) -> Result<()> {
    let native;
    let transport: &dyn Transport = if dry {
        &Preview
    } else {
        native = kc_led_hid::NativeHid::open()?;
        &native
    };
    execute(&mut KcKeyboard::new(transport)?)?;
    if !dry {
        println!("已发送 HID 报文");
    }
    Ok(())
}

struct Preview;
impl Transport for Preview {
    fn prepare_key_trigger(&self) -> Result<()> {
        Ok(())
    }
    fn feature_len(&self) -> usize {
        9
    }
    fn set_feature(&self, report: &[u8]) -> Result<()> {
        print_report("Feature", report)
    }
    fn get_feature(&self, _: &mut [u8]) -> Result<()> {
        Err(HidError("预演模式没有硬件应答".into()).into())
    }
    fn write_output(&self, report: &[u8]) -> Result<()> {
        print_report("Output", report)
    }
    fn enable_key_trigger(&self) -> Result<()> {
        writeln!(io::stdout().lock(), "EC      0x0741 |= 0x08")?;
        Ok(())
    }
}

fn print_report(kind: &str, report: &[u8]) -> Result<()> {
    let mut out = io::stdout().lock();
    write!(out, "{kind}")?;
    for value in report {
        write!(out, " {value:02x}")?;
    }
    writeln!(out)?;
    Ok(())
}

static STOP: AtomicBool = AtomicBool::new(false);
fn stopped() -> bool {
    STOP.load(Ordering::Relaxed)
}

unsafe extern "system" fn handle_stop(event: u32) -> i32 {
    if matches!(event, CTRL_C_EVENT | CTRL_BREAK_EVENT) {
        STOP.store(true, Ordering::Relaxed);
        1
    } else {
        0
    }
}

struct StopHandler;
impl StopHandler {
    fn register() -> Result<Self> {
        if unsafe { SetConsoleCtrlHandler(Some(handle_stop), 1) } == 0 {
            return Err(io::Error::last_os_error().into());
        }
        Ok(Self)
    }
}
impl Drop for StopHandler {
    fn drop(&mut self) {
        unsafe { SetConsoleCtrlHandler(Some(handle_stop), 0) };
    }
}

fn print_help() {
    println!(
        "KC 键盘 SDK 命令
  (直接运行不带参数可打开交互式测试菜单)
  effects
  probe | layout | version | read-effect
  keys <us|iso|jis>
  brightness <0..100>
  switch-light <on|off> [配置.json] 仅调亮度，不切换固件电源
  timeout <on|off> <HID原始字节0..255> [--volatile]
  power <on|off> [--volatile]
  effect <名称|HID-ID> <HID速度0..10> <亮度0..100> <方向名|0..4>
         [--color=R,G,B]... [--volatile]
  user-mode <亮度0..100> <rgb.csv> [--volatile]
  config default                         输出可编辑的 JSON
  config preset <效果>                   输出原厂 KC 灯效预设 JSON
  config get                             读取原厂注册表为 JSON
  config import-nss <原厂.json> <布局>    将 NSS 命名配置转为 SDK JSON
  config export-nss <SDK.json> <原厂模板.json> <布局> <名称>
  config paint-group <SDK.json> <KeyGroups.json> <布局> <组名> <R,G,B>
  config group-list <KeyGroups.json>
  config group-set <KeyGroups.json|-> <布局> <组名> <键名>...
  config group-remove <KeyGroups.json> <组名>
  config paint <配置.json> <us|iso|jis> <键名=R,G,B>...
  config cycle <配置.json> <ac|dc>         输出应用 Fn 亮度循环的 JSON
  config save <配置.json>                写入原厂注册表
  config apply <配置.json> [--ac|--dc] [--volatile]
  watch <配置.json> [--fn]               持续执行 AC/DC、空闲策略或音乐模式
  music <配置.json> [--fn]               持续运行音乐模式
  fn-probe                              检查原厂 Fn 亮度事件订阅
  key-setting get [名称]
  key-setting set <名称> <on|off>
  key-setting get-saved <名称> | key-setting save <名称> <on|off>
    名称: windows-key | fn-lock | num-lock | scroll-lock | caps-lock | disable-touchpad-toggle
  project-aurora <status|on|off>          原厂集成开关
硬件发送命令支持 --dry-run。配置文件速度使用原厂 UI 的 0..9。"
    );
}
