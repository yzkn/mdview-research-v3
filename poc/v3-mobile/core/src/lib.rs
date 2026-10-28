//! mdview の UI に依存しない部分（v3 のモバイル対応の検証）。
//!
//! 本体のファイルを `#[path]` でそのまま読む。本体の `crate::…` の道筋を保つため、本体と同じ名前で
//! クレートの根に置く。**UI 層（app・render）の中で、UI に依存しない部分から呼ばれているものは
//! 下の `render`・`app` に最小限だけ置く**。それが v3 で切り分ける継ぎ目になる。
#![allow(dead_code, unused_imports)]

#[path = "../../../../src/diff/mod.rs"]
pub mod diff;
#[path = "../../../../src/document/mod.rs"]
pub mod document;
#[path = "../../../../src/edit/mod.rs"]
pub mod edit;
#[path = "../../../../src/embed/mod.rs"]
pub mod embed;
#[path = "../../../../src/export/mod.rs"]
pub mod export;
#[path = "../../../../src/io/mod.rs"]
pub mod io;
#[path = "../../../../src/layout/mod.rs"]
pub mod layout;
#[path = "../../../../src/marp/mod.rs"]
pub mod marp;
#[path = "../../../../src/paginate/mod.rs"]
pub mod paginate;
#[path = "../../../../src/parse/mod.rs"]
pub mod parse;
#[path = "../../../../src/search/mod.rs"]
pub mod search;
#[path = "../../../../src/worker.rs"]
pub mod worker;

/// 継ぎ目: UI 層の描画（render）のうち、出力・図・数式が使うもの。
/// **ファイルにしておく**（`mod render { … }` と書くと、`#[path]` が実在しない `core/src/render/` から `..` を辿り、
/// Linux・macOS で読めない。Windows は文字の上で縮めるので通ってしまう。2026-10-28 に CI で踏んだ）
pub mod render;

/// 継ぎ目: 設定がコマンドの履歴の上限を UI 層から借りている
pub mod app {
    pub const COMMAND_RECENT_MAX: usize = 10;
}
