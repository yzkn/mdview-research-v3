//! egui (eframe) 版 PoC。
//!
//! 10MB の文書をロープに載せ、**可視 44 行だけ**を毎フレーム描き直しながら
//! 一定時間スクロールし続ける。iced 版と同じ軌道・同じ行数で走らせる。
//!
//! 実行:
//!   egui-poc.exe [MB]          計測モード（既定 10MB、15 秒で自動終了）
//!   POC_MODE=ime egui-poc.exe  IME 確認モード（日本語入力を手で試す）

use std::time::Instant;

use eframe::egui;
use poc_core::{Doc, Harness, VISIBLE_LINES};

const LINE_HEIGHT: f32 = 18.0;

thread_local! {
    static PROCESS_START: std::cell::Cell<Option<Instant>> = const { std::cell::Cell::new(None) };
}

fn since_start_ms() -> f64 {
    PROCESS_START
        .with(|t| t.get())
        .map_or(0.0, |s| s.elapsed().as_secs_f64() * 1000.0)
}

fn main() -> eframe::Result {
    PROCESS_START.with(|t| t.set(Some(Instant::now())));
    let ime_mode = poc_core::is_ime_mode();
    eprintln!("POCMODE egui ime={ime_mode}");
    let bytes = poc_core::target_bytes_from_args();
    eprintln!("POCPHASE egui state_new_enter {:.0}ms", since_start_ms());
    let doc = Doc::new(poc_core::generate(bytes));
    eprintln!("POCPHASE egui state_new_exit {:.0}ms", since_start_ms());

    // eframe 0.36 の NativeOptions に vsync の切り替えは無い。
    // iced 側にも無いため、両者とも vsync 上限（約 60fps）で揃う。
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui PoC",
        options,
        Box::new(move |cc| {
            install_japanese_font(&cc.egui_ctx);
            Ok(Box::new(PocApp::new(doc, ime_mode)))
        }),
    )
}

/// 日本語フォントを手で組み込む。
///
/// **これ自体が選定上の所見である**: egui の既定フォントに CJK は含まれず、
/// システムフォントへのフォールバックも無いため、アプリ側でフォントを
/// 読み込んで差し込む必要がある。入れなければ日本語は豆腐（□）になる。
fn install_japanese_font(ctx: &egui::Context) -> bool {
    const CANDIDATES: [&str; 3] = [
        "C:/Windows/Fonts/meiryo.ttc",
        "C:/Windows/Fonts/YuGothR.ttc",
        "C:/Windows/Fonts/yumin.ttf",
    ];

    for path in CANDIDATES {
        let Ok(bytes) = std::fs::read(path) else { continue };
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "jp".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(bytes)),
        );
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            fonts.families.entry(family).or_default().insert(0, "jp".to_owned());
        }
        ctx.set_fonts(fonts);
        eprintln!("POCFONT egui loaded {path}");
        return true;
    }

    eprintln!("POCFONT egui FAILED - 日本語が豆腐になる");
    false
}

struct PocApp {
    doc: Doc,
    harness: Harness,
    lines: Vec<String>,
    ime_mode: bool,
    ime_log: Vec<String>,
    ime_text: String,
    insert_ms: f64,
}

impl PocApp {
    fn new(doc: Doc, ime_mode: bool) -> Self {
        let harness = Harness::new("egui", doc.line_count());
        Self {
            doc,
            harness,
            lines: Vec::with_capacity(VISIBLE_LINES),
            ime_mode,
            ime_log: Vec::new(),
            ime_text: String::new(),
            insert_ms: 0.0,
        }
    }

    /// IME 確認モード。受け取った IME イベントをそのまま画面に出す。
    /// 未確定文字列・確定・取り消しが正しく届くかを人が見て判断する。
    fn ime_panel(&mut self, ui: &mut egui::Ui) {
        ui.ctx().input(|input| {
            for event in &input.events {
                match event {
                    egui::Event::Ime(ime) => self.ime_log.push(format!("IME  {ime:?}")),
                    // 変換確定の Enter がアプリ側へ抜けていないかを見る（確認項目 4）。
                    egui::Event::Key { key: egui::Key::Enter, pressed: true, .. } => {
                        self.ime_log.push("KEY  Enter (アプリ側へ到達)".to_owned())
                    }
                    _ => {}
                }
            }
        });
        if self.ime_log.len() > 24 {
            let excess = self.ime_log.len() - 24;
            self.ime_log.drain(..excess);
        }

        ui.heading("egui IME 確認");
        ui.label("下の欄に日本語を入力し、変換候補の位置・未確定の下線・確定を確認する");
        ui.add(
            egui::TextEdit::multiline(&mut self.ime_text)
                .desired_width(f32::INFINITY)
                .desired_rows(6),
        );
        ui.separator();
        ui.label("受信した IME イベント:");
        egui::ScrollArea::vertical().show(ui, |ui| {
            for entry in &self.ime_log {
                ui.monospace(entry);
            }
        });
    }
}

impl eframe::App for PocApp {
    // eframe 0.36 では App::update ではなく App::ui。Ui が直接渡される。
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        if self.ime_mode {
            self.ime_panel(ui);
            return;
        }

        if self.harness.frames == 0 {
            eprintln!("POCPHASE egui first_view {:.0}ms", since_start_ms());
        }

        // 計測対象: 可視行の取り出し → テキスト整形 → 描画プリミティブ生成。
        self.doc.visible(self.harness.scroll_top, &mut self.lines);

        let origin = ui.max_rect().min;
        let font = egui::FontId::monospace(13.0);
        let painter = ui.painter();
        for (row, line) in self.lines.iter().enumerate() {
            painter.text(
                origin + egui::vec2(8.0, row as f32 * LINE_HEIGHT),
                egui::Align2::LEFT_TOP,
                line,
                font.clone(),
                egui::Color32::from_gray(220),
            );
        }

        // 走り続けさせる。これをしないと入力が無い間は描画が止まる。
        ctx.request_repaint();

        if self.harness.tick() {
            self.insert_ms = self.doc.measure_insert(self.doc.bytes / 2);
            self.harness.report(&self.doc, self.insert_ms);
            println!("POCVIEWS {}", self.harness.frames);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
