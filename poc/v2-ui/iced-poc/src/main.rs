//! iced 版 PoC。egui 版と同一条件で計測する。
//!
//! 10MB の文書をロープに載せ、**可視 44 行だけ**を毎回描き直しながら
//! 一定時間スクロールし続ける。
//!
//! 実行:
//!   iced-poc.exe [MB]          計測モード（既定 10MB、15 秒で自動終了）
//!   POC_MODE=ime iced-poc.exe  IME 確認モード

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// view() が実際に呼ばれた回数＝実描画回数。
///
/// update() の呼び出し回数（タイマー刻み）と一致しないため、別に数える。
/// view は &State しか取れないので、グローバルなカウンタで数えるしかない。
static VIEWS: AtomicU64 = AtomicU64::new(0);

use iced::widget::{column, container, scrollable, text, text_input, Column};
use iced::{Element, Font, Length, Subscription, Task};
use poc_core::{Doc, Harness, VISIBLE_LINES};

/// 再描画の駆動間隔。vsync（約 60Hz）に合わせ、egui 版と同じ回数だけ回るようにする。
/// ここを短くすると iced 側だけ update が余計に走り、比較にならない。
const TICK_MS: u64 = 16;

fn main() -> iced::Result {
    // 起動内訳を出すための基準時刻。State::new より前に置く。
    PROCESS_START.with(|t| t.set(Some(Instant::now())));
    iced::application(State::new, update, view)
        .subscription(subscription)
        .title("iced PoC")
        .window_size(iced::Size::new(1200.0, 800.0))
        .run()
}

thread_local! {
    static PROCESS_START: std::cell::Cell<Option<Instant>> = const { std::cell::Cell::new(None) };
}

fn since_start_ms() -> f64 {
    PROCESS_START
        .with(|t| t.get())
        .map_or(0.0, |s| s.elapsed().as_secs_f64() * 1000.0)
}

struct State {
    doc: Doc,
    harness: Harness,
    lines: Vec<String>,
    ime_mode: bool,
    input: String,
    ime_log: Vec<String>,
    insert_ms: f64,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    Input(String),
    /// Enter が text_input の確定として抜けてきた場合に飛ぶ（確認項目 4）。
    Submit,
    /// IME・キーボードの生イベント（確認項目 3・4）。
    Event(iced::Event),
}

impl State {
    fn new() -> Self {
        let ime_mode = poc_core::is_ime_mode();
        eprintln!("POCMODE iced ime={ime_mode}");
        let bytes = poc_core::target_bytes_from_args();
        eprintln!("POCPHASE iced state_new_enter {:.0}ms", since_start_ms());
        let doc = Doc::new(poc_core::generate(bytes));
        let harness = Harness::new("iced", doc.line_count());
        eprintln!("POCPHASE iced state_new_exit {:.0}ms", since_start_ms());
        let mut lines = Vec::with_capacity(VISIBLE_LINES);
        doc.visible(0, &mut lines);
        Self {
            doc,
            harness,
            lines,
            ime_mode,
            input: String::new(),
            ime_log: Vec::new(),
            insert_ms: 0.0,
        }
    }
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Input(value) => {
            state.input = value;
            Task::none()
        }
        Message::Submit => {
            state.ime_log.push("SUBMIT Enter がアプリ側へ到達".to_owned());
            Task::none()
        }
        Message::Event(event) => {
            // マウス移動・ウィンドウイベントは量が多く、IME の確認に関係しないため落とす。
            let text = format!("{event:?}");
            if !text.starts_with("Mouse") && !text.starts_with("Window") && !text.starts_with("Touch") {
                state.ime_log.push(text);
                if state.ime_log.len() > 24 {
                    state.ime_log.remove(0);
                }
            }
            Task::none()
        }
        Message::Tick => {
            if state.ime_mode {
                return Task::none();
            }
            // 計測対象: 可視行の取り出し。整形と描画は view 以降で iced 側が行う。
            state.doc.visible(state.harness.scroll_top, &mut state.lines);
            if state.harness.tick() {
                state.insert_ms = state.doc.measure_insert(state.doc.bytes / 2);
                state.harness.report(&state.doc, state.insert_ms);
                println!("POCVIEWS {}", VIEWS.load(Ordering::Relaxed));
                return iced::exit();
            }
            Task::none()
        }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    if VIEWS.fetch_add(1, Ordering::Relaxed) == 0 {
        eprintln!("POCPHASE iced first_view {:.0}ms", since_start_ms());
    }
    if state.ime_mode {
        return ime_view(state);
    }

    // 可視行ぶんだけウィジェットを作る（仮想化）。
    // 全行を Column に積むと 10MB では組み立てだけで破綻するため、ここは必須。
    let rows: Column<Message> = column(
        state
            .lines
            .iter()
            .map(|line| text(line).font(Font::MONOSPACE).size(13).into()),
    )
    .spacing(0);

    container(rows).width(Length::Fill).height(Length::Fill).padding(8).into()
}

/// IME 確認モード。iced 側が未確定文字列をどう扱うかを人が見て判断する。
fn ime_view(state: &State) -> Element<'_, Message> {
    container(
        scrollable(
            column![
                text("iced IME 確認").size(22),
                text("下の欄に日本語を入力し、変換候補の位置・未確定の下線・確定を確認する"),
                text_input("ここに日本語を入力", &state.input)
                    .on_input(Message::Input)
                    .on_submit(Message::Submit)
                    .padding(8),
                text(format!("欄の中身: {}", state.input)),
                text("受信したイベント（IME・キーボード）:"),
                column(
                    state
                        .ime_log
                        .iter()
                        .map(|entry| text(entry).font(Font::MONOSPACE).size(12).into()),
                ),
            ]
            .spacing(12),
        )
    )
    .padding(16)
    .into()
}

fn subscription(state: &State) -> Subscription<Message> {
    if state.ime_mode {
        // IME 確認モードではスクロールさせない。生イベントだけを拾う。
        return iced::event::listen().map(Message::Event);
    }
    iced::time::every(Duration::from_millis(TICK_MS)).map(|_| Message::Tick)
}
