$ErrorActionPreference = 'Stop'
$installDir = Join-Path $env:LOCALAPPDATA 'Programs\Mnemosyne'
$startMenuDir = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
$shortcutPath = Join-Path $startMenuDir 'Mnemosyne.lnk'
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Mnemosyne'
$binaryPath = Join-Path $PSScriptRoot 'mnemosyne.exe'

if (-not (Test-Path -LiteralPath $binaryPath -PathType Leaf)) {
    throw 'mnemosyne.exe is missing. Extract the complete package before installing.'
}

New-Item -ItemType Directory -Force -Path $installDir, $startMenuDir | Out-Null
Copy-Item -LiteralPath $binaryPath -Destination (Join-Path $installDir 'mnemosyne.exe') -Force
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'uninstall.ps1') -Destination (Join-Path $installDir 'uninstall.ps1') -Force

$wsh = New-Object -ComObject WScript.Shell
$shortcut = $wsh.CreateShortcut($shortcutPath)
$shortcut.TargetPath = Join-Path $installDir 'mnemosyne.exe'
$shortcut.WorkingDirectory = $installDir
$shortcut.IconLocation = Join-Path $installDir 'mnemosyne.exe'
$shortcut.Save()

$uninstallCommand = '"' + (Join-Path $PSHOME 'powershell.exe') + '" -NoProfile -ExecutionPolicy Bypass -File "' + (Join-Path $installDir 'uninstall.ps1') + '"'
New-Item -Path $uninstallKey -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name DisplayName -Value 'Mnemosyne' -PropertyType String -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name DisplayVersion -Value '0.1.0' -PropertyType String -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name InstallLocation -Value $installDir -PropertyType String -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name DisplayIcon -Value (Join-Path $installDir 'mnemosyne.exe') -PropertyType String -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name UninstallString -Value $uninstallCommand -PropertyType String -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name NoModify -Value 1 -PropertyType DWord -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name NoRepair -Value 1 -PropertyType DWord -Force | Out-Null

Write-Host "Mnemosyne installed to $installDir"
Write-Host 'Launch Mnemosyne from the Start menu or uninstall it from Windows Settings > Apps.'
