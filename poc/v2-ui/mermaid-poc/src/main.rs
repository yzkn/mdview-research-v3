//! Mermaid 描画（merman）の検証。技術選定書 DEC-207 の根拠。
//!
//! v1 は mermaid 11.17.2 を WebView 上で動かしていた。v2 では WebView を持たないため、
//! 純 Rust 実装に置き換える必要がある。merman は同じ 11.17.2 に追従しているが
//! `0.8.0-alpha` であり、「Chromium とのピクセル一致は約束しない」と明記されている。
//!
//! 確認すること:
//!   1. v1 のサンプル文書に含まれる図が**そのまま通るか**
//!   2. 社内文書で使われる主要な図種を描けるか
//!   3. 所要時間（プレビューの更新に載せられるか）
//!   4. 出力 SVG を **resvg で実際にラスタライズできるか**
//!      （foreignObject による HTML ラベルが含まれると resvg では描けない）

use std::time::Instant;

use merman::{OperationControl, RenderOutput, RenderRequest, Renderer, SvgRequest};

/// 社内文書で想定される図種。v1 の基本設計書に図種の限定は無いため、代表的なものを並べる。
const CASES: &[(&str, &str)] = &[
    (
        "flowchart（v1 サンプルと同じ構成）",
        "flowchart LR\n    MD[Markdown] --> Parse[markdown-it]\n    Parse --> Sanitize[DOMPurify]\n    Sanitize --> DOM[DOM 挿入]",
    ),
    (
        "flowchart（日本語ラベル）",
        "flowchart TD\n    A[要件定義] --> B{承認されたか}\n    B -->|はい| C[基本設計]\n    B -->|いいえ| A",
    ),
    (
        "sequenceDiagram",
        "sequenceDiagram\n    participant U as 利用者\n    participant A as アプリ\n    U->>A: ファイルを開く\n    A-->>U: プレビューを表示",
    ),
    (
        "classDiagram",
        "classDiagram\n    class Document {\n        +String path\n        +save()\n    }\n    Document <|-- MarkdownDocument",
    ),
    (
        "stateDiagram-v2",
        "stateDiagram-v2\n    [*] --> 編集中\n    編集中 --> 保存済み: 保存\n    保存済み --> 編集中: 変更",
    ),
    (
        "erDiagram",
        "erDiagram\n    USER ||--o{ DOCUMENT : owns\n    DOCUMENT {\n        string title\n    }",
    ),
    (
        "gantt",
        "gantt\n    title 開発計画\n    dateFormat YYYY-MM-DD\n    section 設計\n    技術選定 :a1, 2026-09-14, 7d\n    基本設計 :after a1, 14d",
    ),
    (
        "pie",
        "pie title 内訳\n    \"設計\" : 30\n    \"実装\" : 50\n    \"検証\" : 20",
    ),
];

fn main() {
    let renderer = Renderer::new();
    println!("{:<34} | {:>8} | {:>9} | {:>22}", "図種", "結果", "所要", "SVG / ラスタライズ");
    println!("{}", "-".repeat(84));

    let mut ok = 0usize;
    let mut raster_ok = 0usize;

    for (label, source) in CASES {
        let start = Instant::now();
        let result = renderer.render(RenderRequest::svg(
            source,
            OperationControl::new(),
            SvgRequest::default(),
        ));
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        match result {
            Ok(RenderOutput::Svg(Some(output))) => {
                ok += 1;
                let svg = output.svg();
                // foreignObject が入っていると resvg では描けない（HTML ラベル）。
                let has_foreign = svg.contains("foreignObject");
                let raster = rasterize(svg);
                if raster.is_some() {
                    raster_ok += 1;
                }
                let note = match (raster, has_foreign) {
                    (Some((w, h)), false) => format!("{:.0}KB → {}x{}px", svg.len() as f64 / 1024.0, w, h),
                    (Some((w, h)), true) => format!("{:.0}KB → {}x{}px ※foreignObject", svg.len() as f64 / 1024.0, w, h),
                    (None, _) => format!("{:.0}KB → ラスタライズ不可", svg.len() as f64 / 1024.0),
                };
                println!("{label:<34} | {:>8} | {elapsed:>6.1}ms | {note:>22}", "成功");
            }
            Ok(_) => println!("{label:<34} | {:>8} | {elapsed:>6.1}ms | 図として認識されず", "不一致"),
            Err(error) => {
                let text = format!("{error}");
                let head: String = text.chars().take(40).collect();
                println!("{label:<34} | {:>8} | {elapsed:>6.1}ms | {head}", "失敗");
            }
        }
    }

    println!("\n描画成功 {ok}/{} ／ ラスタライズ成功 {raster_ok}/{}", CASES.len(), CASES.len());

    foreign_object_check();
    site_config_check();
}


