# B. 配布パッケージのタスク

作成日: 2026-10-02
対象: v2.0.0 正式リリースの B（アイコン・インストーラ・関連付け）

---

## 0. 「Windows 以外もインストーラを用意できるか」への回答

**できる。3 OS とも実機を持たずに GitHub Actions 上で作れる。**

いずれも、すでに動いているリリースワークフロー（`.github/workflows/release.yml`）の
各 OS のジョブへ「パッケージを組む手順」を足すだけで済む。

|OS|作れるもの|組み立てに使うもの|runner で動くか|
|---|---|---|---|
|Windows|インストーラ（exe / msi）|Inno Setup または WiX|**動く**（choco で入る）|
|Linux|`.deb` / AppImage / `tar.gz`|`dpkg-deb` / linuxdeploy|**動く**（標準で入っている）|
|macOS|`.app` + `.dmg`|`iconutil` / `hdiutil`|**動く**（標準で入っている）|

### ただし、できないこと

|項目|状況|受け取る側への影響|
|---|---|---|
|**署名・公証**|**しない**（§27.4 の判断を継続）|Windows は SmartScreen、macOS は Gatekeeper の警告が出る。外し方を Release の説明文へ書く|
|**実機での確認**|runner の中でしか試せない|「入って・起動して・消せる」ところまでは CI で自動確認できる。**OS の見た目や関連付けの実挙動は実機が要る**|
|macOS の公証|Apple Developer Program（年額）が要る|未加入のため、当面は `xattr` で隔離属性を外す手順を案内する|

### 費用（基本設計書 §27.9 の倍率）

パッケージを組む処理は数十秒で終わるため、**いまの 1 リリース約 129 分から
大きくは増えない**（見積もり +5 分程度）。macOS は倍率 10 のため、
**パッケージ作成は release ワークフローだけに置き、push ごとの CI には入れない**。

---

## 1. 決まったこと（2026-10-02）

|#|決めたこと|
|---|---|
|D-1|**アイコンはこちらで用意する。** 原本は SVG 3 枚（本体・小さいとき・文書）|
|D-2|**Windows は Inno Setup（exe）**|
|D-3|**Linux は `.deb` + AppImage**（`.rpm` は後回し）|
|D-4|**利用者ごとに入れる**（`%LOCALAPPDATA%`。管理者権限が要らない）|

---

## 2. タスク

### B-1. アイコンを作る（**完了 2026-10-02**）

- 原本は **SVG 1 枚**。そこから各形式を生成するスクリプトを書く（`tools/make-icons.py`）
- 生成するもの

|用途|形式|サイズ|
|---|---|---|
|Windows の exe / インストーラ|`.ico`|16 / 32 / 48 / 64 / 128 / 256|
|macOS の `.app`|`.icns`|16〜1024（`iconutil`。**macOS 上でしか作れない**ので CI で作る）|
|Linux の `.desktop`|`.png`|16 / 32 / 48 / 64 / 128 / 256|
|文書の関連付け（別図案）|同上|同上|

- **生成物はリポジトリへ置かない**（同梱フォントと同じ扱い。`assets/README.md`）。
  CI とローカルでスクリプトから作る

### B-2. 実行ファイルへアイコンを埋める（**完了 2026-10-02**）

- `build.rs` + `embed-resource` で Windows の exe へ `.ico` を埋める
- 版数・会社名などの**バージョン情報リソース**も同時に入れる
- **Windows 以外では何もしない**ようにする（`cfg`）

### B-3. Windows インストーラ（**完了 2026-10-02。実機で確認済み**）

- D-2 の選択に従う。既定は **Inno Setup**（`packaging/windows/mdview.iss`）
- やること

|項目|内容|
|---|---|
|置き場|`%LOCALAPPDATA%\Programs\mdview`（利用者ごと・管理者不要）|
|スタートメニュー|`mdview`|
|関連付け|`.md` / `.markdown` を**選択制**で登録（既定は off）|
|アンインストール|「アプリと機能」に出す|
|上書き更新|同じ版数以下なら入れ直さない|

- **v1 の `packaging/`（Tauri + WiX + 署名）は捨てる**。C の棚卸しで扱う

### B-4. 関連付けから開く（**完了 2026-10-02。実機で確認済み**）

**既定の関連付けは書き換えない。** 実機で確かめたところ、`.md` には
すでに別の道具の関連付けと `UserChoice` が入っていた。Windows 8 以降は
`UserChoice` が優先されるため書いても効かず、消すときに**元から入って
いた値まで消してしまう**。`OpenWithProgids` へ足す形に改めた。

- 受入条件 §23.1 の未達項目
- Windows: インストーラが `HKCU\Software\Classes` へ登録
- アプリ側は**すでに第 1 引数を開く**（§19.1）ので、実装の追加は無い見込み
- **確認は実機**（CI では関連付けの実挙動まで見られない）

### B-5. Linux パッケージ（**書けた。CI 確認待ち**）

