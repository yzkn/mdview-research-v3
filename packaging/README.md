# packaging — 配布物の生成

## v2 のもの（いま使う）

|置き場|役割|
|---|---|
|`windows/mdview.iss`|Inno Setup のスクリプト。**右クリックの「mdview で開く」**（v2.2.0。`SystemFileAssociations` に書くので、既定のアプリが何でも出る。選択肢で既定は付ける）。**利用者ごと**に入れ、`.md` / `.markdown` の関連付けを選択制で登録する|
|`../src/bin/mdview-console.rs`|端末用の入口。インストーラが **`mdview.com`** として本体の隣に入れる（v2.1.3）。`cargo build --release` で `mdview-console.exe` が出来る|
|`windows/build-installer.ps1`|版数を `Cargo.toml` から取り、`ISCC` を呼ぶ|
|`linux/build-deb.sh`|`.deb` を組む（`.desktop` / アイコン / MIME つき）|
|`linux/build-appimage.sh`|AppImage を組む（単一ファイル）|
|`linux/check-runtime-libs.sh`|実行時に読み込む（dlopen）ライブラリがそろっているかを見る。**`.deb` だけを入れたまっさらな環境で呼ぶ**（Release が使う。v2.1.2）。漏れていたら `build-deb.sh` の `Depends` に足す|
|`macos/build-dmg.sh`|`.app` と `.dmg` を組む。**macOS でしか動かない**|
|`RELEASE-NOTES-TEMPLATE.md`|Release の説明文のひな形。**署名していないので警告の外し方を必ず載せる**|

**署名はしない**（基本設計書 §27.4）。代わりに SHA-256 の照合値を添える。

アイコンは生成物。先に `cargo run --example make-icons` を実行する
（[`assets/README.md`](../assets/README.md)）。

---

## v1 のもの（**捨てた**。2026-10-05）

v1（Tauri + WiX + 自己署名）時代の 4 本を消した。

|消したもの|なぜ残せないか|
|---|---|
|`Invoke-Release.ps1`|**`package.json` から版数を読む。** v2 には無く、走らせると途中で壊れる|
|`build-windows.ps1`|Tauri のバンドラを呼ぶ|
|`sign.ps1` / `New-InternalSigningCert.ps1`|**署名しない方針**（基本設計書 §27.4）|

**残しておくほうが危ない。** 配布物は GitHub Actions が作るのに、
手元で走らせられる別経路が残っていると、どちらが正か分からなくなる。

## 手順

**実施手順は [`doc/リリース手順.md`](../doc/リリース手順.md) を参照。**
本ファイルはスクリプトの一覧と方針のみを扱う。

> 署名の手順書は **2026-10-05 に廃止した**（署名しない方針。基本設計書 §27.4）。

## 方針

|OS|配布形態|署名|
|---|---|---|
|Windows|MSI / NSIS セットアップ EXE|**自己署名のコード署名証明書 ＋ GPO で信頼配布**|
|macOS|dmg / app|署名・公証なし（Gatekeeper の警告は社内配布ポリシーで対応）|
|Linux|AppImage / deb / rpm|署名なし|

**署名は手元の端末で行い、CI では署名しない**（秘密鍵を CI に置かないため）。CI（`.github/workflows/release.yml`）が生成する配布物は未署名で、動作確認用である。

Tauri ではインストーラの中にアプリ本体の exe が入るため、成果物へ後から署名するのではなく**バンドラの `signCommand` 経由で署名する**。これにより exe とインストーラの両方が署名される。マシン固有の拇印を `tauri.conf.json` に残さないよう、`build-windows.ps1` が `--config` によるインライン上書きで渡している。

配布スクリプトの置き場を `dist/` ではなく `packaging/` としたのは、`dist/` が Vite の出力先であり、`vite build` が既定で出力先を空にするため（置いたスクリプトが削除される）。

*関連: `design/Markdown Viewer実装方針書.md` §9*
