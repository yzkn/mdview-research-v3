# v2.0.0 UI ツールキット選定 PoC

`design/Markdown Viewer v2.0.0 技術選定書.md` §6 の実測に使ったコード。
選定が確定したら丸ごと捨ててよい（本体のビルドには一切関与しない）。

## 構成

|クレート|役割|
|---|---|
|`core`|文書生成・ロープ・計測ハーネス。**iced / egui で共有**し、差がツールキットだけになるようにする|
|`egui-poc`|egui (eframe 0.36) 版|
|`iced-poc`|iced 0.14 版|
|`parse-poc`|comrak / pulldown-cmark の所要時間計測|
|`pdf-poc`|**PDF 生成の計測**（krilla）。ページ数・しおり・フォント埋め込み|
|`mermaid-poc`|**Mermaid 描画の検証**（merman + resvg）。HTML ラベルの欠落確認を含む|
|`math-poc`|**数式描画の検証**（latex-rust + resvg）|
|`font-size.py`|**同梱フォントの実サイズ測定**。配布 zip（211MB / 302MB）を落とさず、末尾 1MB の中央ディレクトリだけを HTTP Range で取得して読む|
|`hl-poc`|**コードブロック着色の計測**（syntect）。起動時コストと 1 ブロックあたりの時間|
|`incparse-poc`|**増分解析の検証**。tree-sitter-md の全解析・増分・最悪ケース（未閉鎖フェンス挿入）と、採用した代替案（行走査 + 可視範囲のみ comrak）の比較|

## 実行

```powershell
cargo build --release

# UI 計測（両ツールキットを 10MB で 15 秒ずつ。CPU とメモリはプロセス外から測る）
powershell -ExecutionPolicy Bypass -File run-bench.ps1 -SizeMb 10

# パーサ計測（既定 0.3 / 1 / 10 MB）
.\target\release\parse-poc.exe

# 同梱フォントの実サイズ（ダウンロードは合計 2MB で済む）
python font-size.py

# IME 確認（自動計測では判定できない。人が日本語を入力して確かめる）
.\target\release\iced-poc.exe --ime
.\target\release\egui-poc.exe --ime
```

**IME 確認は `--ime` で起動する。** 環境変数 `POC_MODE=ime` も受けるが、cmd.exe の
`set POC_MODE=ime && ...` は**値に末尾の空白を含める**ため（`"ime "`）判定から外れ、
計測モード（文字が流れ、15 秒で終了）で起動してしまう。

手順と判定基準は技術選定書 §6.5 を参照。起動モードの自動確認は
`powershell -ExecutionPolicy Bypass -File imecheck.ps1`（20 秒後も生きていれば IME モード）。

## ビルド環境の注意（2026-09-14 時点）

**`cargo build` を直接叩くとリンクに失敗する。**

**rustc 1.97.1 の自動検出が Visual Studio 18 の MSVC を拾わない。**
その結果 PATH 上の `C:\Program Files\Git\usr\bin\link.exe`（GNU coreutils の
`link`）を掴み、`link: extra operand ...` で失敗する。
回避のため `build-msvc.bat` を置いた。`vcvars64.bat` を読んでから cargo を
起動するだけのもので、`~/.cargo/config.toml` 等の設定ファイルは変更していない。

```powershell
.\build-msvc.bat build --release
```

rustc 側が VS 18 を検出できるようになれば、この節と `build-msvc.bat` は不要になる。

**解消済み（2026-09-15）**: Windows SDK の未導入により `kernel32.lib` が存在せず、
MSVC のリンカに到達しても `LNK1181` で止まる問題があった。SDK 10.0.22621.0 の
追加インストールで解消した。

## 注意

- **計測は CPU が空いた状態で行う。** 他のビルドと並行すると値が大きく崩れる
  （v1 の実装方針書 §8.4 でも同じ注意が記録されている）。
- `frames` は実描画回数（`view()` の呼び出し回数）で数えている。
  タイマー刻み数で数えると iced 側が vsync 上限を超えた値になり、比較にならない。
