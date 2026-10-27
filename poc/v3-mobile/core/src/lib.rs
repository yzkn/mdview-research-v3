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

/// 継ぎ目: UI 層の描画（render）のうち、出力・図・数式が使うもの
pub mod render {
    #[path = "../../../../../src/render/backdrop.rs"]
    pub mod backdrop;
    #[path = "../../../../../src/render/fonts.rs"]
    pub mod fonts;
    #[path = "../../../../../src/render/slides.rs"]
    pub mod slides;

    /// 継ぎ目: 本体は render/preview.rs の配色を使う（PPTX の書き出し）。検証では字の色のまま
    pub fn token_color(_role: crate::parse::highlight::TokenRole, text: iced::Color) -> iced::Color {
        text
    }
}

/// 継ぎ目: 設定がコマンドの履歴の上限を UI 層から借りている
pub mod app {
    pub const COMMAND_RECENT_MAX: usize = 10;
}
