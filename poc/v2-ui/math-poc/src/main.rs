//! 数式描画（latex-rust）の検証。技術選定書 §7.1 の「数式」の要 PoC。
//!
//! v1 は KaTeX を WebView 上で動かしていた。v2 では WebView を持たないため、
//! 純 Rust 実装に置き換える必要がある。
//!
//! 確認すること:
//!   1. v1 のサンプルにある数式が通るか
//!   2. 技術文書で使われる代表的な記法（分数・総和・積分・行列・場合分け）が通るか
//!   3. `\text{}` に**日本語**を入れられるか（社内文書では頻出）
//!   4. 出力 SVG を resvg で描けるか（§7.3 で Mermaid が引っかかった点）
//!   5. 所要時間

use std::time::Instant;

use latex_rust::{latex_to_svg, MathFont, SvgOptions};

/// 検証するケース。「表示」は display（別行立て）か inline かを示す。
const CASES: &[(&str, &str, bool)] = &[
    ("v1 サンプル（インライン）", "E = mc^2", false),
    ("v1 サンプル（別行立て）", r"\int_0^\infty e^{-x^2}\,dx = \frac{\sqrt{\pi}}{2}", true),
    ("分数", r"\frac{a + b}{c - d}", false),
    ("総和", r"\sum_{i=1}^{n} x_i^2", true),
    ("総乗と極限", r"\lim_{n \to \infty} \prod_{k=1}^{n} a_k", true),
    ("根号", r"\sqrt[3]{x^2 + y^2}", false),
    ("ギリシャ文字と添字", r"\alpha_1 + \beta^{2} + \gamma_{ij}", false),
    ("行列", r"\begin{pmatrix} a & b \ c & d \end{pmatrix}", true),
    ("場合分け", r"f(x) = \begin{cases} 1 & x > 0 \ 0 & x \le 0 \end{cases}", true),
    ("演算子と関係子", r"a \ne b \le c \ge d \times e \div f \pm g", false),
    ("上下線・帽子", r"\bar{x} \hat{y} \vec{v} \overline{AB}", false),
    ("text（英字）", r"\text{if } x > 0", false),
    ("text（**日本語**）", r"\text{面積} = \pi r^2", false),
    // --- 環境（行の積み上げ）が効いているかの確認 ---
    // 効いていれば 2 行ぶんの高さになる。1 行のままなら `\\` が無視されている。
    ("段組み確認: 基準 x^2（1 行）", r"x^2", true),
    ("段組み確認: pmatrix 2 行", r"\begin{pmatrix} a \\ b \end{pmatrix}", true),
    ("段組み確認: aligned 2 行", r"\begin{aligned} a &= 1 \\ b &= 2 \end{aligned}", true),
];

fn main() {
    let font = match MathFont::stix_two_math() {
        Ok(font) => font,
        Err(error) => {
            eprintln!("埋め込みフォントの読み込みに失敗: {error}");
            std::process::exit(1);
        }
    };

    println!("{:<28} | {:>6} | {:>8} | {:>9} | {}", "ケース", "結果", "所要", "SVG", "ラスタライズ");
    println!("{}", "-".repeat(78));

    let mut ok = 0usize;
    let mut raster_ok = 0usize;

    for (label, latex, display) in CASES {
        let options = SvgOptions { display: *display, ..SvgOptions::default() };
        let start = Instant::now();
        let result = latex_to_svg(latex, &font, &options);
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        match result {
            Ok(svg) => {
                ok += 1;
                // 数式は文字ではなくパスで出ているはず。パスなら外部フォント不要で描ける。
                let paths = svg.matches("<path").count();
                let texts = svg.matches("<text").count();
                let raster = rasterize(&svg);
                if raster.is_some() {
                    raster_ok += 1;
                }
                let note = match raster {
                    Some((w, h, dark)) => format!("{w}x{h}px 暗い画素 {dark}"),
                    None => "不可".to_owned(),
                };
                println!(
                    "{label:<28} | {:>6} | {elapsed:>5.2}ms | path {paths:>2} text {texts:>2} | {note}",
                    "成功"
                );
            }
            Err(error) => {
                let text = format!("{error}");
                let head: String = text.chars().take(34).collect();
                println!("{label:<28} | {:>6} | {elapsed:>5.2}ms | {head}", "失敗");
            }
        }
    }

    println!("\n描画成功 {ok}/{} ／ ラスタライズ成功 {raster_ok}/{}", CASES.len(), CASES.len());
}

/// resvg で描けるか、実際に暗い画素が出るかを見る。
///
/// **フォントは読み込まない。** 数式がパスで出ていれば外部フォント無しで描けるはずで、
/// それを確かめるのがここでの目的である（§7.3 の Mermaid とは逆の確認）。
fn rasterize(svg: &str) -> Option<(u32, u32, u32)> {
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg, &options).ok()?;
    let size = tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width().max(1), size.height().max(1))?;
    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());
    let dark = pixmap
        .pixels()
        .iter()
        .filter(|p| {
            let luminance =
                (p.red() as u32 * 299 + p.green() as u32 * 587 + p.blue() as u32 * 114) / 1000;
            p.alpha() > 16 && luminance < 128
        })
        .count() as u32;
    Some((size.width(), size.height(), dark))
}
