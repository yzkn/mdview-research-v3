# mdview v3 の調査（モバイル対応・PC 版の UI）

mdview（Rust と iced で作った Markdown ビューア・エディタ）の v3 で、スマートフォン・タブレット（Android・iOS）に
対応できるかを確かめるための一式。**調査用で、配る物は作らない。**

あわせて、PC 版の UI（OS に寄せた見た目・メニューの幅と階層・macOS の OS のメニュー）を改善できるかも確かめる。

結果と考え方は次にある。

- [doc/v3モバイル対応の実現可能性検証.md](doc/v3モバイル対応の実現可能性検証.md)
- [doc/v3のPC版UI改善の実現可能性検証.md](doc/v3のPC版UI改善の実現可能性検証.md)
- [doc/v3モバイルのUI検討.md](doc/v3モバイルのUI検討.md)・[画面設計案（PPTX）](doc/v3モバイル画面設計案.pptx)
- [doc/v3追加機能の提案（類似プロジェクトの仕様から）.md](doc/v3追加機能の提案（類似プロジェクトの仕様から）.md)
- [doc/v3.0.0の要件定義書.md](doc/v3.0.0の要件定義書.md)（上の調査をまとめた v3 の要件。CI で確かめることは §6.2）

## 中身

|場所|中身|
|---|---|
|`src/` ほか|mdview v2 のソース（公開版と同じもの）|
|`poc/v3-mobile/core/`|`src/` の UI に依存しない部分（解析・レイアウト・出力・図・数式）を `#[path]` で読むクレート|
|`poc/v3-mobile/iced-shim/`|名前を `iced` にして `iced_core` の型だけを差し出すクレート（窓・GPU の部品を持ち込まない）|
|`poc/v3-mobile/dioxus-app/`|閲覧と原文の編集の試作（Dioxus 0.7。端末の WebView で描く）|
|`poc/v3-mobile/mock/`|モバイルの画面と操作の試作（HTML。**端末のブラウザーでそのまま開ける**。`gallery.html` で端末の大きさに並べて見比べる）|
|`poc/v3-mobile/iced-android/`|iced 0.14 を Android・iOS 向けに型検査するだけのクレート|
|`.github/workflows/v3-mobile-poc.yml`|Android（APK）と iOS（シミュレーター向けの .app）で組めるかを確かめる|
|`poc/v3-desktop-ui/`|PC 版の UI の試作（OS に寄せた見た目・中身に合わせたメニューの幅・横へ開く下の階層・macOS の OS のメニュー）|
|`.github/workflows/v3-desktop-ui-poc.yml`|macOS で PC 版の UI の試作を組んで起動し、窓と画面全体の写真を残す|
|`poc/v3-pdf-emoji/`|OS の絵文字の字体で PDF に絵文字を描く試作（本体と同じ PDF の部品 krilla）|
|`.github/workflows/v3-pdf-emoji-poc.yml`|Windows・macOS・Linux で PDF に絵文字を描き、行ごとに色が出たかを要約に書く|

## GitHub Actions で確かめる

`poc/v3-mobile/` か作業の流れのファイルを変えて push すると流れる（Actions の画面から手でも起動できる）。
Android と iOS のジョブがそれぞれ次の 4 段を流し、**1 段が落ちても先へ進めて、どこまで通ったかを Summary に書く**。

1. `core` を組む
2. iced 0.14 を型検査する
3. Dioxus の CLI（`dx`）を入れる
4. Dioxus の試作を APK・.app に組む（組めたら成果物に 14 日残す）

`poc/v3-desktop-ui/` か `v3-desktop-ui-poc.yml` を変えて push すると、macOS で PC 版の UI の試作が流れる（組む → 起動して OS のメニューを組む・写真を撮る。写真は成果物に 14 日残す）。

`poc/v3-pdf-emoji/` か `v3-pdf-emoji-poc.yml` を変えて push すると、3 つの OS で PDF の絵文字の試作が流れる（PDF と絵を成果物に 14 日残す）。

## 手元で組む（PC）

同梱の字体はリポジトリに無いので、最初に取る（`include_bytes!` で読むため）。

```sh
python tools/fetch-fonts.py
cargo test  --manifest-path poc/v3-mobile/Cargo.toml -p mdview-core
cargo run   --manifest-path poc/v3-mobile/Cargo.toml -p mdview-mobile-poc
cargo run   --manifest-path poc/v3-mobile/Cargo.toml -p mdview-core --release --example render_cost
cargo run   --manifest-path poc/v3-desktop-ui/Cargo.toml
cargo run   --manifest-path poc/v3-pdf-emoji/Cargo.toml -- emoji.pdf
```

`mdview-core` の単体試験のうち 3 件は、見本の画像を作業フォルダからの相対パスで探すため、`poc/v3-mobile/` から流すと落ちる。

## ライセンス

[LICENSE](LICENSE) のとおり。依存の著作権表示は `assets/licenses/` にある。
