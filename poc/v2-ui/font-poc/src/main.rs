//! 同梱フォントの読み込み時間の計測。基本設計書 §20.3 / §25-7 の未検証項目。
//!
//! 起動時間の目標は 2 秒で、PoC の実測では GPU / ウィンドウ初期化だけで
//! 1.0〜1.3 秒を使う。**21.6MB のフォントを起動時に読む余地があるか**を確かめる。
//!
//! 同梱予定の PlemolJP / IBM Plex Sans JP は手元に無いため、
//! **サイズの近いシステムフォントで代用する**。目的は MB あたりの傾向を掴むことで、
//! 書体そのものの評価ではない。

use std::time::Instant;

use fontdb::Database;

/// 代用に使う候補。合計が同梱予定（21.6MB）に近くなるよう複数読む。
const CANDIDATES: &[&str] = &[
    "C:/Windows/Fonts/meiryo.ttc",
    "C:/Windows/Fonts/meiryob.ttc",
    "C:/Windows/Fonts/YuGothR.ttc",
    "C:/Windows/Fonts/YuGothB.ttc",
    "C:/Windows/Fonts/msgothic.ttc",
    "C:/Windows/Fonts/BIZ-UDGothicR.ttc",
];

fn main() {
    println!("同梱フォントの読み込み時間（代用フォントでの計測）\n");

    // 1. ファイルからバイト列を読む（実装では実行ファイルへ埋め込むため、この段は消える）
    let mut fonts: Vec<(&str, Vec<u8>)> = Vec::new();
    for path in CANDIDATES {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.push((path, bytes));
        }
    }
    if fonts.is_empty() {
        eprintln!("代用フォントが見つからない。計測を中止する。");
        std::process::exit(1);
    }

    let total_bytes: usize = fonts.iter().map(|(_, b)| b.len()).sum();
    println!("読み込み対象 {} 件 / 合計 {:.2} MB", fonts.len(), total_bytes as f64 / 1_048_576.0);
    for (path, bytes) in &fonts {
        println!("  {:>6.2} MB  {}", bytes.len() as f64 / 1_048_576.0, path);
    }

    // 2. fontdb へ登録する（＝実装で起動時に走る処理）
    //    埋め込みバイト列からの登録に相当するよう、メモリ上のデータを渡す。
    println!("\n--- fontdb への登録（3 回） ---");
    let mut times = Vec::new();
    let mut face_count = 0usize;
    for _ in 0..3 {
        let start = Instant::now();
        let mut db = Database::new();
        for (_, bytes) in &fonts {
            db.load_font_data(bytes.clone());
        }
        times.push(start.elapsed().as_secs_f64() * 1000.0);
        face_count = db.len();
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = times[times.len() / 2];

    println!("登録 {:.1}ms（{} フェイス）", median, face_count);
    println!("1MB あたり {:.2}ms", median / (total_bytes as f64 / 1_048_576.0));

    // 3. 同梱予定（21.6MB）への外挿
    let per_mb = median / (total_bytes as f64 / 1_048_576.0);
    println!("\n--- 同梱予定 21.6MB への外挿 ---");
    println!("推定 {:.0}ms", per_mb * 21.6);
    println!("\n判定の目安: 起動時間の目標 2 秒に対し、GPU 初期化 1.0〜1.3 秒を除いた");
    println!("残りは 0.7〜1.0 秒。ここに収まるなら遅延読み込みは不要。");
}