- `.deb`（`dpkg-deb` で組む）

|同梱するもの|置き場|
|---|---|
|実行ファイル|`/usr/bin/mdview`|
|`.desktop`|`/usr/share/applications/mdview.desktop`|
|アイコン|`/usr/share/icons/hicolor/<size>/apps/mdview.png`|
|MIME 関連付け|`text/markdown` を `mdview.desktop` へ|

- AppImage（D-3 で ② 以上を選んだ場合）。`linuxdeploy` で組む
- **`.rpm` は後回し**にできる（必要になった時点で `cargo-generate-rpm`）

### B-6. macOS パッケージ（**書けた。CI 確認待ち**）

- `.app` バンドルをスクリプトで組む（`packaging/macos/bundle.sh`）

|項目|内容|
|---|---|
|`Info.plist`|`CFBundleIdentifier` / 版数 / **`CFBundleDocumentTypes` で `.md` を関連付け**|
|アイコン|`iconutil` で `.icns` を作って同梱|
|`.dmg`|`hdiutil` で作る。背景画像は付けない（単純に保つ）|

- **署名も公証もしない。** 初回起動の手順（`右クリック > 開く`、または
  `xattr -dr com.apple.quarantine`）を Release の説明文へ

### B-7. リリースワークフローへ組み込む（**完了 2026-10-02**）

- 各 OS のジョブへ「パッケージを組む」手順を足し、成果物に加える
- **push ごとの CI には入れない**（macOS の倍率が効く）
- 成果物の名前を揃える

```
mdview-<版数>-windows-x86_64-setup.exe
mdview-<版数>-windows-x86_64.exe          ← 単一実行ファイル（従来どおり）
mdview-<版数>-linux-x86_64.deb
mdview-<版数>-linux-x86_64.AppImage
mdview-<版数>-linux-x86_64                ← 単一実行ファイル
mdview-<版数>-macos-aarch64.dmg
mdview-<版数>-macos-aarch64               ← 単一実行ファイル
```

- **単一実行ファイルは残す**（§27.3）。インストーラを使わない配り方を塞がない

### B-8. 入れて・起動して・消す煙試験（**完了 2026-10-02**。`--version` を含む）

**CI で自動確認できる範囲**を確かめる。ここを作らないと、壊れた
インストーラをそのまま配ることになる。

|OS|やること|
|---|---|
|Windows|`/VERYSILENT` で入れる → exe が在る → 版数が取れる → `/VERYSILENT /UNINSTALL` で消える|
|Linux|`apt-get install ./…`（依存を解決させる。v2.1.1 で `dpkg -i` から変えた）→ 依存に `libxkbcommon-x11-0` が在る → `/usr/bin/mdview` が在る → `.desktop` が妥当（`desktop-file-validate`）→ `dpkg -r`|
|macOS|`.dmg` をマウント → `.app` の構造と `Info.plist` を確かめる → アンマウント|

- **窓を出す確認はしない**（runner に画面が無い）。起動確認は `--version` 相当で行う
  → **`--version` / `--help` の口が要る**（いま無い。B-8 で足す）

### B-9. 受け取る側の手順（**ひな形を用意。完了**）

- Release の説明文のひな形（警告の外し方・入れ方・消し方）
- `doc/` 側は C の棚卸しとあわせて整理する

### B-10. 公開用複製へ反映（E と重複。0.5 日）

- `migration-v2/` にも同じ仕組みを入れる。社名・個人名を入れないよう注意
  （`CFBundleIdentifier` や MSI の `Manufacturer` に社名を入れない）

---

## 3. 見積もり

|項目|工数|
|---|---|
|B-1 アイコン|0.5 日（作図は別）|
|B-2 exe への埋め込み|0.5 日|
|B-3 Windows インストーラ|1 日|
|B-4 関連付け|B-3 に含む|
|B-5 Linux パッケージ|1 日|
|B-6 macOS パッケージ|1 日|
|B-7 ワークフロー組み込み|0.5 日|
|B-8 煙試験（`--version` を含む）|0.5 日|
|B-9 受け取る側の手順|0.5 日|
|B-10 公開用複製へ反映|0.5 日|
|**合計**|**約 6 日**|

Windows だけなら **約 3 日**（B-1 / B-2 / B-3 / B-4 / B-7 の一部 / B-8 の一部 / B-9）。

---

## 4. 進める順

1. **D-1〜D-4 を決める**
2. B-1 → B-2 → B-8 の `--version`（**先に作る。** 煙試験の土台になる）
3. B-3 → B-4（Windows を通しで完成させ、実機で確認する）
4. B-7（Windows ぶんをワークフローへ）→ B-8（Windows の煙試験）
5. B-5 → B-6（Linux → macOS。CI の煙試験まで）
6. B-9 → B-10

**Windows を先に通す。** 実機で確かめられる唯一の OS であり、
ここで固めた形を他の 2 つへ写すほうが手戻りが少ない。
