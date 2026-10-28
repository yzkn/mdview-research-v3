//! 候補の絵文字の字体を、本体と同じ iced で画面に描く（v3 の R-19）。
//!
//! `mdview-emoji-screen-poc <撮る PNG> <字体> [<字体> …]`。字体ごとに 1 列ずつ、同じ行（../v3-pdf-emoji と同じ並び）を描き、
//! 窓を PNG に撮って終わる（外からは画面を撮れないため、窓が自分で撮る）
use iced::widget::{column, row, text};
use iced::{Element, Font, Size, Task};

/// 描く行（../v3-pdf-emoji/src/main.rs と同じ）
const LINES: [&str; 6] = ["😀 🎉 🙂 👍 📝 🐛 🍣", "❤️", "⚠️ ✅", "👨‍👩‍👧", "🇯🇵", "👍🏽"];

struct App {
    shot: std::path::PathBuf,
    /// 字体の名前（族）と、ファイルの名前
    fonts: Vec<(&'static str, String)>,
    ticks: u32,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    Shot(iced::window::Screenshot),
}

/// 字体の族の名前（name の 1 番）
fn family(data: &[u8]) -> String {
    let face = ttf_parser::Face::parse(data, 0).expect("字体を読めない");
    face.names()
        .into_iter()
        .find(|n| n.name_id == ttf_parser::name_id::FAMILY && n.is_unicode())
        .and_then(|n| n.to_string())
        .unwrap_or_else(|| "?".into())
}

fn main() -> iced::Result {
    let mut args = std::env::args().skip(1);
    let shot = std::path::PathBuf::from(args.next().expect("撮る PNG を指す"));
    let mut fonts = Vec::new();
    let mut bytes = Vec::new();
    for path in args {
        let data = std::fs::read(&path).expect("字体を読めない");
        let name: &'static str = Box::leak(family(&data).into_boxed_str());
        let file = std::path::Path::new(&path).file_name().unwrap().to_string_lossy().into_owned();
        println!("{file}: 族 {name}");
        fonts.push((name, file));
        bytes.push(data);
    }
    let width = 40.0 + 340.0 * fonts.len() as f32;
    let mut app = iced::application(
        move || {
            (
                App { shot: shot.clone(), fonts: fonts.clone(), ticks: 0 },
                Task::none(),
            )
        },
        App::update,
        App::view,
    )
    .subscription(|_: &App| iced::time::every(std::time::Duration::from_millis(700)).map(|_| Message::Tick))
    .window_size(Size::new(width, 560.0));
    for data in bytes {
        app = app.font(data);
    }
    app.run()
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.ticks += 1;
                // 字体を読み込み、描き終わるのを待ってから撮る
                if self.ticks == 3 {
                    return iced::window::oldest()
                        .and_then(iced::window::screenshot)
                        .map(Message::Shot);
                }
            }
            Message::Shot(shot) => {
                let saved = image::save_buffer(
                    &self.shot,
                    &shot.rgba,
                    shot.size.width,
                    shot.size.height,
                    image::ColorType::Rgba8,
                );
                println!("撮った: {} {:?}", self.shot.display(), saved.err());
                return iced::exit();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        row(self.fonts.iter().map(|(name, file)| {
            let font = Font::with_name(name);
            let mut lines = column![text(file.clone()).size(14)].spacing(14);
            for line in LINES {
                lines = lines.push(text(line).font(font).size(36).shaping(text::Shaping::Advanced));
            }
            lines.width(340).into()
        }))
        .padding(20)
        .into()
    }
}

