$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "  Mnemosyne & Decod Unified Installer    " -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""

$installDir = Join-Path $env:LOCALAPPDATA 'Programs\Mnemosyne'
$startMenuDir = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
$desktopDir = [Environment]::GetFolderPath('Desktop')
$shortcutPath = Join-Path $startMenuDir 'Mnemosyne.lnk'
$desktopShortcutPath = Join-Path $desktopDir 'Mnemosyne.lnk'
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Mnemosyne'

# 1. Locate or build mnemosyne.exe
$binaryPath = Join-Path $PSScriptRoot 'mnemosyne.exe'
if (-not (Test-Path -LiteralPath $binaryPath -PathType Leaf)) {
    # Check target\release
    $repoRoot = (Get-Item $PSScriptRoot).Parent.Parent.FullName
    $candidate = Join-Path $repoRoot 'target\release\mnemosyne.exe'
    if (Test-Path -LiteralPath $candidate -PathType Leaf) {
        $binaryPath = $candidate
    } else {
        Write-Host "Building Mnemosyne in release mode..." -ForegroundColor Yellow
        Push-Location $repoRoot
        try {
            cargo build --release
            $binaryPath = $candidate
        } finally {
            Pop-Location
        }
    }
}

if (-not (Test-Path -LiteralPath $binaryPath -PathType Leaf)) {
    throw "mnemosyne.exe not found at $binaryPath"
}

# 2. Install files
Write-Host "Installing Mnemosyne to $installDir..." -ForegroundColor Green
New-Item -ItemType Directory -Force -Path $installDir, $startMenuDir | Out-Null
Copy-Item -LiteralPath $binaryPath -Destination (Join-Path $installDir 'mnemosyne.exe') -Force
if (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'uninstall.ps1')) {
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'uninstall.ps1') -Destination (Join-Path $installDir 'uninstall.ps1') -Force
}

# 3. Create Shortcuts (Start Menu & Desktop)
$wsh = New-Object -ComObject WScript.Shell
foreach ($scPath in @($shortcutPath, $desktopShortcutPath)) {
    $shortcut = $wsh.CreateShortcut($scPath)
    $shortcut.TargetPath = Join-Path $installDir 'mnemosyne.exe'
    $shortcut.WorkingDirectory = $installDir
    $shortcut.IconLocation = Join-Path $installDir 'mnemosyne.exe'
    $shortcut.Save()
}
Write-Host "✓ Shortcuts created in Start Menu and Desktop" -ForegroundColor Green

# 4. Register uninstaller in Windows
$uninstallCommand = '"' + (Join-Path $PSHOME 'powershell.exe') + '" -NoProfile -ExecutionPolicy Bypass -File "' + (Join-Path $installDir 'uninstall.ps1') + '"'
New-Item -Path $uninstallKey -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name DisplayName -Value 'Mnemosyne' -PropertyType String -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name DisplayVersion -Value '0.1.0' -PropertyType String -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name InstallLocation -Value $installDir -PropertyType String -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name DisplayIcon -Value (Join-Path $installDir 'mnemosyne.exe') -PropertyType String -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name UninstallString -Value $uninstallCommand -PropertyType String -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name NoModify -Value 1 -PropertyType DWord -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name NoRepair -Value 1 -PropertyType DWord -Force | Out-Null

# 5. Check and install Claude Code CLI if missing
Write-Host ""
Write-Host "Checking Claude Code CLI..." -ForegroundColor Cyan
$claudeInstalled = $false
try {
    $claudeVer = & cmd.exe /c "claude --version" 2>$null
    if ($LASTEXITCODE -eq 0) { $claudeInstalled = $true }
} catch {}

if ($claudeInstalled) {
    Write-Host "✓ Claude Code CLI already installed: $claudeVer" -ForegroundColor Green
} else {
    Write-Host "Claude Code CLI not found. Attempting install via npm..." -ForegroundColor Yellow
    try {
        & cmd.exe /c "npm install -g @anthropic-ai/claude-code"
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✓ Claude Code CLI installed successfully!" -ForegroundColor Green
        } else {
            Write-Host "⚠ npm install failed. You can install it inside Mnemosyne Settings." -ForegroundColor Yellow
        }
    } catch {
        Write-Host "⚠ Node.js/npm not detected. Install Node.js from https://nodejs.org" -ForegroundColor Yellow
    }
}

# 6. Check Decod setup
Write-Host ""
Write-Host "Setting up Decod dependencies..." -ForegroundColor Cyan
$decodDir = Join-Path (Get-Item $PSScriptRoot).Parent.Parent.Parent.FullName 'Decod'
if (-not (Test-Path $decodDir)) {
    $decodDir = 'C:\Users\knyaz\Desktop\Decod'
}
if (Test-Path (Join-Path $decodDir 'setup_decod.py')) {
    try {
        & python (Join-Path $decodDir 'setup_decod.py') --install-all
        Write-Host "✓ Decod environment ready" -ForegroundColor Green
    } catch {
        Write-Host "⚠ Could not run setup_decod.py: $_" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "=========================================" -ForegroundColor Green
Write-Host "  Installation Complete!                 " -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
Write-Host "Launch Mnemosyne from Desktop or Start Menu."
