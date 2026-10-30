//! PDF 生成（krilla）の検証。技術選定書 DEC-205 の根拠。
//!
//! v1 は WebView2 の `PrintToPdf` に依存し、300KB の最悪ケース（283 ページ）で
//! 中央値 10.70 秒だった。うち自前処理は 26.2ms（0.3% 未満）で、**短縮の余地が
//! アプリ側に無い**ことが v1 の結論である（実装方針書 §8.4）。
//!
//! ここでは自前生成に置き換えた場合の所要時間を測る。あわせて v1 で
//! Windows 限定だった**しおり（アウトライン）を全 OS で出せること**、
//! および**日本語フォントの埋め込み**（§7.3 / DEC-208）を確認する。
//!
//! 実行: pdf-poc.exe [ページ数 ...]   既定は 283 / 1000 / 5000

use std::time::Instant;

use krilla::geom::Point;
use krilla::destination::XyzDestination;
use krilla::outline::{Outline, OutlineNode};
use krilla::page::PageSettings;
use krilla::text::{Font, TextDirection};
use krilla::Document;

/// A4 縦（ポイント単位）。v1 の出力仕様（基本設計書 §23.2）に合わせる。
const PAGE_WIDTH: f32 = 595.0;
const PAGE_HEIGHT: f32 = 842.0;
const MARGIN: f32 = 56.0;
const BODY_SIZE: f32 = 10.5;
const LINE_HEIGHT: f32 = 16.0;
/// 1 ページに入る本文行数。
const LINES_PER_PAGE: usize = 44;

/// 日本語フォントを読む。
///
/// 本番は同梱フォント（DEC-209: PlemolJP / IBM Plex Sans JP）を使うが、
/// 本 PoC ではまだ同梱していないため、システムフォントで代用する。
/// **埋め込みのコストと出力サイズの傾向を見るのが目的**であり、
/// 書体そのものの評価ではない。
fn load_font() -> Option<(Font, &'static str)> {
    const CANDIDATES: [&str; 3] = [
        "C:/Windows/Fonts/meiryo.ttc",
        "C:/Windows/Fonts/YuGothR.ttc",
        "C:/Windows/Fonts/yumin.ttf",
    ];
    for path in CANDIDATES {
        let Ok(data) = std::fs::read(path) else { continue };
        if let Some(font) = Font::new(data.into(), 0) {
            return Some((font, path));
        }
    }
    None
}

fn build(pages: usize, font: &Font) -> (Vec<u8>, usize) {
    let mut document = Document::new();
    let mut outline = Outline::new();
    let mut headings = 0usize;

    for page_index in 0..pages {
        let mut page = document.start_page_with(
            PageSettings::from_wh(PAGE_WIDTH, PAGE_HEIGHT).expect("用紙サイズが不正"),
        );
        let mut surface = page.surface();

        let mut y = MARGIN;
        for line_index in 0..LINES_PER_PAGE {
            // 各ページの先頭行を見出しに見立て、しおりの対象にする。
            let is_heading = line_index == 0;
            let text = if is_heading {
                format!("{} 章 見出しのサンプル", page_index + 1)
            } else {
                format!(
                    "本文の {} 行目。日本語と ASCII (sample text 123) を混在させている。",
                    line_index
                )
            };

            surface.draw_text(
                Point::from_xy(MARGIN, y),
                font.clone(),
                if is_heading { 14.0 } else { BODY_SIZE },
                &text,
                false,
                TextDirection::Auto,
            );

            if is_heading {
                // しおり。v1 は Windows でしか付けられなかった（基本設計書 §20.1）。
                outline.push_child(OutlineNode::new(
                    text.clone(),
                    XyzDestination::new(page_index, Point::from_xy(0.0, y)),
                ));
                headings += 1;
            }

            y += if is_heading { LINE_HEIGHT * 1.6 } else { LINE_HEIGHT };
        }

        // ページ番号（下端中央）。v1 の出力仕様に合わせる。
        surface.draw_text(
            Point::from_xy(PAGE_WIDTH / 2.0 - 12.0, PAGE_HEIGHT - MARGIN / 2.0),
            font.clone(),
            9.0,
            &format!("- {} -", page_index + 1),
            false,
            TextDirection::Auto,
        );

        surface.finish();
        page.finish();
    }

    document.set_outline(outline);
    let bytes = document.finish().expect("PDF の書き出しに失敗");
    (bytes, headings)
}

fn main() {
    let Some((font, font_path)) = load_font() else {
        eprintln!("日本語フォントを読めなかった。計測を中止する。");
        std::process::exit(1);
    };
    println!("使用フォント: {font_path}\n");

    let targets: Vec<usize> = {
        let args: Vec<usize> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
        if args.is_empty() { vec![283, 1000, 5000] } else { args }
    };

    println!(
        "{:>8} | {:>12} | {:>14} | {:>10} | {:>9}",
        "ページ数", "生成時間", "100 ページ換算", "出力サイズ", "しおり"
    );
    println!("{}", "-".repeat(68));

    for pages in targets {
        // 1 回目はフォントの解析ぶん遅いので捨てる。
        let _ = build(8, &font);

        let start = Instant::now();
        let (bytes, headings) = build(pages, &font);
        let elapsed = start.elapsed().as_secs_f64();

        println!(
            "{:>8} | {:>9.2} 秒 | {:>11.2} 秒 | {:>7.1} MB | {:>7} 件",
            pages,
            elapsed,
            elapsed / pages as f64 * 100.0,
            bytes.len() as f64 / 1_048_576.0,
            headings
        );

        if pages == 283 {
            std::fs::write("out/sample-283.pdf", &bytes).expect("保存に失敗");
            println!("         （out/sample-283.pdf に保存。v1 の 10.70 秒と同じページ数）");
        }
    }
}
