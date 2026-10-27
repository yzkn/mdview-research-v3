# mdview v3 モバイル対応の調査

mdview（Rust と iced で作った Markdown ビューア・エディタ）の v3 で、スマートフォン・タブレット（Android・iOS）に
対応できるかを確かめるための一式。**調査用で、配る物は作らない。**

結果と考え方は [doc/v3モバイル対応の実現可能性検証.md](doc/v3モバイル対応の実現可能性検証.md) にある。

## 中身

|場所|中身|
|---|---|
|`src/` ほか|mdview v2 のソース（公開版と同じもの）|
|`poc/v3-mobile/core/`|`src/` の UI に依存しない部分（解析・レイアウト・出力・図・数式）を `#[path]` で読むクレート|
|`poc/v3-mobile/iced-shim/`|名前を `iced` にして `iced_core` の型だけを差し出すクレート（窓・GPU の部品を持ち込まない）|
|`poc/v3-mobile/dioxus-app/`|閲覧と原文の編集の試作（Dioxus 0.7。端末の WebView で描く）|
|`poc/v3-mobile/iced-android/`|iced 0.14 を Android・iOS 向けに型検査するだけのクレート|
|`.github/workflows/v3-mobile-poc.yml`|Android（APK）と iOS（シミュレーター向けの .app）で組めるかを確かめる|

## GitHub Actions で確かめる

`poc/v3-mobile/` か作業の流れのファイルを変えて push すると流れる（Actions の画面から手でも起動できる）。
Android と iOS のジョブがそれぞれ次の 4 段を流し、**1 段が落ちても先へ進めて、どこまで通ったかを Summary に書く**。

1. `core` を組む
2. iced 0.14 を型検査する
3. Dioxus の CLI（`dx`）を入れる
4. Dioxus の試作を APK・.app に組む（組めたら成果物に 14 日残す）

## 手元で組む（PC）

同梱の字体はリポジトリに無いので、最初に取る（`include_bytes!` で読むため）。

```sh
python tools/fetch-fonts.py
cargo test  --manifest-path poc/v3-mobile/Cargo.toml -p mdview-core
cargo run   --manifest-path poc/v3-mobile/Cargo.toml -p mdview-mobile-poc
cargo run   --manifest-path poc/v3-mobile/Cargo.toml -p mdview-core --release --example render_cost
```

`mdview-core` の単体試験のうち 3 件は、見本の画像を作業フォルダからの相対パスで探すため、`poc/v3-mobile/` から流すと落ちる。

## ライセンス

[LICENSE](LICENSE) のとおり。依存の著作権表示は `assets/licenses/` にある。
