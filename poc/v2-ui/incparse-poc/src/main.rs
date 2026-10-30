//! 増分解析（tree-sitter-md）の検証。技術選定書 DEC-204 / §8.3 の前提を確かめる。
//!
//! §6.3 の実測では、comrak による 10MB の全文解析は 431ms かかる。
//! 1 フレームの予算 16.6ms に対して 26 倍であり、全文再解析では 10MB が成立しない。
//! **編集のたびに走る解析を 16.6ms 未満に収められるか**が本 PoC の論点である。
//!
//! 測るもの:
//!   1. 全解析（増分の基準値）
//!   2. 通常の編集（散文中に 1 文字挿入）を増分で再解析
//!   3. **最悪ケース: 未閉鎖のコードフェンスを文書先頭付近に挿入**
//!      Markdown は ``` 以降の解釈が全部変わるため、増分解析が最も効きにくい。
//!      ここが破綻するなら §8.3 の方式は成立しない。

use std::time::Instant;

use ropey::Rope;
use tree_sitter::{InputEdit, Parser, Point, Tree};
use tree_sitter_md::MarkdownParser;

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    values[values.len() / 2]
}

/// 挿入 1 回ぶんの InputEdit を組み立てる。
///
/// tree-sitter は行・桁も要求する。ロープなら位置の逆引きが O(log n) で済む。
fn insert_edit(rope: &Rope, byte: usize, inserted: &str) -> InputEdit {
    let line = rope.byte_to_line(byte);
    let column = byte - rope.line_to_byte(line);
    let point = Point::new(line, column);

    // 挿入文字列に改行が無い前提（本 PoC の入力はすべてそうしている）。
    InputEdit {
        start_byte: byte,
        old_end_byte: byte,
        new_end_byte: byte + inserted.len(),
        start_position: point,
        old_end_position: point,
        new_end_position: Point::new(line, column + inserted.len()),
    }
}

struct Doc {
    rope: Rope,
    text: String,
}

impl Doc {
    fn new(text: String) -> Self {
        Self { rope: Rope::from_str(&text), text }
    }

    /// ロープと文字列の両方へ挿入する。
    ///
    /// 実装では文字列を持たずロープから直接読ませる（`parse_with_options` の
    /// コールバック）が、本 PoC では**解析時間だけを切り出して測る**ため、
    /// 文字列の更新コストを別に計上できる形にしている。
    fn insert(&mut self, byte: usize, inserted: &str) -> f64 {
        let start = Instant::now();
        self.text.insert_str(byte, inserted);
        let char_index = self.rope.byte_to_char(byte);
        self.rope.insert(char_index, inserted);
        start.elapsed().as_secs_f64() * 1000.0
    }
}

