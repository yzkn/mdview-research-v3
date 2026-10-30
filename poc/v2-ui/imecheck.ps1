# IME 確認モードで起動できているかの自動チェック。
#
# 計測モードで起動してしまうと 15 秒で自動終了する。20 秒後も生きていれば
# IME モードに入れている。中身（変換候補の位置など）は人が見るしかない。

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
foreach ($name in @("iced", "egui")) {
    $exe = Join-Path $root "target\release\$name-poc.exe"
    $err = Join-Path $root "out\imecheck-$name.err.txt"
    $out = Join-Path $root "out\imecheck-$name.out.txt"

    $proc = Start-Process -FilePath $exe -ArgumentList "--ime" -PassThru `
        -RedirectStandardError $err -RedirectStandardOutput $out
    Start-Sleep -Seconds 20
    $proc.Refresh()
    $alive = -not $proc.HasExited
    $mode = (Get-Content $err -ErrorAction SilentlyContinue | Where-Object { $_ -like "POCMODE*" }) -join ""
    "{0,-5} 20秒後も起動中={1,-5} {2}" -f $name, $alive, $mode
    if ($alive) { $proc.Kill(); $proc.WaitForExit() }
}
