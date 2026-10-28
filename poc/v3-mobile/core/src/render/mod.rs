//! 継ぎ目: UI 層の描画（render）のうち、出力・図・数式が使うもの（v3 のモバイル対応の検証）

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
