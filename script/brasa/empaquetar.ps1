# Compila Brasa en release para una arquitectura de Windows y prepara la carpeta dist\Brasa-<version>-<arch>.
# Con -Instalador genera ademas el instalador (Inno Setup 6) y dist\latest.json para el actualizador.
#
#   powershell -File script\brasa\empaquetar.ps1 -Arch x64 [-Instalador] [-BaseUrl https://...] [-SinCompilar]
param(
    [ValidateSet('x64', 'arm64')][string]$Arch = 'x64',
    [switch]$Instalador,
    [string]$BaseUrl = '',
    [switch]$SinCompilar
)

$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path "$PSScriptRoot\..\..").Path
$Version = (Get-Content "$Root\VERSION" -TotalCount 1).Trim()
$Target = if ($Arch -eq 'arm64') { 'aarch64-pc-windows-msvc' } else { 'x86_64-pc-windows-msvc' }
$Out = "$Root\target\$Target\release"
$Dist = "$Root\dist"
$Folder = "$Dist\Brasa-$Version-$Arch"

if (-not $SinCompilar) {
    $env:GIT_RELEASE_TAG = "v$Version"
    $env:CARGO_BIN_NAME = 'oss'
    $env:WARP_APP_NAME = 'Brasa'
    $env:CARGO_FULL_PROFILE = 'release'
    cargo build --release -p warp --bin brasa --features release_bundle --target $Target
    if ($LASTEXITCODE -ne 0) { throw "La compilacion de Brasa ($Arch) ha fallado" }
}

if (Test-Path $Folder) { Remove-Item $Folder -Recurse -Force }
New-Item -ItemType Directory -Path "$Folder\$Arch" -Force | Out-Null

$Assets = "$Root\app\assets\windows\$Arch"
Copy-Item "$Out\brasa.exe" $Folder
foreach ($dll in 'conpty.dll', 'vcruntime140.dll', 'vcruntime140_1.dll', 'msvcp140.dll', 'dxcompiler.dll', 'dxil.dll') {
    Copy-Item "$Assets\$dll" $Folder
}
Copy-Item "$Assets\OpenConsole.exe" "$Folder\$Arch"
Copy-Item "$Root\app\assets\bundled\bootstrap\pwsh.ps1" $Folder
Copy-Item "$Root\brand\icon\icon.ico" $Folder
New-Item -ItemType Directory -Path "$Folder\resources" -Force | Out-Null
Copy-Item "$Root\resources\bundled" "$Folder\resources\bundled" -Recurse
foreach ($license in 'LICENSE-AGPL', 'LICENSE-MIT') {
    Copy-Item "$Root\$license" "$Folder\$license.txt"
}
Write-Output "Carpeta lista: $Folder"

if (-not $Instalador) { return }

$Iscc = @("${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe", "$env:ProgramFiles\Inno Setup 6\ISCC.exe", "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe") |
    Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $Iscc) { throw 'No encuentro Inno Setup 6 (ISCC.exe)' }
& $Iscc "/DVersion=$Version" "/DArch=$Arch" "/DSource=$Folder" "/DOutDir=$Dist" "$PSScriptRoot\brasa.iss"
if ($LASTEXITCODE -ne 0) { throw 'Inno Setup ha fallado' }

$Setup = "$Dist\Brasa-Setup-$Version-$Arch.exe"
$Sha = (Get-FileHash $Setup -Algorithm SHA256).Hash.ToLowerInvariant()
$Url = if ($BaseUrl) { "$BaseUrl/Brasa-Setup-$Version-$Arch.exe" } else { "Brasa-Setup-$Version-$Arch.exe" }
$Manifest = [ordered]@{ version = $Version; $Arch = [ordered]@{ url = $Url; sha256 = $Sha } }
$Manifest | ConvertTo-Json -Depth 3 | Set-Content "$Dist\latest.json" -Encoding utf8
Write-Output "Instalador: $Setup"
