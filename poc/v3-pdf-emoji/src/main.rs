//! OS の絵文字の字体で、PDF に絵文字を描けるかを見る（v3 の M-7）。
//!
//! `mdview-pdf-emoji-poc <出力.pdf> [字体のファイル]`。字体を指さなければ OS の決まった場所から探す。
//! 1 ページに、絵文字の種類ごとに 1 行ずつ描く（行の並びは下の `LINES`）。字体の形式（COLR・SVG・sbix・CBDT）と
//! PDF の大きさを標準出力へ出す。**描けたかの判定は、PDF を絵にして色の画素を数える側（CI の Python）で行う**
use std::path::PathBuf;
use std::sync::Arc;

use krilla::geom::Point;
use krilla::page::PageSettings;
use krilla::text::{Font, TextDirection};
use krilla::Document;

/// 描く行（v2.9.0 の確認で気になったものを並べる）
const LINES: [(&str, &str); 6] = [
    ("1 basic", "😀 🎉 🙂 👍 📝 🐛 🍣"),
    ("2 heart (FE0F)", "❤️"),
    ("3 warning (FE0F)", "⚠️ ✅"),
    ("4 ZWJ family", "👨‍👩‍👧"),
    ("5 flag", "🇯🇵"),
    ("6 skin tone", "👍🏽"),
];

/// OS の絵文字の字体の場所（あれば先頭のもの）
fn system_emoji_font() -> Option<PathBuf> {
    let candidates: &[&str] = if cfg!(windows) {
        &[r"C:\Windows\Fonts\seguiemj.ttf"]
    } else if cfg!(target_os = "macos") {
        &["/System/Library/Fonts/Apple Color Emoji.ttc"]
    } else {
        &[
            "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
            "/usr/share/fonts/noto/NotoColorEmoji.ttf",
            "/usr/share/fonts/google-noto-emoji/NotoColorEmoji.ttf",
        ]
    };
    candidates.iter().map(PathBuf::from).find(|p| p.exists())
}

/// 字体の色の形式（持っている表）
fn formats(data: &[u8]) -> Vec<&'static str> {
    let Ok(face) = ttf_parser::Face::parse(data, 0) else {
        return vec!["読めない"];
    };
    let raw = face.raw_face();
    let mut out = Vec::new();
    for (tag, name) in [(b"COLR", "COLR"), (b"SVG ", "SVG"), (b"sbix", "sbix"), (b"CBDT", "CBDT")] {
        if raw.table(ttf_parser::Tag::from_bytes(tag)).is_some() {
            out.push(name);
        }
    }
    if out.is_empty() {
        out.push("色の表が無い");
    }
    out
}

fn main() {
    let mut args = std::env::args().skip(1);
    let out = PathBuf::from(args.next().expect("出力の PDF を指す"));
    let path = args.next().map(PathBuf::from).or_else(system_emoji_font).expect("絵文字の字体が見つからない");
    let data = std::fs::read(&path).expect("字体を読めない");
    println!("字体: {}（{} バイト）", path.display(), data.len());
    println!("形式: {}", formats(&data).join("・"));

    let font = Font::new(Arc::new(data).into(), 0).expect("krilla が字体を読めない");
    let mut document = Document::new();
    let mut page = document.start_page_with(PageSettings::from_wh(420.0, 60.0 + 56.0 * LINES.len() as f32).unwrap());
    let mut surface = page.surface();
    for (i, (_, text)) in LINES.iter().enumerate() {
        // **字体 1 つだけで描く**（krilla の簡単な口は代わりの字体を探さない）。本番では本文の字体と行を分けて並べる
        surface.draw_text(
            Point::from_xy(30.0, 60.0 + 56.0 * i as f32),
            font.clone(),
            36.0,
            text,
            false,
            TextDirection::Auto,
        );
    }
    surface.finish();
    page.finish();
    let pdf = document.finish().expect("PDF を作れない");
    std::fs::write(&out, &pdf).expect("PDF を書けない");
    println!("PDF: {}（{} バイト）", out.display(), pdf.len());
    for (label, text) in LINES {
        println!("行 {label}: {text}");
    }
}
