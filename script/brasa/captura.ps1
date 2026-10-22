# Captura la ventana principal de un proceso (aunque esté detrás de otras) a un PNG.
# Uso: powershell -File script\brasa\captura.ps1 -Proceso brasa -Salida captura.png
param(
    [string]$Proceso = 'brasa',
    [Parameter(Mandatory = $true)][string]$Salida
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System; using System.Runtime.InteropServices;
public class Ventana {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
}
"@
[Ventana]::SetProcessDPIAware() | Out-Null
$p = Get-Process $Proceso | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
if (-not $p) { throw "No hay ventana de $Proceso" }
$r = New-Object Ventana+RECT
[Ventana]::GetWindowRect($p.MainWindowHandle, [ref]$r) | Out-Null
$bmp = New-Object System.Drawing.Bitmap ($r.R - $r.L), ($r.B - $r.T)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$dc = $g.GetHdc()
[Ventana]::PrintWindow($p.MainWindowHandle, $dc, 2) | Out-Null
$g.ReleaseHdc($dc)
$bmp.Save($Salida)
"$($bmp.Width)x$($bmp.Height) -> $Salida"
