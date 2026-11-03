; Compile through scripts/package-windows.ps1. Keep this AppId stable across
; versions and architectures: one current-user installation owns the protocol.
#if VER < EncodeVer(6, 3, 0)
  #error Inno Setup 6.3 or later is required
#endif
#ifndef AppVersion
  #error AppVersion is required
#endif
#ifndef AppTarget
  #error AppTarget is required
#endif
#ifndef PublishDir
  #error PublishDir is required
#endif
#if AppTarget == "win-x64"
  #define AppArchitecture "x64compatible"
#elif AppTarget == "win-arm64"
  #define AppArchitecture "arm64"
#else
  #error Unsupported AppTarget
#endif
#define AppGuid "8068245C-BA48-47BE-A55D-C8C7F484C506"
#define UninstallKey "Software\Microsoft\Windows\CurrentVersion\Uninstall\{" + AppGuid + "}_is1"
#define InstallerKey "Software\SunCode\Installer\" + AppGuid

[Setup]
#if VER >= EncodeVer(7, 0, 0)
SetupArchitecture=x86
#endif
AppId={{{#AppGuid}}
AppName=SunCode
AppVersion={#AppVersion}
AppPublisher=SunCode
AppPublisherURL=https://www.suncode.ai
AppSupportURL=https://www.suncode.ai
DefaultDirName={localappdata}\Programs\SunCode
DefaultGroupName=SunCode
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed={#AppArchitecture}
ArchitecturesInstallIn64BitMode={#AppArchitecture}
MinVersion=10.0.17763
WizardStyle=modern
SetupIconFile=..\..\Assets\suncode.ico
UninstallDisplayIcon={app}\SunCode.exe
UninstallDisplayName=SunCode
VersionInfoVersion={#AppVersion}.0
Compression=lzma2
SolidCompression=yes
DiskSpanning=no
CloseApplications=yes
CloseApplicationsFilter=*.exe,*.dll
RestartApplications=no
SetupLogging=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#PublishDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs; Excludes: "*.pdb,*.dbg,*.dSYM\*,share\*"
Source: "..\..\..\..\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\SunCode"; Filename: "{app}\SunCode.exe"; WorkingDir: "{app}"
Name: "{autodesktop}\SunCode"; Filename: "{app}\SunCode.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Classes\suncode"; ValueType: string; ValueName: ""; ValueData: "URL:SunCode Activation Protocol"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\suncode"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""
Root: HKCU; Subkey: "Software\Classes\suncode\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\SunCode.exe,0"
Root: HKCU; Subkey: "Software\Classes\suncode\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\SunCode.exe"" ""%1"""
Root: HKCU; Subkey: "{#InstallerKey}"; ValueType: string; ValueName: "SunCodeTarget"; ValueData: "{#AppTarget}"; Flags: uninsdeletekey

[Run]
Filename: "{app}\SunCode.exe"; Description: "{cm:LaunchProgram,SunCode}"; Flags: nowait postinstall skipifsilent unchecked

[Code]
function InitializeSetup(): Boolean;
var
  InstalledDirectory, InstalledTarget, InstalledVersionText: String;
  InstalledVersion, SetupVersion: Int64;
begin
  Result := False;
  if RegQueryStringValue(HKCU, '{#InstallerKey}', 'SunCodeTarget', InstalledTarget) and
     (InstalledTarget <> '{#AppTarget}') then
  begin
    SuppressibleMsgBox('Uninstall the existing SunCode installation before changing architecture. Your session data will be kept.', mbError, MB_OK, IDOK);
    exit;
  end;

  if not StrToVersion('{#AppVersion}', SetupVersion) then exit;
  if (RegQueryStringValue(HKCU, '{#UninstallKey}', 'DisplayVersion', InstalledVersionText) and
      StrToVersion(InstalledVersionText, InstalledVersion) and
      (ComparePackedVersion(InstalledVersion, SetupVersion) > 0)) or
     (RegQueryStringValue(HKCU, '{#UninstallKey}', 'InstallLocation', InstalledDirectory) and
      GetPackedVersion(AddBackslash(InstalledDirectory) + 'SunCode.exe', InstalledVersion) and
      (ComparePackedVersion(InstalledVersion, SetupVersion) > 0)) then
  begin
    SuppressibleMsgBox('A newer version of SunCode is already installed. Installing an older version could make existing session data unreadable.', mbError, MB_OK, IDOK);
    exit;
  end;
  Result := True;
end;

// No [UninstallDelete] wildcard: remove only installer-owned files. Sessions,
// credentials, downloaded shells and updater state in the data directory survive.
