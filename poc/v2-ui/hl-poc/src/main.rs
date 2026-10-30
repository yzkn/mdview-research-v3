//! コードブロックのハイライト（OPEN-206）の検証。
//!
//! §8.3 で Markdown の文法解析に tree-sitter を使わないと決めたが、
//! **フェンス内のコードの着色は別の話**である。対象は可視範囲の数十行に限られる。
//!
//! 起動時間が既に 1.1〜1.4 秒（§6.2）と厳しいため、
//! **構文定義の読み込みコスト**も併せて測る。

use std::time::Instant;

use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// 可視範囲に入り得るコードブロックの大きさ（行数）。
const BLOCK_LINES: usize = 50;

fn sample_code(language: &str) -> String {
    let unit = match language {
        "Rust" => "fn process(items: &[String], index: usize) -> Option<&String> {\n    // 日本語のコメントを含める\n    items.get(index).filter(|s| !s.is_empty())\n}\n",
        "Python" => "def process(items, index):\n    # 日本語のコメント\n    if index < len(items):\n        return items[index]\n    return None\n",
        _ => "function process(items, index) {\n  // 日本語のコメント\n  return items[index] ?? null;\n}\n",
    };
    unit.repeat(BLOCK_LINES / 4 + 1)
}

fn main() {
    // --- 起動時のコスト ---
    let start = Instant::now();
    let syntaxes = SyntaxSet::load_defaults_newlines();
    let syntax_ms = start.elapsed().as_secs_f64() * 1000.0;

    let start = Instant::now();
    let themes = ThemeSet::load_defaults();
    let theme_ms = start.elapsed().as_secs_f64() * 1000.0;

    println!("起動時のコスト");
    println!("  構文定義の読み込み: {syntax_ms:>6.1}ms（{} 種）", syntaxes.syntaxes().len());
    println!("  テーマの読み込み  : {theme_ms:>6.1}ms（{} 種）", themes.themes.len());
    println!("  合計              : {:>6.1}ms\n", syntax_ms + theme_ms);

    // --- 1 ブロックあたりのハイライト ---
    let theme = &themes.themes["base16-ocean.dark"];
    println!("1 ブロック（約 {BLOCK_LINES} 行）のハイライト");

    for language in ["Rust", "Python", "JavaScript"] {
        let code = sample_code(language);
        let Some(syntax) = syntaxes.find_syntax_by_name(language) else {
            println!("  {language:<12}: 構文定義が見つからない");
            continue;
        };

        // ウォームアップ
        let mut warm = HighlightLines::new(syntax, theme);
        for line in LinesWithEndings::from(&code) {
            let _ = warm.highlight_line(line, &syntaxes);
        }

        let mut times = Vec::new();
        for _ in 0..20 {
            let mut highlighter = HighlightLines::new(syntax, theme);
            let start = Instant::now();
            let mut spans = 0usize;
            for line in LinesWithEndings::from(&code) {
                if let Ok(ranges) = highlighter.highlight_line(line, &syntaxes) {
                    spans += ranges.len();
                }
            }
            times.push((start.elapsed().as_secs_f64() * 1000.0, spans));
        }
        times.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let (ms, spans) = times[times.len() / 2];
        let lines = code.lines().count();
        println!("  {language:<12}: {ms:>6.3}ms（{lines} 行 / {spans} スパン）");
    }

    println!("\n判定の目安: 1 ブロック 16.6ms を十分下回ること。");
    println!("起動コストが大きい場合は、最初のコードブロックが現れるまで遅延して読み込む。");
}
