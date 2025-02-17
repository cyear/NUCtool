use powershell_script::PsScriptBuilder;
use std::{env, process};

/**
* @Author: cyear
* @Create time: 2025-01-30
* @Description:
* @Version: 0.3.5
**/

pub fn privilege_escalation() {
    let ps = PsScriptBuilder::new()
        .no_profile(true)
        .non_interactive(true)
        .hidden(true)
        .print_commands(false)
        .build();
    let s = r#"if (([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { Write-Output "1" } else { Write-Output "0" }"#;
    let out = ps.run(s).unwrap();
    println!("{:?}", &out);
    if out.to_string().trim().parse::<i64>().unwrap() == 0 {
        if let Ok(path) = env::current_exe() {
            let path_str = path.to_string_lossy();
            println!("当前程序的路径是: {}", &path_str);
            ps.run(format!(r#"Start-Process "{}" -Verb RunAs"#, path_str).as_str())
                .unwrap();
            process::exit(0);
        }
    };
}
