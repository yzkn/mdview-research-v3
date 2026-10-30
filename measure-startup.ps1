# 起動時間の実測（詳細設計書 DD-OPEN-01）。
#
# アプリ自身が初回描画時に STARTUP 行を出す。PoC（技術選定書 §6.2）と同じ
# 「初回の view() 呼び出しまで」を測るので、PoC の 1,275〜1,390ms と直接比較できる。
#
# **これは「中身が描けるまで」ではない**（2026-10-16）。プロセスの起動（約 140 ms）と、
# GPU での最初の描画が入らない。ダブルクリックから中身が描けるまでは、試験の口
# （--automation）で最初の画面写しが取れるまでを外から測る（基本設計書 §20.3）
#
# 使い方: powershell -ExecutionPolicy Bypass -File measure-startup.ps1 [-File <path>] [-Times 3]

param([string]$File = "perf\v2-10mb.md", [int]$Times = 3)

$exe = ".\target\release\mdview.exe"
if (-not (Test-Path $exe)) { Write-Host "リリースビルドがない"; exit 1 }

# **v2.1.3 から、STARTUP 行は計測のときだけ出す。** 立てないと 1 行も出ない
$env:MDVIEW_STARTUP_LOG = "1"

$values = @()
for ($i = 1; $i -le $Times; $i++) {
    $err = "startup-$i.txt"
    $p = Start-Process $exe -ArgumentList $File -PassThru -RedirectStandardError $err -RedirectStandardOutput "startup-$i.out.txt"
    Start-Sleep -Seconds 6
    $p.Refresh()
    if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force }
    $p.WaitForExit()

    $line = Get-Content $err -ErrorAction SilentlyContinue | Where-Object { $_ -like "STARTUP*" } | Select-Object -First 1
    if ($line) {
        $ms = [double](($line -split " ")[-1] -replace "ms", "")
        $values += $ms
        "{0} 回目: {1:N0} ms" -f $i, $ms
    } else {
        "{0} 回目: STARTUP 行が無い（起動に失敗した可能性）" -f $i
        Get-Content $err -ErrorAction SilentlyContinue | Select-Object -First 3
    }
    Remove-Item $err, "startup-$i.out.txt" -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 500
}

if ($values.Count -gt 0) {
    $sorted = $values | Sort-Object
    "中央値: {0:N0} ms" -f $sorted[[math]::Floor($sorted.Count / 2)]
}
