@echo off
REM rustc 1.97.1 の自動検出が Visual Studio 18 の MSVC を拾わないため、
REM 開発者環境（vcvars64）を読み込んでから cargo を起動する。
REM 検出が直れば不要になる。設定ファイル（.cargo/config.toml）は変更しない方針。
call "C:\Program Files\Microsoft Visual Studio\18\Enterprise\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
if errorlevel 1 (
  echo vcvars64.bat の読み込みに失敗した
  exit /b 1
)
"%USERPROFILE%\.cargo\bin\cargo.exe" %*
