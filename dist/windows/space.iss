; Space for Windows — per-user installer (Inno Setup 6).
;
; Built by scripts/package-windows.ps1, which passes the version, the package
; architecture, and the staged portable directory:
;   ISCC.exe /DAppVersion=0.2.97 /DArch=x86_64 /DPackageDir=<stage> /DOutputDir=<out> space.iss
;
; Installs into %LOCALAPPDATA%\Programs\Space without elevation, like VS
; Code's user setup: the directory stays writable by its user, so the in-app
; updater (crates/update/src/windows.rs) can replace space.exe in place. The
; staged directory already carries space-update.json, which marks the install
; as update-managed. Re-running a newer installer upgrades in place; user data
; lives in %LOCALAPPDATA%\Space and is never touched here.

#ifndef AppVersion
  #error AppVersion must be defined (/DAppVersion=x.y.z)
#endif
#ifndef Arch
  #error Arch must be defined (/DArch=x86_64 or /DArch=aarch64)
#endif
#ifndef PackageDir
  #error PackageDir must be defined (/DPackageDir=<staged package directory>)
#endif
#ifndef OutputDir
  #define OutputDir "."
#endif

#if Arch == "aarch64"
  #define ArchAllowed "arm64"
#else
  #define ArchAllowed "x64compatible"
#endif

[Setup]
; Never change AppId: it identifies the installation across upgrades, and
; crates/update/src/windows.rs refreshes DisplayVersion under this key after
; in-app updates.
AppId={{AD5DEC34-E254-467B-8F24-8127EBAF4DA6}
AppName=Space
AppVersion={#AppVersion}
AppVerName=Space {#AppVersion}
AppPublisher=pentomake
AppPublisherURL=https://space.pentomake.com
AppSupportURL=https://github.com/strongd3v/test/issues
AppUpdatesURL=https://github.com/strongd3v/test/releases
VersionInfoVersion={#AppVersion}
PrivilegesRequired=lowest
DefaultDirName={autopf}\Space
DisableProgramGroupPage=yes
DisableDirPage=auto
DisableReadyPage=yes
ArchitecturesAllowed={#ArchAllowed}
ArchitecturesInstallIn64BitMode={#ArchAllowed}
MinVersion=10.0
OutputDir={#OutputDir}
OutputBaseFilename=space-{#AppVersion}-windows-setup
SetupIconFile=space.ico
UninstallDisplayIcon={app}\space.exe
UninstallDisplayName=Space
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes
; A running Space is closed through the Restart Manager before its files are
; replaced; the updated app starts again from the finish page.
CloseApplications=yes
RestartApplications=no

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#PackageDir}\space.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PackageDir}\space-update.json"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PackageDir}\LICENSE.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PackageDir}\licenses\*"; DestDir: "{app}\licenses"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\Space"; Filename: "{app}\space.exe"
Name: "{autodesktop}\Space"; Filename: "{app}\space.exe"; Tasks: desktopicon

[Registry]
; space:// conversation links — the scheme macOS registers in Info.plist and
; Linux in space.desktop.
Root: HKCU; Subkey: "Software\Classes\space"; ValueType: string; ValueName: ""; ValueData: "URL:Space"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\space"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""
Root: HKCU; Subkey: "Software\Classes\space\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: """{app}\space.exe"",0"
Root: HKCU; Subkey: "Software\Classes\space\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\space.exe"" ""%1"""

[Run]
Filename: "{app}\space.exe"; Description: "{cm:LaunchProgram,Space}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; Leftovers of in-app updates (crates/update/src/windows.rs).
Type: files; Name: "{app}\space.exe.old"
Type: files; Name: "{app}\.space-update-incoming.exe"
Type: filesandordirs; Name: "{app}\.space-update-*"
