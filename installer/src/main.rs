use std::fs;
use std::path::PathBuf;
use std::process::Command;

const EMBEDDED_BINARY: &[u8] = include_bytes!("../../target/release/mnemosyne.exe");

fn main() {
    println!("=========================================");
    println!("  Mnemosyne & Decod Desktop Setup        ");
    println!("=========================================");
    println!();

    let local_appdata = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Local".to_string());
    let install_dir = PathBuf::from(&local_appdata).join("Programs").join("Mnemosyne");
    
    if let Err(e) = fs::create_dir_all(&install_dir) {
        eprintln!("Failed to create install directory: {e}");
        wait_exit();
        return;
    }

    let exe_path = install_dir.join("mnemosyne.exe");
    println!("Installing Mnemosyne to: {}", exe_path.display());
    if let Err(e) = fs::write(&exe_path, EMBEDDED_BINARY) {
        eprintln!("Failed to write mnemosyne.exe: {e}");
        wait_exit();
        return;
    }

    let uninstall_script = r#"@echo off
taskkill /f /im mnemosyne.exe >nul 2>&1
timeout /t 1 /nobreak >nul
del "%userprofile%\Desktop\Mnemosyne.lnk" >nul 2>&1
del "%appdata%\Microsoft\Windows\Start Menu\Programs\Mnemosyne.lnk" >nul 2>&1
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Mnemosyne" /f >nul 2>&1
rmdir /s /q "%~dp0" >nul 2>&1
"#;
    let _ = fs::write(install_dir.join("uninstall.cmd"), uninstall_script);

    println!("Creating desktop and start menu shortcuts...");
    let script = format!(
        r#"
        $wsh = New-Object -ComObject WScript.Shell
        $desktop = [Environment]::GetFolderPath('Desktop')
        $startMenu = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'

        foreach ($dir in @($desktop, $startMenu)) {{
            if (Test-Path $dir) {{
                $sc = $wsh.CreateShortcut((Join-Path $dir 'Mnemosyne.lnk'))
                $sc.TargetPath = '{0}'
                $sc.WorkingDirectory = '{1}'
                $sc.IconLocation = '{0}'
                $sc.Save()
            }}
        }}

        $uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Mnemosyne'
        New-Item -Path $uninstallKey -Force | Out-Null
        New-ItemProperty -Path $uninstallKey -Name DisplayName -Value 'Mnemosyne' -PropertyType String -Force | Out-Null
        New-ItemProperty -Path $uninstallKey -Name DisplayVersion -Value '0.1.0' -PropertyType String -Force | Out-Null
        New-ItemProperty -Path $uninstallKey -Name InstallLocation -Value '{1}' -PropertyType String -Force | Out-Null
        New-ItemProperty -Path $uninstallKey -Name DisplayIcon -Value '{0}' -PropertyType String -Force | Out-Null
        New-ItemProperty -Path $uninstallKey -Name UninstallString -Value 'cmd.exe /c "{1}\\uninstall.cmd"' -PropertyType String -Force | Out-Null
        New-ItemProperty -Path $uninstallKey -Name QuietUninstallString -Value 'cmd.exe /c "{1}\\uninstall.cmd"' -PropertyType String -Force | Out-Null
        New-ItemProperty -Path $uninstallKey -Name NoModify -Value 1 -PropertyType DWord -Force | Out-Null
        New-ItemProperty -Path $uninstallKey -Name NoRepair -Value 1 -PropertyType DWord -Force | Out-Null
        "#,
        exe_path.display().to_string().replace('\\', "\\\\"),
        install_dir.display().to_string().replace('\\', "\\\\")
    );

    let _ = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .output();
    println!("✓ Shortcuts created.");

    println!("Checking Claude Code CLI...");
    let claude_check = Command::new("cmd.exe")
        .args(["/D", "/C", "claude --version"])
        .output();
    if claude_check.map(|o| o.status.success()).unwrap_or(false) {
        println!("✓ Claude Code CLI is ready.");
    } else {
        println!("Installing Claude Code CLI via npm...");
        let _ = Command::new("cmd.exe")
            .args(["/D", "/C", "npm install -g @anthropic-ai/claude-code"])
            .status();
    }

    println!();
    println!("=========================================");
    println!("  Installation Successful!               ");
    println!("=========================================");
    println!("Launching Mnemosyne...");
    let _ = Command::new(&exe_path).spawn();
}

fn wait_exit() {
    println!("\nPress Enter to exit...");
    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf);
}
