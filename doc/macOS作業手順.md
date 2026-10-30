# macOS での作業手順

対象: v2.0.0 以降
最終更新: 2026-10-05

---

## 0. いまの状況

**macOS の実機は無い。** 配布物は GitHub Actions の macOS runner が組む
（基本設計書 §27）。

|やること|どこで|
|---|---|
|ビルド・`.app` と `.dmg` の組み立て|**GitHub Actions**|
|中身の形が壊れていないか|**GitHub Actions**（煙試験）|
|**画面の見た目・IME・字形**|**誰も見ていない**|

そのため、macOS 版は**「見た人がいない版」として配る**
（[リリース手順](リリース手順.md) §5）。伏せない。

この文書は 2 つの場面で使う。

- **§1**: 受け取った人に試してもらうとき
- **§2**: 実機が手に入ったとき

---

## 1. 受け取った人に試してもらう

### 1-1. 入れる

1. Release から `mdview-<版数>-macos-aarch64.dmg` を落とす
2. 開いて `mdview.app` を `Applications` へ引きずる

### 1-2. 初回起動（**警告が出る**）

**署名も公証もしていない**（§27.4）ため、Gatekeeper に止められる。

```
右クリック > 開く > （ダイアログで）開く
```

それでも開けない場合:

```bash
xattr -dr com.apple.quarantine /Applications/mdview.app
```

### 1-3. 見てほしいところ

[受入確認チェックリスト](受入確認チェックリスト.md) の**実機**の項目のうち、
特に次を見てもらう。CI では確かめられないものである（§27.5）。

|#|見るところ|なぜ|
|---|---|---|
|D-1|**日本語の字形**（`入` `直` `骨` `今` `令`）|中国語字形が出ると読みにくい。v1 で実際に踏んだ|
|C-1|**IME で日本語を入力できる**|入力そのものが要る|
|C-2|変換候補の窓がキャレットの位置に出る|位置の通知を怠ると画面の隅に出る|
|D-7|スクロールが引っかからない|runner に GPU が無い|
|A-8|起動が速いか（体感）|—|
|E-6|PDF / HTML の見た目|出力はできるが、判断は人が見る|

### 1-4. 落ちたときに送ってほしいもの

```bash
# アプリが出したもの（パニックの位置と理由）。
# **本体を直に起こす。** v2.1.0 から .app の本体は起動用（Finder からファイルを受け取る）で、
# 本体は中の Helpers にある
/Applications/mdview.app/Contents/Helpers/mdview.app/Contents/MacOS/mdview 2>&1 | tail -40

# システム側にも残っていることがある
log show --last 15m --predicate 'process == "mdview"' --info --debug | grep -i panic
```

---

## 2. 実機が手に入ったら

### 2-1. 用意するもの

|もの|入れ方|
|---|---|
|Xcode Command Line Tools|`xcode-select --install`|
|Rust|`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh`|
|Python 3|同梱フォントの取得に使う|

**版数は `rust-toolchain.toml` に従う**（§27.6）。手元と CI をそろえるため、
`rustup` が自動で合わせる。

### 2-2. ビルド

```bash
git clone <このリポジトリ>
cd <リポジトリ>

# 同梱フォントはリポジトリに無い（21.56MB あるため）
python3 tools/fetch-fonts.py

# アイコンの材料（.iconset）を作る
cargo run --release --example make-icons

cargo build --release
./target/release/mdview --version
```

### 2-3. `.app` と `.dmg` を作る

```bash
packaging/macos/build-dmg.sh
```

`dist/mdview-<版数>-macos-aarch64.dmg` が出来る。
**`.icns` は macOS でしか作れない**ため、この手順だけは実機か runner が要る。

### 2-4. 確認

[受入確認チェックリスト](受入確認チェックリスト.md) の**実機**の項目を実施する。

---

## 3. 公証について

**していない。** Apple Developer Program（年額）が要るためである。

加入した場合に要るのは次の 3 つ。いずれも**この手順書の範囲外**なので、
やると決めた時点で別に設計する。

- Developer ID Application 証明書での署名
- `notarytool` での公証と `stapler` での添付
- CI へ秘密鍵を預ける手立て（GitHub の Secrets）

---

## 変更管理

|日付|内容|
|---|---|
|2026-10-05|**書き直した。** v1（Tauri + npm + 手元ビルド）前提から、CI がビルドする前提へ。実機が無いことを明記し、「受け取った人に試してもらう」節を主にした|
