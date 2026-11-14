; Instalador universal de Brasa (Inno Setup 6).
;
; Es un instalador pequeño que no hace falta cambiar nunca: mira si el PC es x64 o ARM64, lee
; latest.json de la última release de SifG22/brasa (el mismo aviso que usa el actualizador de la
; app), descarga el instalador que toca, comprueba su SHA256 y lo ejecuta. Así el enlace de descarga
; siempre da la última versión.
;
;   ISCC.exe /DOutDir=..\..\dist web.iss        -> dist\Brasa-Setup.exe
;   ISCC.exe /DDryRun ...                        -> descarga y comprueba, pero no instala

#ifndef OutDir
  #define OutDir "..\..\dist"
#endif
#define LatestUrl "https://github.com/SifG22/brasa/releases/latest/download/"

[Setup]
AppId=BrasaWebInstaller
AppName=Brasa
AppVerName=Brasa
AppPublisher=SifG22
AppPublisherURL=https://sifg.net
AppSupportURL=https://github.com/SifG22/brasa/issues
VersionInfoVersion=1.0.0
VersionInfoCompany=SifG22
VersionInfoDescription=Instalador de Brasa
VersionInfoProductName=Brasa
VersionInfoCopyright=© 2026 SifG22
CreateAppDir=no
Uninstallable=no
PrivilegesRequired=lowest
DisableWelcomePage=yes
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ShowLanguageDialog=no
LanguageDetectionMethod=uilanguage
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes
OutputDir={#OutDir}
OutputBaseFilename=Brasa-Setup
SetupIconFile=..\..\brand\icon\icon.ico

[Languages]
; Los mismos nombres que en brasa.iss, para pasarle el idioma con /LANG.
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "en"; MessagesFile: "compiler:Default.isl"

[Messages]
es.ReadyLabel2b=Se descargará la última versión de Brasa para tu PC y se instalará. Es gratis y no necesita cuenta.%n%nPulsa Instalar para continuar.
en.ReadyLabel2b=Setup will download the latest version of Brasa for your PC and install it. It is free and needs no account.%n%nClick Install to continue.

[CustomMessages]
es.Looking=Buscando la última versión...
en.Looking=Looking for the latest version...
es.Downloading=Descargando Brasa %1...
en.Downloading=Downloading Brasa %1...
es.Installing=Instalando Brasa %1...
en.Installing=Installing Brasa %1...
es.Wait=Tarda alrededor de un minuto.
en.Wait=This takes about a minute.
es.DownloadFailed=No se pudo descargar Brasa. Comprueba tu conexión a internet y vuelve a intentarlo.%n%n%1
en.DownloadFailed=Brasa could not be downloaded. Check your internet connection and try again.%n%n%1
es.NotFound=La última versión no tiene instalador para este PC (%1).
en.NotFound=No installer for this PC (%1) was found in the latest version.
es.InstallFailed=La instalación de Brasa no terminó (código %1).
en.InstallFailed=The installation of Brasa did not finish (code %1).
es.Launch=Abrir Brasa
en.Launch=Open Brasa

[Run]
Filename: "{code:GetBrasaExe}"; Description: "{cm:Launch}"; Flags: nowait postinstall skipifsilent skipifdoesntexist

[Code]
var
  DownloadPage: TDownloadWizardPage;
  Arch: String;

function OnDownloadProgress(const Url, FileName: String; const Progress, ProgressMax: Int64): Boolean;
begin
  Result := True;
end;

procedure InitializeWizard;
begin
  if IsArm64 then
    Arch := 'arm64'
  else
    Arch := 'x64';
  DownloadPage := CreateDownloadPage(SetupMessage(msgWizardPreparing), CustomMessage('Wait'), @OnDownloadProgress);
end;

{ El valor de texto de "Key" que aparece después de la posición From en Text, o '' si no está }
function JsonValue(const Text, Key: String; From: Integer): String;
var
  Rest: String;
  P: Integer;
begin
  Result := '';
  Rest := Copy(Text, From, MaxInt);
  P := Pos('"' + Key + '"', Rest);
  if P = 0 then
    Exit;
  Rest := Copy(Rest, P + Length(Key) + 2, MaxInt);
  P := Pos(':', Rest);
  if P = 0 then
    Exit;
  Rest := Copy(Rest, P + 1, MaxInt);
  P := Pos('"', Rest);
  if P = 0 then
    Exit;
  Rest := Copy(Rest, P + 1, MaxInt);
  P := Pos('"', Rest);
  if P = 0 then
    Exit;
  Result := Copy(Rest, 1, P - 1);
end;

// Busca en latest.json el instalador de este PC. El archivo tiene esta forma:
// "version": ..., "x64": ("url": ..., "sha256": ...), "arm64": (...)
function FindInstaller(const LatestFile: String; var Url, FileName, Hash, Version: String): Boolean;
var
  Lines: TArrayOfString;
  Text: String;
  I, ArchPos: Integer;
begin
  Result := False;
  if not LoadStringsFromFile(LatestFile, Lines) then
    Exit;
  Text := '';
  for I := 0 to GetArrayLength(Lines) - 1 do
    Text := Text + Lines[I] + ' ';
  Version := JsonValue(Text, 'version', 1);
  ArchPos := Pos('"' + Arch + '"', Text);
  if ArchPos = 0 then
    Exit;
  Url := JsonValue(Text, 'url', ArchPos);
  Hash := Lowercase(JsonValue(Text, 'sha256', ArchPos));
  FileName := Url;
  while Pos('/', FileName) > 0 do
    FileName := Copy(FileName, Pos('/', FileName) + 1, MaxInt);
  Result := (Url <> '') and (Length(Hash) = 64) and (FileName <> '');
end;

function DownloadAndInstall: Boolean;
var
  Url, FileName, Hash, Version: String;
  ResultCode: Integer;
begin
  Result := False;

  DownloadPage.Clear;
  DownloadPage.SetText(CustomMessage('Looking'), CustomMessage('Wait'));
  DownloadPage.Add('{#LatestUrl}latest.json', 'latest.json', '');
  DownloadPage.Download;

  if not FindInstaller(ExpandConstant('{tmp}\latest.json'), Url, FileName, Hash, Version) then
    RaiseException(FmtMessage(CustomMessage('NotFound'), [Arch]));

  DownloadPage.Clear;
  DownloadPage.SetText(FmtMessage(CustomMessage('Downloading'), [Version]), CustomMessage('Wait'));
  DownloadPage.Add(Url, FileName, Hash);
  DownloadPage.Download;

  DownloadPage.AbortButton.Hide;
  DownloadPage.SetText(FmtMessage(CustomMessage('Installing'), [Version]), CustomMessage('Wait'));
  DownloadPage.ProgressBar.Style := npbstMarquee;
#ifdef DryRun
  Log('Prueba: se ejecutaría ' + FileName + ' (' + Hash + ')');
  ResultCode := 0;
#else
  if not Exec(ExpandConstant('{tmp}\') + FileName, '/SILENT /SP- /NORESTART /CURRENTUSER /LANG=' + ActiveLanguage, '', SW_SHOW, ewWaitUntilTerminated, ResultCode) then
    RaiseException(SysErrorMessage(ResultCode));
#endif
  if ResultCode <> 0 then
    RaiseException(FmtMessage(CustomMessage('InstallFailed'), [IntToStr(ResultCode)]));

  Result := True;
end;

function NextButtonClick(CurPageID: Integer): Boolean;
begin
  Result := True;
  if CurPageID <> wpReady then
    Exit;

  DownloadPage.Show;
  try
    try
      Result := DownloadAndInstall;
    except
      if not DownloadPage.AbortedByUser then
        SuppressibleMsgBox(FmtMessage(CustomMessage('DownloadFailed'), [GetExceptionMessage]), mbCriticalError, MB_OK, IDOK);
      Result := False;
    end;
  finally
    DownloadPage.ProgressBar.Style := npbstNormal;
    DownloadPage.Hide;
  end;
end;

{ Dónde ha quedado Brasa: el instalador completo reutiliza la carpeta de una instalación anterior }
function GetBrasaExe(Param: String): String;
var
  Location: String;
begin
  if not RegQueryStringValue(HKCU, 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{8C4E2F71-3A9B-4D6E-B15F-7E2A9C0D4B63}_is1', 'InstallLocation', Location) then
    Location := ExpandConstant('{localappdata}\Programs\Brasa');
  Result := AddBackslash(Location) + 'brasa.exe';
end;