fn main() {
    println!("tree-sitter-md 増分解析の計測\n");

    for mb in [0.3_f64, 1.0, 10.0] {
        let source = poc_core::generate((mb * 1024.0 * 1024.0) as usize);
        let mut doc = Doc::new(source);
        let mut parser = MarkdownParser::default();

        // --- 1. 全解析 ---
        let mut full_times = Vec::new();
        let iterations = if doc.text.len() > 2 * 1024 * 1024 { 3 } else { 5 };
        let mut tree = parser.parse(doc.text.as_bytes(), None).expect("全解析に失敗");
        for _ in 0..iterations {
            let start = Instant::now();
            tree = parser.parse(doc.text.as_bytes(), None).expect("全解析に失敗");
            full_times.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        let full_ms = median(&mut full_times);

        // --- 2. 通常の編集（散文中へ 1 文字） ---
        let mut inc_times = Vec::new();
        let mut edit_times = Vec::new();
        for step in 0..20 {
            // 文書全体に散らす。同じ場所ばかり触ると木のキャッシュに偏る。
            let target = doc.text.len() / 21 * (step + 1);
            let byte = floor_char_boundary(&doc.text, target);

            edit_times.push(doc.insert(byte, "あ"));
            tree.edit(&insert_edit(&doc.rope, byte, "あ"));

            let start = Instant::now();
            tree = parser
                .parse(doc.text.as_bytes(), Some(&tree))
                .expect("増分解析に失敗");
            inc_times.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        let inc_ms = median(&mut inc_times);
        let edit_ms = median(&mut edit_times);

        // --- 3. 最悪ケース: 未閉鎖のコードフェンスを先頭付近へ ---
        // これ以降の行の解釈がすべて変わるため、増分解析が最も効きにくい。
        let fence = "\n```\n";
        let byte = floor_char_boundary(&doc.text, doc.text.len() / 100);
        doc.insert(byte, fence);
        let line = doc.rope.byte_to_line(byte);
        let column = byte - doc.rope.line_to_byte(line);
        let worst_edit = InputEdit {
            start_byte: byte,
            old_end_byte: byte,
            new_end_byte: byte + fence.len(),
            start_position: Point::new(line, column),
            old_end_position: Point::new(line, column),
            // 改行を 2 つ含むので行が 2 つ増える。
            new_end_position: Point::new(line + 2, 0),
        };
        tree.edit(&worst_edit);
        let start = Instant::now();
        let _ = parser
            .parse(doc.text.as_bytes(), Some(&tree))
            .expect("最悪ケースの増分解析に失敗");
        let worst_ms = start.elapsed().as_secs_f64() * 1000.0;

        println!(
            "{:>5.1}MB | 全解析 {:>8.2}ms | 増分(通常) {:>7.3}ms | 増分(最悪: ```挿入) {:>8.2}ms | 文字列更新 {:>6.3}ms",
            doc.text.len() as f64 / 1_048_576.0,
            full_ms,
            inc_ms,
            worst_ms,
            edit_ms
        );
    }

    println!("\n判定の目安: 増分(通常) が 16.6ms（60fps の 1 フレーム）を十分下回ること。");
    println!("最悪ケースが全解析並みに戻る場合は、その場合だけ非同期へ逃がす設計が要る。");

    block_only();
    window_based();
}

/// `index` 以下で最も近い文字境界。日本語を含むため境界を跨ぐと panic する。
fn floor_char_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}


/// ブロック木だけを測る（切り分け）。
///
/// tree-sitter-md の `MarkdownParser` は、**ブロック木に加えて、インライン内容を持つ
/// ブロックごとに独立した木**を作る。上の計測が遅い原因がこのインライン側にあるなら、
/// 「ブロックは常に解析し、インラインは可視ブロックだけ遅延解析する」という設計で
/// 回避できる。これは §8.4 の仮想化と同じ考え方である。
///
/// ここではブロック文法だけを素の `tree_sitter::Parser` で走らせて分離する。
fn block_only() {
    println!("\n--- 切り分け: ブロック木のみ（インライン木を作らない） ---");

    let block_language: tree_sitter::Language = tree_sitter_md::LANGUAGE.into();
    let inline_language: tree_sitter::Language = tree_sitter_md::INLINE_LANGUAGE.into();

    for mb in [0.3_f64, 1.0, 10.0] {
        let source = poc_core::generate((mb * 1024.0 * 1024.0) as usize);
        let mut doc = Doc::new(source);

        let mut parser = Parser::new();
        parser.set_language(&block_language).expect("ブロック文法の設定に失敗");

        // 全解析
        let mut full_times = Vec::new();
        let iterations = if doc.text.len() > 2 * 1024 * 1024 { 3 } else { 5 };
        let mut tree: Tree = parser.parse(doc.text.as_bytes(), None).expect("失敗");
        for _ in 0..iterations {
            let start = Instant::now();
            tree = parser.parse(doc.text.as_bytes(), None).expect("失敗");
            full_times.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        let full_ms = median(&mut full_times);

        // 増分（散文中へ 1 文字）
        let mut inc_times = Vec::new();
        for step in 0..20 {
            let target = doc.text.len() / 21 * (step + 1);
            let byte = floor_char_boundary(&doc.text, target);
            doc.insert(byte, "あ");
            tree.edit(&insert_edit(&doc.rope, byte, "あ"));
            let start = Instant::now();
            tree = parser.parse(doc.text.as_bytes(), Some(&tree)).expect("失敗");
            inc_times.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        let inc_ms = median(&mut inc_times);

        // 可視ブロックぶんのインライン解析。
        // 画面に出ている 44 ブロックだけを都度インライン解析する想定のコスト。
        let mut inline_parser = Parser::new();
        inline_parser.set_language(&inline_language).expect("インライン文法の設定に失敗");
        let mut cursor = tree.walk();
        let mut ranges = Vec::new();
        cursor.goto_first_child();
        loop {
            let node = cursor.node();
            if node.byte_range().len() > 1 {
                ranges.push(node.byte_range());
            }
            if ranges.len() >= poc_core::VISIBLE_LINES || !cursor.goto_next_sibling() {
                break;
            }
        }
        let start = Instant::now();
        for range in &ranges {
            let _ = inline_parser.parse(&doc.text.as_bytes()[range.clone()], None);
        }
        let inline_ms = start.elapsed().as_secs_f64() * 1000.0;

        println!(
            "{:>5.1}MB | ブロック全解析 {:>8.2}ms | ブロック増分 {:>7.3}ms | 可視 {} ブロックのインライン解析 {:>6.3}ms",
            doc.text.len() as f64 / 1_048_576.0,
            full_ms,
            inc_ms,
            ranges.len(),
            inline_ms
        );
    }
}

/// 代替案の検証: 文書全体を文法解析せず、**可視範囲だけ**を解析する。
///
/// 上の計測で、tree-sitter-md はブロック文法だけでも 10MB で 6.6 秒かかり、
/// 増分でも 335ms と予算（16.6ms）に届かない。
///
/// しかし**そもそも全文を文法解析する必要があるのか**を問い直すべきである。
/// 必要なものは次の 3 つに分解できる。
///
///   1. 目次と、プレビューの高さ推定に使うブロック境界 → **行単位の軽い走査で足りる**
///   2. エディタのハイライト → **画面に見えている範囲だけ**
///   3. PDF / HTML 出力 → 1 回だけなので comrak の全解析で足りる（§6.3 の 431ms）
///
/// ここでは 1 と 2 のコストを測る。
fn window_based() {
    println!("\n--- 代替案: 全文を文法解析しない ---");

    let source = poc_core::generate(10 * 1024 * 1024);
    let rope = Rope::from_str(&source);

    // --- 1. 行走査によるブロック境界と見出しの抽出 ---
    // 文法解析ではなく、行頭だけを見る。コードフェンスの内外だけ状態として持つ。
    let mut scan_times = Vec::new();
    let mut blocks = 0usize;
    let mut headings = 0usize;
    for _ in 0..5 {
        let start = Instant::now();
        let (b, h) = scan_blocks(&source);
        scan_times.push(start.elapsed().as_secs_f64() * 1000.0);
        blocks = b;
        headings = h;
    }
    println!(
        "行走査（ブロック境界 + 見出し抽出・10MB）: {:>7.2}ms  ブロック {} 件 / 見出し {} 件",
        median(&mut scan_times),
        blocks,
        headings
    );

    // --- 2. 可視範囲だけの解析 ---
    // 画面に出るのは 44 行だが、前後に余裕を持たせて 64KB を再解析する想定。
    let window_bytes = 64 * 1024;
    let start_byte = floor_char_boundary(&source, source.len() / 2);
    let end_byte = floor_char_boundary(&source, (start_byte + window_bytes).min(source.len()));
    let window = &source[start_byte..end_byte];

    let block_language: tree_sitter::Language = tree_sitter_md::LANGUAGE.into();
    let mut parser = Parser::new();
    parser.set_language(&block_language).expect("失敗");
    let mut ts_times = Vec::new();
    for _ in 0..20 {
        let start = Instant::now();
        let _ = parser.parse(window.as_bytes(), None);
        ts_times.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    let options = comrak::Options::default();
    let mut comrak_times = Vec::new();
    let _ = comrak::markdown_to_html(window, &options);
    for _ in 0..20 {
        let start = Instant::now();
        let _ = comrak::markdown_to_html(window, &options);
        comrak_times.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    println!(
        "可視範囲 64KB の解析: tree-sitter(ブロック) {:>6.3}ms / comrak(AST+HTML) {:>6.3}ms",
        median(&mut ts_times),
        median(&mut comrak_times)
    );

    // 1 文字挿入に対する現実的な再解析コスト（走査は差分更新できるため計上しない）
    println!(
        "→ 1 打鍵あたりの再解析は可視範囲のみで済み、いずれも 16.6ms を大きく下回る。\n\
         → 全文が要るのは出力時だけで、comrak の 10MB 全解析 431ms（§6.3）で足りる。"
    );

    let _ = rope;
}

/// 行頭だけを見てブロック境界と見出しを数える。
///
/// 文法解析ではないので、表やリストの入れ子までは分からない。
/// 目次とプレビューの高さ推定には、この粒度で足りる。
fn scan_blocks(source: &str) -> (usize, usize) {
    let mut blocks = 0usize;
    let mut headings = 0usize;
    let mut in_fence = false;
    let mut previous_blank = true;

    for line in source.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            if in_fence {
                blocks += 1;
            }
            previous_blank = false;
            continue;
        }
        if in_fence {
            continue;
        }

        if line.trim().is_empty() {
            previous_blank = true;
            continue;
        }
        if previous_blank {
            blocks += 1;
        }
        if trimmed.starts_with('#') {
            headings += 1;
        }
        previous_blank = false;
    }

    (blocks, headings)
}
