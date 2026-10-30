//! Markdown パーサの所要時間を測る。
//!
//! 現行実装の markdown-it は 300KB で 140ms、1MB で 417ms（実装方針書 §8.4）。
//! v2.0.0 で狙う 10MB に対し、Rust のパーサが何桁の世界にいるのかを確かめる。
//!
//! 実行: parse-poc.exe [MB ...]   既定は 0.3 / 1 / 10 MB

use std::time::Instant;

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    values[values.len() / 2]
}

fn main() {
    let sizes: Vec<f64> = {
        let args: Vec<f64> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
        if args.is_empty() { vec![0.3, 1.0, 10.0] } else { args }
    };

    println!("{:>8} | {:>16} | {:>16} | {:>12}", "入力", "comrak (AST+HTML)", "pulldown (AST)", "生成 HTML");
    println!("{}", "-".repeat(64));

    for mb in sizes {
        let source = poc_core::generate((mb * 1024.0 * 1024.0) as usize);
        // 1MB を超えると試行ごとの分散が小さくなるので回数を落とす。
        let iterations = if source.len() > 2 * 1024 * 1024 { 3 } else { 7 };

        let options = comrak::Options::default();
        // ウォームアップ。初回はアロケータの立ち上がりぶん遅く、中央値まで歪む。
        let mut html = comrak::markdown_to_html(&source, &options);

        let mut comrak_ms = Vec::new();
        for _ in 0..iterations {
            let start = Instant::now();
            html = comrak::markdown_to_html(&source, &options);
            comrak_ms.push(start.elapsed().as_secs_f64() * 1000.0);
        }

        let mut pulldown_ms = Vec::new();
        for _ in 0..iterations {
            let start = Instant::now();
            let count = pulldown_cmark::Parser::new(&source).count();
            pulldown_ms.push(start.elapsed().as_secs_f64() * 1000.0);
            std::hint::black_box(count);
        }

        println!(
            "{:>6.1}MB | {:>13.1}ms | {:>13.1}ms | {:>9.1}MB",
            source.len() as f64 / 1_048_576.0,
            median(&mut comrak_ms),
            median(&mut pulldown_ms),
            html.len() as f64 / 1_048_576.0
        );
    }
}
