; RYOCS installer. First: cargo build --release
; Build: "%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe" kurulum\ryocs.iss  ->  target\kurulum\RYOCS-Setup-<version>.exe
; CI passes the version with /DSurum=x.y.z
#ifndef Surum
  #define Surum "1.1.10"
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
; Windows 10 1709 or newer: the app calls GetUserDefaultGeoName (1709) and Windows CNG pseudo-handles; on older Windows the exe would not start
MinVersion=10.0.16299
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
en.WebView2=Installing the Microsoft Edge WebView2 Runtime (needed by RYOCS)...
tr.WebView2=Microsoft Edge WebView2 Runtime kuruluyor (RYOCS için gerekli)...

[Tasks]
Name: "masaustu"; Description: "{cm:Masaustu}"; Flags: unchecked

[Files]
Source: "..\target\release\ryocs.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\yazi\OFL.txt"; DestDir: "{app}"; DestName: "Poppins-OFL.txt"; Flags: ignoreversion
; WebView2 Runtime: Windows 11 has it, some Windows 10 (LTSC, N) do not. CI puts Microsoft's bootstrapper next to the exe;
; it downloads and installs the runtime only when it is missing (needs internet). A local build without the file still compiles.
Source: "..\target\MicrosoftEdgeWebview2Setup.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall skipifsourcedoesntexist; Check: WebView2Yok

[Icons]
; The app ID makes notifications show under the "RYOCS" name and groups correctly on the taskbar
Name: "{autoprograms}\RYOCS"; Filename: "{app}\ryocs.exe"; AppUserModelID: "RYOCS"
Name: "{autodesktop}\RYOCS"; Filename: "{app}\ryocs.exe"; AppUserModelID: "RYOCS"; Tasks: masaustu

[Registry]
; the notification name key the app writes at startup is removed on uninstall
Root: HKCU; Subkey: "Software\Classes\AppUserModelId\RYOCS"; Flags: uninsdeletekey

[Run]
Filename: "{tmp}\MicrosoftEdgeWebview2Setup.exe"; Parameters: "/silent /install"; StatusMsg: "{cm:WebView2}"; Flags: waituntilterminated; Check: WebView2Kur
Filename: "{app}\ryocs.exe"; Description: "{cm:Baslat}"; Flags: nowait postinstall skipifsilent
; one-click update in the app runs this installer with /SILENT: start RYOCS again afterwards
Filename: "{app}\ryocs.exe"; Flags: nowait; Check: WizardSilent

[UninstallRun]
; no "Start with Windows" entry is left behind; personal data (%APPDATA%\RYOCS) is deliberately not deleted
Filename: "{sys}\reg.exe"; Parameters: "delete HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v RYOCS /f"; Flags: runhidden; RunOnceId: "baslat"

[UninstallDelete]
; the web view's own folder (browser cache and per-computer conveniences) and copies left by manual updates
Type: filesandordirs; Name: "{app}\ryocs.exe.WebView2"
Type: files; Name: "{app}\ryocs.*.exe"

[Code]
// WebView2 Runtime installed (per machine or per user)? Microsoft's documented check: the "pv" version under EdgeUpdate\Clients
function WebView2Var(Kok: Integer; Anahtar: String): Boolean;
var V: String;
begin
  Result := RegQueryStringValue(Kok, Anahtar, 'pv', V) and (V <> '') and (V <> '0.0.0.0');
end;
function WebView2Yok(): Boolean;
begin
  Result := not (WebView2Var(HKLM, 'SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}')
    or WebView2Var(HKLM, 'SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}')
    or WebView2Var(HKCU, 'Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'));
end;
function WebView2Kur(): Boolean;
begin
  Result := WebView2Yok() and FileExists(ExpandConstant('{tmp}\MicrosoftEdgeWebview2Setup.exe'));
end;
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
