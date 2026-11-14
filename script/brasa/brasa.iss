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
; Sin página de licencia: LICENSE-AGPL.txt y LICENSE-MIT.txt se copian a la carpeta del programa (empaquetar.ps1).
OutputDir={#OutDir}
OutputBaseFilename=Brasa-Setup-{#Version}-{#Arch}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
; Avisa a Windows al terminar para que refresque los iconos (si no, el escritorio puede seguir
; mostrando el logo de la versión anterior).
ChangesAssociations=yes
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
; Al actualizar desde Brasa se vuelve a abrir solo.
Filename: "{app}\brasa.exe"; Flags: nowait; Check: IsUpdateFromBrasa

[Code]
// El actualizador de Brasa abre este instalador con /SILENT ... /RESTARTAPPLICATIONS.
function IsUpdateFromBrasa: Boolean;
begin
  Result := WizardSilent and (Pos('/RESTARTAPPLICATIONS', Uppercase(GetCmdTail)) > 0);
end;

function BrasaIsRunning: Boolean;
var
  Code: Integer;
begin
  // find termina con 0 cuando tasklist muestra algún brasa.exe.
  Result := Exec(ExpandConstant('{cmd}'), '/C tasklist /FI "IMAGENAME eq brasa.exe" /NH | find /I "brasa.exe" >NUL',
    '', SW_HIDE, ewWaitUntilTerminated, Code) and (Code = 0);
end;

// Al actualizar, Brasa sigue abierto y sus consolas usan archivos de la carpeta del programa, así
// que el instalador no podría reemplazarlos. Se cierra Brasa antes de copiar nada. Sin /T: este
// instalador lo ha abierto Brasa y es proceso hijo suyo.
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  I, Code: Integer;
begin
  Result := '';
  if not IsUpdateFromBrasa or not BrasaIsRunning then
    Exit;
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM brasa.exe', '', SW_HIDE, ewWaitUntilTerminated, Code);
  for I := 1 to 20 do
  begin
    if not BrasaIsRunning then
      Break;
    Sleep(500);
  end;
  // Las consolas de Brasa (OpenConsole.exe) se cierran un momento después que Brasa.
  Sleep(2000);
end;
