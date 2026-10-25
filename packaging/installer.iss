; Inkk Professional Windows Installer Script (Inno Setup)
#define MyAppName "Inkk"
#define MyAppVersion "0.1.2"
#define MyAppPublisher "NTDD-code"
#define MyAppURL "https://github.com/NTDD-code/inkk-notes"
#define MyAppExeName "inkk.exe"

[Setup]
AppId={{E68F9A1B-32A4-4A8E-95B7-512E4E1C2B01}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
LicenseFile=..\LICENSE-MIT
OutputDir=..\dist
OutputBaseFilename=Inkk-v{#MyAppVersion}-Setup
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
SetupIconFile=..\assets\icon.ico
UninstallDisplayIcon={app}\{#MyAppExeName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\dist\Inkk-v{#MyAppVersion}-Windows-x64\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\dist\Inkk-v{#MyAppVersion}-Windows-x64\assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\dist\Inkk-v{#MyAppVersion}-Windows-x64\README.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\dist\Inkk-v{#MyAppVersion}-Windows-x64\update_manifest.json"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\dist\Inkk-v{#MyAppVersion}-Windows-x64\LICENSE-MIT"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\dist\Inkk-v{#MyAppVersion}-Windows-x64\LICENSE-APACHE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\icon.ico"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\icon.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
