//! iced / egui PoC の共通部分。
//!
//! 両 PoC が **同一の文書・同一のスクロール軌道・同一の可視行数**を使うようにして、
//! 計測差がツールキット側の描画コストだけになるようにする。

use std::time::Instant;

pub use ropey::Rope;

/// 画面に見えている行数の想定（1200x800 ウィンドウ・行高 18px 相当）。
pub const VISIBLE_LINES: usize = 44;

/// 計測を打ち切るまでの秒数。両 PoC で揃える。
pub const RUN_SECS: f64 = 15.0;

/// 最悪ケースの Markdown を生成する。
///
/// 既存実装の `perf/worstCaseSample.ts` と同じ考え方で、
/// 見出し・日本語の散文・表・コードブロックを一巡させる。
/// 日本語を必ず混ぜるのは、テキスト整形（シェーピング）のコストが
/// ASCII のみの文書では過小評価になるため。
pub fn generate(target_bytes: usize) -> String {
    let mut out = String::with_capacity(target_bytes + 4096);
    let mut section = 0usize;

    while out.len() < target_bytes {
        section += 1;
        out.push_str(&format!("\n## {section} 章 サンプル見出し\n\n"));
        out.push_str(
            "この段落は日本語の散文である。テキスト整形のコストを実際の利用条件に\
             近づけるため、全角文字と半角英数 (ASCII mixed 123) を混在させている。\n\n",
        );
        out.push_str("| 項目 | 内容 | 備考 |\n|---|---|---|\n");
        for row in 0..4 {
            out.push_str(&format!("| 行{row} | 値 {row} | 補足テキスト |\n"));
        }
        out.push_str("\n```rust\nfn sample_function(index: usize) -> usize {\n");
        out.push_str("    // コメントにも日本語を混ぜる\n    index * 2 + 1\n}\n```\n\n");
        out.push_str("- 箇条書きの項目 A\n- 箇条書きの項目 B\n  - ネストした項目\n\n");
    }

    out
}

/// 文書を保持し、可視範囲の行だけを取り出す。
///
/// これが「仮想化」の中身であり、10MB 対応の前提になる。
/// 全行をウィジェット化する素朴な実装との差を見るために PoC で共通化している。
pub struct Doc {
    rope: Rope,
    pub bytes: usize,
    pub load_ms: f64,
}

impl Doc {
    pub fn new(text: String) -> Self {
        let bytes = text.len();
        let start = Instant::now();
        let rope = Rope::from_str(&text);
        let load_ms = start.elapsed().as_secs_f64() * 1000.0;
        Self { rope, bytes, load_ms }
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    /// `first` 行目から `VISIBLE_LINES` 行を取り出す。
    /// 毎フレーム呼ばれるため、ここが O(文書サイズ) になっていないことが重要。
    pub fn visible(&self, first: usize, out: &mut Vec<String>) {
        out.clear();
        let last = (first + VISIBLE_LINES).min(self.line_count());
        for index in first..last {
            let line = self.rope.line(index);
            // 行末の改行は描画しない。極端に長い行は打ち切る（エディタの通常挙動）。
            let text: String = line.chars().take(400).filter(|c| *c != '\n').collect();
            out.push(text);
        }
    }

    /// 1 文字挿入の所要時間を測る。ロープ側の編集コストの確認用。
    pub fn measure_insert(&mut self, char_index: usize) -> f64 {
        let start = Instant::now();
        self.rope.insert_char(char_index.min(self.rope.len_chars()), 'あ');
        start.elapsed().as_secs_f64() * 1000.0
    }
}

/// フレーム数と経過時間を数え、決められた秒数で終了を指示する。
pub struct Harness {
    pub name: &'static str,
    start: Option<Instant>,
    process_start: Instant,
    pub frames: u64,
    pub first_frame_ms: f64,
    pub scroll_top: usize,
    max_top: usize,
}

impl Harness {
    pub fn new(name: &'static str, line_count: usize) -> Self {
        Self {
            name,
            start: None,
            process_start: Instant::now(),
            frames: 0,
            first_frame_ms: 0.0,
            scroll_top: 0,
            max_top: line_count.saturating_sub(VISIBLE_LINES),
        }
    }

    /// 1 フレーム分進める。`true` を返したら計測終了。
    ///
    /// スクロール量は「1 フレームあたり 7 行」で固定する。フレームレートに
    /// 依存させると、速い側ほど遠くまで進んで条件が変わってしまうため。
    pub fn tick(&mut self) -> bool {
        let now = Instant::now();
        if self.start.is_none() {
            self.start = Some(now);
            self.first_frame_ms = self.process_start.elapsed().as_secs_f64() * 1000.0;
        }
        self.frames += 1;
        self.scroll_top += 7;
        if self.scroll_top > self.max_top {
            self.scroll_top = 0;
        }
        self.elapsed() >= RUN_SECS
    }

    pub fn elapsed(&self) -> f64 {
        self.start.map_or(0.0, |s| s.elapsed().as_secs_f64())
    }

    /// 結果を 1 行の JSON で出す。走らせる側はこの行だけを拾えばよい。
    pub fn report(&self, doc: &Doc, insert_ms: f64) {
        let fps = if self.elapsed() > 0.0 { self.frames as f64 / self.elapsed() } else { 0.0 };
        println!(
            "POCRESULT {{\"toolkit\":\"{}\",\"bytes\":{},\"lines\":{},\
             \"rope_load_ms\":{:.2},\"insert_ms\":{:.4},\
             \"first_frame_ms\":{:.1},\"frames\":{},\"elapsed_s\":{:.2},\"fps\":{:.1}}}",
            self.name,
            doc.bytes,
            doc.line_count(),
            doc.load_ms,
            insert_ms,
            self.first_frame_ms,
            self.frames,
            self.elapsed(),
            fps
        );
    }
}

/// 起動引数から対象サイズ（MB）を読む。既定は 10MB。
///
/// 数値として読めない引数（`--ime` 等）は無視して既定値にする。
pub fn target_bytes_from_args() -> usize {
    std::env::args()
        .skip(1)
        .find_map(|a| a.parse::<f64>().ok())
        .map_or(10 * 1024 * 1024, |mb| (mb * 1024.0 * 1024.0) as usize)
}

/// IME 確認モードかどうか。
///
/// **引数 `--ime` を第一の指定手段とする。** 環境変数だけに頼ると、
/// cmd.exe の `set POC_MODE=ime && ...` が**値に末尾の空白を含めてしまい**
/// （`"ime "`）、一致判定から外れて計測モードで起動する事故が起きる。
/// 環境変数も受けるが、前後の空白を落としてから比較する。
pub fn is_ime_mode() -> bool {
    if std::env::args().any(|a| a == "--ime") {
        return true;
    }
    std::env::var("POC_MODE").is_ok_and(|v| v.trim().eq_ignore_ascii_case("ime"))
}