/// resvg に渡すオプション。
///
/// **`Options::default()` のフォントデータベースは空**である。そのまま使うと
/// `<text>` が一切描かれず、「ラスタライズできた」ように見えて中身が空になる。
/// §6.5.5 と同じ話で、フォントは自前で用意する必要がある。
fn svg_options() -> resvg::usvg::Options<'static> {
    let mut options = resvg::usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    // 日本語が既定で引かれるようにしておく（本番は同梱フォントを指定する。DEC-209）。
    options.font_family = "Meiryo".to_owned();
    options
}

/// resvg で SVG を実際に画像化できるか確かめる。
///
/// 画面表示も PDF 出力も、最終的には SVG を描けることが前提になる。
/// **SVG が出ること**と**描けること**は別なので、ここまで確認する。
fn rasterize(svg: &str) -> Option<(u32, u32)> {
    let options = svg_options();
    let tree = resvg::usvg::Tree::from_str(svg, &options).ok()?;
    let size = tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())?;
    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());
    Some((size.width(), size.height()))
}

/// foreignObject（HTML ラベル）の影響を確かめる。
///
/// resvg は foreignObject を描けない。SVG のサイズが返っても、**中のラベル文字が
/// 落ちている**可能性がある。サイズだけ見て「描けた」と判断してはいけない。
///
/// 同じ図を「既定」と「htmlLabels を切った状態」で描き、**実際に暗い画素数**を
/// 比べる。既定の側だけ極端に少なければ、ラベルが欠落している。
fn foreign_object_check() {
    println!("\n--- foreignObject（HTML ラベル）の影響 ---");

    let renderer = Renderer::new();
    let base = "flowchart TD\n    A[要件定義] --> B{承認されたか}\n    B -->|はい| C[基本設計]\n    B -->|いいえ| A";
    let no_html = format!(
        "%%{{init: {{\"flowchart\": {{\"htmlLabels\": false}}}} }}%%\n{base}"
    );

    for (label, source) in [("既定（htmlLabels 有効）", base), ("htmlLabels: false", no_html.as_str())] {
        let Ok(RenderOutput::Svg(Some(output))) = renderer.render(RenderRequest::svg(
            source,
            OperationControl::new(),
            SvgRequest::default(),
        )) else {
            println!("{label:<26} | 描画に失敗");
            continue;
        };
        let svg = output.svg();
        let foreign = svg.matches("foreignObject").count();
        let text_elements = svg.matches("<text").count();
        match ink(svg) {
            Some((w, h, ink_pixels)) => println!(
                "{label:<26} | foreignObject {foreign:>2} 箇所 / <text> {text_elements:>2} 箇所 | {w}x{h}px 中 暗い画素 {ink_pixels}",
            ),
            None => println!("{label:<26} | ラスタライズ不可"),
        }
    }
}

/// ラスタライズして、透明でない画素の数を返す。
fn ink(svg: &str) -> Option<(u32, u32, u32)> {
    let options = svg_options();
    let tree = resvg::usvg::Tree::from_str(svg, &options).ok()?;
    let size = tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())?;
    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());
    // 背景が不透明に塗られるため、alpha では区別できない（全画素が「塗られた」になる）。
    // 文字・線が出ているかを見たいので、**暗い画素**だけを数える。
    let dark = pixmap
        .pixels()
        .iter()
        .filter(|p| {
            let luminance = (p.red() as u32 * 299 + p.green() as u32 * 587 + p.blue() as u32 * 114) / 1000;
            p.alpha() > 16 && luminance < 128
        })
        .count() as u32;
    Some((size.width(), size.height(), dark))
}

/// サイト全体の設定で HTML ラベルを無効化できるか確かめる。
///
/// `%%{init: ...}%%` を文書ごとに書かせるのは現実的でない。
/// **アプリ側で一括して切れる**必要がある。
fn site_config_check() {
    println!("\n--- サイト設定で htmlLabels を一括無効化できるか ---");

    let config = merman::MermaidConfig::from_value(serde_json::json!({
        "htmlLabels": false,
        "flowchart": { "htmlLabels": false },
        "class": { "htmlLabels": false },
        "state": { "htmlLabels": false },
        "er": { "htmlLabels": false },
    }));
    let renderer = Renderer::new().with_engine(merman::Engine::new().with_site_config(config));

    let mut total_foreign = 0usize;
    for (label, source) in CASES {
        let Ok(RenderOutput::Svg(Some(output))) = renderer.render(RenderRequest::svg(
            source,
            OperationControl::new(),
            SvgRequest::default(),
        )) else {
            println!("{label:<34} | 描画に失敗");
            continue;
        };
        let svg = output.svg();
        let foreign = svg.matches("foreignObject").count();
        let texts = svg.matches("<text").count();
        total_foreign += foreign;
        let dark = ink(svg).map(|(_, _, d)| d).unwrap_or(0);
        println!("{label:<34} | foreignObject {foreign:>2} / <text> {texts:>2} | 暗い画素 {dark:>6}");
    }

    if total_foreign == 0 {
        println!("\n→ サイト設定で一括無効化できた。resvg で描画可能。");
    } else {
        println!("\n→ foreignObject が {total_foreign} 箇所残る。resvg 単体では文字が欠ける。");
    }
}
