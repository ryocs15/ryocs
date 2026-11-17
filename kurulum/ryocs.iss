; RYOCS installer. First: cargo build --release
; Build: "%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe" kurulum\ryocs.iss  ->  target\kurulum\RYOCS-Setup-<version>.exe
; CI passes the version with /DSurum=x.y.z
#ifndef Surum
  #define Surum "0.5.0"
#endif

[Setup]
AppId={{6F2B8C1E-4A7D-4E3B-9C55-2D1A8E7F0B34}
AppName=RYOCS
AppVersion={#Surum}
AppPublisher=RYOCS
AppPublisherURL=https://github.com/ryocs15/ryocs
DefaultDirName={localappdata}\Programs\RYOCS
DisableProgramGroupPage=yes
DisableDirPage=yes
; does not ask for admin rights: installs for this user only
PrivilegesRequired=lowest
OutputDir=..\target\kurulum
OutputBaseFilename=RYOCS-Setup-{#Surum}
SetupIconFile=..\assets\ryocs.ico
UninstallDisplayIcon={app}\ryocs.exe
UninstallDisplayName=RYOCS
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
; the installer language follows the Windows UI language (English unless Turkish)
ShowLanguageDialog=no
LicenseFile=..\LICENSE

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "tr"; MessagesFile: "compiler:Languages\Turkish.isl"

[CustomMessages]
en.Masaustu=Create a desktop shortcut
tr.Masaustu=Masaüstüne kısayol oluştur
en.Baslat=Start RYOCS
tr.Baslat=RYOCS'u başlat

[Tasks]
Name: "masaustu"; Description: "{cm:Masaustu}"; Flags: unchecked

[Files]
Source: "..\target\release\ryocs.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
; The app ID makes notifications show under the "RYOCS" name and groups correctly on the taskbar
Name: "{autoprograms}\RYOCS"; Filename: "{app}\ryocs.exe"; AppUserModelID: "RYOCS"
Name: "{autodesktop}\RYOCS"; Filename: "{app}\ryocs.exe"; AppUserModelID: "RYOCS"; Tasks: masaustu

[Registry]
; the notification name key the app writes at startup is removed on uninstall
Root: HKCU; Subkey: "Software\Classes\AppUserModelId\RYOCS"; Flags: uninsdeletekey

[Run]
Filename: "{app}\ryocs.exe"; Description: "{cm:Baslat}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; no "Start with Windows" entry is left behind; personal data (%APPDATA%\RYOCS) is deliberately not deleted
Filename: "{sys}\reg.exe"; Parameters: "delete HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v RYOCS /f"; Flags: runhidden; RunOnceId: "baslat"

[Code]
// A RYOCS running in the tray locks the file: it is closed before install and uninstall
procedure Kapat();
var Sonuc: Integer;
begin
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/IM ryocs.exe /F', '', SW_HIDE, ewWaitUntilTerminated, Sonuc);
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  Kapat();
  Result := '';
end;

function InitializeUninstall(): Boolean;
begin
  Kapat();
  Result := True;
end;
