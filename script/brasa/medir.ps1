# Mide arranque en frío, tiempo hasta ventana lista y memoria de una app de terminal.
# Uso: powershell -File script\brasa\medir.ps1 -Exe <ruta.exe> [-Runs 3]
param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [int]$Runs = 3,
    [int]$IdleSeconds = 8
)

$ErrorActionPreference = 'Stop'
$name = [IO.Path]::GetFileNameWithoutExtension($Exe)
$results = @()

for ($i = 1; $i -le $Runs; $i++) {
    Get-Process -Name $name -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Seconds 2

    $sw = [Diagnostics.Stopwatch]::StartNew()
    $p = Start-Process -FilePath $Exe -PassThru
    $window = $null
    while ($sw.Elapsed.TotalSeconds -lt 60) {
        $p.Refresh()
        if ($p.MainWindowHandle -ne 0) { $window = $sw.Elapsed.TotalMilliseconds; break }
        Start-Sleep -Milliseconds 10
    }

    # "Listo para escribir": la CPU del proceso deja de crecer durante 300 ms seguidos.
    $lastCpu = -1; $stableSince = $null; $ready = $null
    while ($sw.Elapsed.TotalSeconds -lt 60) {
        $p.Refresh()
        $cpu = $p.TotalProcessorTime.TotalMilliseconds
        if ($cpu -eq $lastCpu) {
            if (-not $stableSince) { $stableSince = $sw.Elapsed.TotalMilliseconds }
            elseif ($sw.Elapsed.TotalMilliseconds - $stableSince -ge 300) { $ready = $stableSince; break }
        } else { $stableSince = $null }
        $lastCpu = $cpu
        Start-Sleep -Milliseconds 25
    }

    Start-Sleep -Seconds $IdleSeconds
    $procs = Get-Process -Name $name -ErrorAction SilentlyContinue
    $mem = ($procs | Measure-Object -Property WorkingSet64 -Sum).Sum / 1MB
    $priv = ($procs | Measure-Object -Property PrivateMemorySize64 -Sum).Sum / 1MB
    $results += [pscustomobject]@{
        Run = $i
        VentanaMs = [int]$window
        ListoMs = [int]$ready
        MemoriaMB = [int]$mem
        PrivadaMB = [int]$priv
    }
    $procs | Stop-Process -Force
}

$results | Format-Table -AutoSize
$results | Measure-Object -Property VentanaMs, ListoMs, MemoriaMB, PrivadaMB -Average |
    Select-Object Property, @{n = 'Media'; e = { [int]$_.Average } } | Format-Table -AutoSize
