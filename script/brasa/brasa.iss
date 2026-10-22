; Instalador de Brasa para Windows (Inno Setup 6). Lo genera script\brasa\empaquetar.ps1, que pasa
; la version, la arquitectura y la carpeta ya preparada:
;   ISCC.exe /DVersion=0.0.1 /DArch=x64 /DSource=..\..\dist\Brasa-0.0.1-x64 /DOutDir=..\..\dist brasa.iss
;
; Se instala para el usuario actual (sin pedir administrador) en %LOCALAPPDATA%\Programs\Brasa, con
; acceso en el menu Inicio (y en el escritorio si se marca), y se desinstala desde Configuracion >
; Aplicaciones.

#ifndef Version
  #define Version "0.0.1"
#endif
#ifndef Arch
  #define Arch "x64"
#endif
#ifndef Source
  #define Source "..\..\dist\Brasa-" + Version + "-" + Arch
#endif
#ifndef OutDir
  #define OutDir "..\..\dist"
#endif

[Setup]
; El AppId identifica a Brasa para actualizar y desinstalar: no cambiarlo nunca.
AppId={{8C4E2F71-3A9B-4D6E-B15F-7E2A9C0D4B63}
AppName=Brasa
AppVersion={#Version}
AppVerName=Brasa {#Version}
AppPublisher=SifG22
AppPublisherURL=https://sifg.net
AppSupportURL=https://github.com/SifG22/brasa/issues
AppUpdatesURL=https://github.com/SifG22/brasa/releases
DefaultDirName={autopf}\Brasa
DefaultGroupName=Brasa
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
#if Arch == "arm64"
ArchitecturesAllowed=arm64
ArchitecturesInstallIn64BitMode=arm64
#else
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
#endif
MinVersion=10.0.17763
SetupIconFile=..\..\brand\icon\icon.ico
UninstallDisplayIcon={app}\brasa.exe
UninstallDisplayName=Brasa
LicenseFile=..\..\LICENSE-AGPL
OutputDir={#OutDir}
OutputBaseFilename=Brasa-Setup-{#Version}-{#Arch}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
VersionInfoVersion={#Version}
VersionInfoCompany=SifG22
VersionInfoDescription=Instalador de Brasa
VersionInfoProductName=Brasa

[Languages]
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "en"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "{#Source}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\Brasa"; Filename: "{app}\brasa.exe"; AppUserModelID: "net.sifg.Brasa"
Name: "{autodesktop}\Brasa"; Filename: "{app}\brasa.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\brasa.exe"; Description: "{cm:LaunchProgram,Brasa}"; Flags: nowait postinstall skipifsilent
