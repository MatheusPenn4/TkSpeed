# TkSpeed — Exportar Diagnóstico (Alpha)
# Gera TkSpeed-Diagnostic.zip na Área de Trabalho com banco, logs e info do sistema.
# NÃO altera o TkSpeed. Uso: clique direito > "Executar com PowerShell".
# Para suporte remoto: envie o .zip gerado ao time TkSpeed.

$ErrorActionPreference = "SilentlyContinue"

$appData   = Join-Path $env:APPDATA "TkSpeed"
$stamp     = Get-Date -Format "yyyyMMdd-HHmmss"
$staging   = Join-Path $env:TEMP ("tkspeed-diag-" + $stamp)
$outZip    = Join-Path ([Environment]::GetFolderPath("Desktop")) "TkSpeed-Diagnostic.zip"

New-Item -ItemType Directory -Force $staging | Out-Null

Write-Host "Coletando diagnóstico do TkSpeed..." -ForegroundColor Cyan

# 1) Banco de dados (hardware, score, findings, benchmarks, histórico de otimizações).
#    Inclui WAL/SHM para capturar dados não-commitados (ideal: fechar o app antes).
foreach ($f in @("tkspeed.db", "tkspeed.db-wal", "tkspeed.db-shm")) {
    $src = Join-Path $appData $f
    if (Test-Path $src) { Copy-Item $src $staging -Force }
}

# 2) Logs.
$logs = Join-Path $appData "logs"
if (Test-Path $logs) {
    New-Item -ItemType Directory -Force (Join-Path $staging "logs") | Out-Null
    Copy-Item (Join-Path $logs "*") (Join-Path $staging "logs") -Force -Recurse
}

# 3) Resumo do sistema.
$sys = Join-Path $staging "system.txt"
"=== TkSpeed Diagnostic — $stamp ===" | Out-File $sys -Encoding utf8
try {
    $os  = Get-CimInstance Win32_OperatingSystem
    $cpu = Get-CimInstance Win32_Processor
    $gpu = Get-CimInstance Win32_VideoController
    $cs  = Get-CimInstance Win32_ComputerSystem
    "OS        : $($os.Caption) $($os.Version) (build $($os.BuildNumber))" | Out-File $sys -Append -Encoding utf8
    "Máquina   : $($cs.Manufacturer) $($cs.Model)"                         | Out-File $sys -Append -Encoding utf8
    "CPU       : $($cpu.Name)"                                             | Out-File $sys -Append -Encoding utf8
    "RAM (GB)  : $([math]::Round($cs.TotalPhysicalMemory/1GB,1))"          | Out-File $sys -Append -Encoding utf8
    foreach ($g in $gpu) { "GPU       : $($g.Name) (driver $($g.DriverVersion))" | Out-File $sys -Append -Encoding utf8 }
    "Plano ativo:" | Out-File $sys -Append -Encoding utf8
    (powercfg /getactivescheme) | Out-File $sys -Append -Encoding utf8
} catch {
    "Falha ao coletar info do sistema: $_" | Out-File $sys -Append -Encoding utf8
}

# 4) Compactar.
if (Test-Path $outZip) { Remove-Item $outZip -Force }
Compress-Archive -Path (Join-Path $staging "*") -DestinationPath $outZip -Force
Remove-Item $staging -Recurse -Force

Write-Host ""
Write-Host "Diagnóstico gerado em:" -ForegroundColor Green
Write-Host "  $outZip"
Write-Host ""
Write-Host "Envie este arquivo ao time TkSpeed junto com o FEEDBACK.md." -ForegroundColor Yellow
Write-Host "Pressione Enter para fechar."
Read-Host | Out-Null
