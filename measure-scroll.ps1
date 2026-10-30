# スクロール性能の実測（詳細設計書 PERF-02）。
#
# PoC（技術選定書 §6.2）と同じ条件で測り、直接比較できるようにする。
# 可視 44 行・1 フレーム 7 行・15 秒。CPU とメモリはプロセスの外から測る。
#
# 使い方: powershell -ExecutionPolicy Bypass -File measure-scroll.ps1 [-File <path>]

param([string]$File = "perf\v2-10mb.md", [string[]]$Extra = @())

$exe = ".\target\release\mdview.exe"
if (-not (Test-Path $exe)) { Write-Host "リリースビルドがない"; exit 1 }
if (-not (Test-Path $File)) { Write-Host "計測用の文書がない: $File"; exit 1 }

$out = "scroll-bench.out.txt"
$err = "scroll-bench.err.txt"

$startedAt = Get-Date
$proc = Start-Process $exe -ArgumentList (@($File, "--bench-scroll") + $Extra) -PassThru `
    -RedirectStandardOutput $out -RedirectStandardError $err

$peakWorkingSetMb = 0
while (-not $proc.HasExited) {
    try {
        $proc.Refresh()
        $ws = $proc.WorkingSet64 / 1MB
        if ($ws -gt $peakWorkingSetMb) { $peakWorkingSetMb = $ws }
    } catch { }
    Start-Sleep -Milliseconds 200
    if (((Get-Date) - $startedAt).TotalSeconds -gt 60) { $proc.Kill(); break }
}
$proc.WaitForExit()

$cpuMs = 0
try { $cpuMs = $proc.TotalProcessorTime.TotalMilliseconds } catch { }

$line = Get-Content $out -ErrorAction SilentlyContinue | Where-Object { $_ -like "BENCHRESULT*" } | Select-Object -First 1
if (-not $line) {
    Write-Host "BENCHRESULT が出ていない"
    Get-Content $err -ErrorAction SilentlyContinue | Select-Object -First 5
    exit 1
}

$json = ($line -replace "^BENCHRESULT ", "") | ConvertFrom-Json
$startup = Get-Content $err -ErrorAction SilentlyContinue | Where-Object { $_ -like "STARTUP*" } | Select-Object -First 1

[PSCustomObject]@{
    lines            = $json.lines
    ticks            = $json.ticks
    frames           = $json.frames
    fps              = [math]::Round($json.fps, 1)
    cpu_ms_total     = [math]::Round($cpuMs, 0)
    cpu_ms_per_frame = if ($json.frames -gt 0) { [math]::Round($cpuMs / $json.frames, 2) } else { 0 }
    peak_ws_mb       = [math]::Round($peakWorkingSetMb, 0)
    # --bench-edit のときだけ出る（OPEN-201: 打鍵 → プレビュー反映）
    edits            = $json.edits
    latency_median_ms = $json.latency_median_ms
    latency_worst_ms  = $json.latency_worst_ms
} | Format-List

if ($startup) { $startup }
Remove-Item $out, $err -ErrorAction SilentlyContinue
