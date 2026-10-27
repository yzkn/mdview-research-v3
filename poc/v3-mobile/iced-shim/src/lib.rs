//! `iced` の型だけを差し出す（v3 のモバイル対応の検証）。
//!
//! 本体の UI に依存しない部分は、色・矩形・字体などの値の型だけを iced から借りている。
//! それらは `iced_core` にあるので、窓と GPU の部品を持ち込まずに組める。

pub use iced_core::*;

/// 本体は `iced::advanced::{image, renderer, text}` を名指しする
pub mod advanced {
    pub use iced_core::*;
}
