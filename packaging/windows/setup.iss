; Inno Setup Script for Mnemosyne
; Requires Inno Setup (https://jrsoftware.org/isinfo.php)

[Setup]
AppId={{5E97D4B6-6B2A-4D4F-9E01-9F2F7E41C1A0}
AppName=Mnemosyne
AppVersion=0.1.0
AppPublisher=Vorwerk
AppPublisherURL=https://github.com/VorwerkUa2/Mnemosyne
AppSupportURL=https://github.com/VorwerkUa2/Mnemosyne
AppUpdatesURL=https://github.com/VorwerkUa2/Mnemosyne/releases
DefaultDirName={autopf}\Mnemosyne
DisableProgramGroupPage=yes
OutputDir=..\..\
OutputBaseFilename=Mnemosyne-InnoSetup
Compression=lzma
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "ukrainian"; MessagesFile: "compiler:Languages\Ukrainian.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\..\target\release\mnemosyne.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: isreadme ignoreversion

[Icons]
Name: "{autoprograms}\Mnemosyne"; Filename: "{app}\mnemosyne.exe"
Name: "{autodesktop}\Mnemosyne"; Filename: "{app}\mnemosyne.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\mnemosyne.exe"; Description: "{cm:LaunchProgram,Mnemosyne}"; Flags: nowait postinstall skipifsilent
