# v2.0.0 UI ツールキット選定 PoC の計測ランナー。
#
# アプリ内で測れない CPU 時間とメモリを外から測る。こうすると iced / egui の
# どちらにも同じものさしが当たり、フレームレートの上限（vsync）に左右されない。
#
# 使い方: powershell -ExecutionPolicy Bypass -File run-bench.ps1 [-SizeMb 10]

param(
    [double]$SizeMb = 10
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$outDir = Join-Path $root "out"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

function Measure-Poc {
    param([string]$Name, [string]$ExePath)

    if (-not (Test-Path $ExePath)) {
        Write-Host "SKIP $Name (未ビルド: $ExePath)"
        return
    }

    $stdout = Join-Path $outDir "$Name.stdout.txt"
    $stderr = Join-Path $outDir "$Name.stderr.txt"

    $startedAt = Get-Date
    $proc = Start-Process -FilePath $ExePath -ArgumentList $SizeMb `
        -RedirectStandardOutput $stdout -RedirectStandardError $stderr -PassThru

    # 起動直後はメモリが立ち上がりきっていないため、終了までポーリングして最大値を取る。
    $peakWorkingSetMb = 0
    while (-not $proc.HasExited) {
        try {
            $proc.Refresh()
            $ws = $proc.WorkingSet64 / 1MB
            if ($ws -gt $peakWorkingSetMb) { $peakWorkingSetMb = $ws }
        } catch { }
        Start-Sleep -Milliseconds 200
        if (((Get-Date) - $startedAt).TotalSeconds -gt 90) {
            Write-Host "TIMEOUT $Name"
            $proc.Kill()
            break
        }
    }
    $proc.WaitForExit()

    # TotalProcessorTime は全スレッドの合計。フレーム数で割れば 1 フレームあたりの CPU 時間になる。
    $cpuMs = 0
    try { $cpuMs = $proc.TotalProcessorTime.TotalMilliseconds } catch { }
    $wallS = ((Get-Date) - $startedAt).TotalSeconds

    $line = (Get-Content $stdout -ErrorAction SilentlyContinue | Where-Object { $_ -like "POCRESULT*" } | Select-Object -First 1)
    if (-not $line) {
        Write-Host "NORESULT $Name (stdout/stderr を確認)"
        return
    }

    $json = ($line -replace "^POCRESULT ", "") | ConvertFrom-Json

    # 実描画回数。iced は update の呼び出し回数と一致しないため別に拾う。
    $viewsLine = (Get-Content $stdout -ErrorAction SilentlyContinue | Where-Object { $_ -like "POCVIEWS*" } | Select-Object -First 1)
    $views = if ($viewsLine) { [int]($viewsLine -replace "^POCVIEWS ", "") } else { 0 }
    $phases = (Get-Content $stderr -ErrorAction SilentlyContinue | Where-Object { $_ -like "POCPHASE*" }) -join " / "
    $exeMb = (Get-Item $ExePath).Length / 1MB
    # 実描画回数で割る。ここを update 回数で割ると iced が不当に速く見える。
    $cpuPerFrame = if ($views -gt 0) { $cpuMs / $views } else { 0 }
    $renderFps = if ($json.elapsed_s -gt 0) { $views / $json.elapsed_s } else { 0 }

    [PSCustomObject]@{
        toolkit          = $json.toolkit
        input_mb         = [math]::Round($json.bytes / 1MB, 1)
        lines            = $json.lines
        rope_load_ms     = [math]::Round($json.rope_load_ms, 1)
        insert_ms        = [math]::Round($json.insert_ms, 4)
        first_frame_ms   = [math]::Round($json.first_frame_ms, 0)
        ticks            = $json.frames
        views            = $views
        render_fps       = [math]::Round($renderFps, 1)
        cpu_ms_total     = [math]::Round($cpuMs, 0)
        cpu_ms_per_frame = [math]::Round($cpuPerFrame, 2)
        peak_ws_mb       = [math]::Round($peakWorkingSetMb, 0)
        exe_mb           = [math]::Round($exeMb, 1)
        wall_s           = [math]::Round($wallS, 1)
        phases           = $phases
    }
}

$results = @()
$results += Measure-Poc -Name "egui" -ExePath (Join-Path $root "target\release\egui-poc.exe")
$results += Measure-Poc -Name "iced" -ExePath (Join-Path $root "target\release\iced-poc.exe")

$results | Format-Table -AutoSize
$results | ConvertTo-Json -Depth 3 | Set-Content -Path (Join-Path $outDir "bench.json") -Encoding UTF8
Write-Host "saved: $(Join-Path $outDir 'bench.json')"
