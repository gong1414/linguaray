#ifndef SourceDir
  #error SourceDir must point to the Flutter Windows release directory.
#endif

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
AppId={{9DD27DE6-0D88-4CA9-8150-6454E5F2951D}
AppName=LinguaRay
AppVersion={#AppVersion}
AppPublisher=LinguaRay contributors
AppPublisherURL=https://github.com/gong1414/linguaray
AppSupportURL=https://github.com/gong1414/linguaray/issues
AppUpdatesURL=https://github.com/gong1414/linguaray/releases
DefaultDirName={localappdata}\Programs\LinguaRay
DefaultGroupName=LinguaRay
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputBaseFilename=LinguaRay-windows-x64
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
SetupIconFile=..\..\runner\resources\app_icon.ico
UninstallDisplayIcon={app}\linguaray.exe
CloseApplications=yes
RestartApplications=no
AppMutex=Local\LinguaRaySingleInstance
VersionInfoVersion={#AppVersion}
VersionInfoCompany=LinguaRay contributors
VersionInfoDescription=LinguaRay desktop installer
VersionInfoProductName=LinguaRay
VersionInfoProductVersion={#AppVersion}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "chinesesimplified"; MessagesFile: "Languages\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

Source: "Languages\InnoSetup-LICENSE.txt"; DestDir: "{app}\licenses"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\LinguaRay"; Filename: "{app}\linguaray.exe"
Name: "{autodesktop}\LinguaRay"; Filename: "{app}\linguaray.exe"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Classes\linguaray"; ValueType: string; ValueName: ""; ValueData: "URL:LinguaRay Protocol"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\linguaray"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""
Root: HKCU; Subkey: "Software\Classes\linguaray\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\linguaray.exe,0"
Root: HKCU; Subkey: "Software\Classes\linguaray\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\linguaray.exe"" ""%1"""

[Run]
Filename: "{app}\linguaray.exe"; Description: "{cm:LaunchProgram,LinguaRay}"; Flags: nowait postinstall skipifsilent
