//! v3 の PC 版の UI の試作（調査用）。
//!
//! - **メニューの幅を中身に合わせる**: 見出しも開いたメニューも、字の幅を測って決める（本体は 96px・320px の決め打ち）
//! - **下の階層を横へ開く**: ▸ の行に乗るか押すと、右に次の段が開く（本体はその場で縦に広がる）
//! - **OS に寄せた見た目**: 寸法・字体・強調色・ダイアログのボタンの並びを OS ごとに変える。別の OS の見た目にも切り替えて見比べられる
//! - **macOS では画面の上端のメニューバー**（`native_menu`）。窓の中のメニューは出さない
//!
//! 環境変数 `POC_SHOT_DIR` を与えると、決まった場面を順に開いて窓を PNG に撮り、終わる（外からは画面を撮れないため）
mod look;
mod model;
#[cfg(target_os = "macos")]
mod native_menu;

use iced::advanced::text::{self as core_text, Paragraph as _, Text};
use iced::widget::{
    button, checkbox, column, container, mouse_area, pick_list, row, rule, stack, text, text_input,
    Space,
};
use iced::{Element, Font, Length, Pixels, Size, Subscription, Task, Theme};

use look::{Look, Os};
use model::{Heading, Node};

fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .title("mdview（v3 の UI の試作）")
        .subscription(App::subscription)
        .theme(App::theme)
        .window_size(Size::new(1000.0, 680.0))
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    /// 見出しを押した（開いていれば閉じる）
    OpenMenu(usize),
    /// メニューが開いている間に別の見出しへ乗った
    HoverHeading(usize),
    /// 下の階層のある行に乗った・押した（`depth` 段目の `index` 番目）
    OpenSub(usize, usize),
    /// 押すと動く行に乗った（その段より深い下の階層を閉じる）
    HoverAction(usize),
    Run(String),
    CloseMenu,
    Look(Os),
    Dark(bool),
    Name(String),
    Remember(bool),
    NativeDialog,
    DialogClosed(String),
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    Init,
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    PollNativeMenu,
    /// 撮る場面を 1 つ進める
    ShotTick,
    Shot(iced::window::Screenshot),
}

/// 撮る場面（名前・見た目・暗いか・開く見出しと下の階層）
const SHOTS: [(&str, Os, bool, usize, &[usize]); 5] = [
    ("1-windows-file-open", Os::Windows, false, 0, &[3]),
    ("2-windows-edit-nested", Os::Windows, false, 1, &[10, 2]),
    ("3-windows-dark-file-compare", Os::Windows, true, 0, &[5]),
    ("4-macos-look-file-export", Os::Mac, false, 0, &[11]),
    ("5-gnome-look-dark-view", Os::Gnome, true, 2, &[]),
];

struct App {
    menus: Vec<Heading>,
    /// 開いている見出しと、開いている下の階層の番号の並び
    open: Option<(usize, Vec<usize>)>,
    look: Look,
    dark: bool,
    name: String,
    remember: bool,
    last: String,
    #[cfg(target_os = "macos")]
    native: Option<native_menu::NativeMenu>,
    /// 撮る先と、いまの場面の番号
    shot: Option<(std::path::PathBuf, usize)>,
}

/// 字の幅を測る（本体の `render/measure.rs` と同じ口。描画役の値は要らない）
fn text_width(content: &str, size: f32, font: Font) -> f32 {
    if content.is_empty() {
        return 0.0;
    }
    let paragraph = <<iced::Renderer as core_text::Renderer>::Paragraph as core_text::Paragraph>::with_text(Text {
        content,
        bounds: Size::new(f32::INFINITY, size * 2.0),
        size: Pixels(size),
        line_height: core_text::LineHeight::default(),
        font,
        align_x: core_text::Alignment::Left,
        align_y: iced::alignment::Vertical::Top,
        shaping: core_text::Shaping::Advanced,
        wrapping: core_text::Wrapping::None,
    });
    paragraph.min_width().ceil()
}

/// 見出しの左右の余白
const HEADING_PAD: f32 = 10.0;
/// 開いたメニューの内側の余白
const PANEL_PAD: f32 = 6.0;
/// 名前と打鍵の間
const SHORTCUT_GAP: f32 = 32.0;
/// 左の印の列（● など）と右の ▸ の列
const MARK_COLUMN: f32 = 22.0;
/// 区切りの高さ
const SEPARATOR: f32 = 9.0;

impl App {
    fn boot() -> (App, Task<Message>) {
        let app = App {
            menus: model::menus(),
            open: None,
            look: Look::of(Os::current()),
            dark: false,
            name: String::new(),
            remember: true,
            last: "（まだ何も押していない）".into(),
            #[cfg(target_os = "macos")]
            native: None,
            shot: std::env::var_os("POC_SHOT_DIR").map(|dir| (dir.into(), 0)),
        };
        // macOS: アプリができてから OS のメニューを組む
        (app, Task::done(Message::Init))
    }

    fn native_menu_bar(&self) -> bool {
        cfg!(target_os = "macos")
    }

    fn theme(&self) -> Theme {
        if self.dark {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.shot.is_some() {
            return iced::time::every(std::time::Duration::from_millis(900)).map(|_| Message::ShotTick);
        }
        if self.native_menu_bar() {
            iced::time::every(std::time::Duration::from_millis(50)).map(|_| Message::PollNativeMenu)
        } else {
            Subscription::none()
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenMenu(index) => {
                self.open = match &self.open {
                    Some((open, _)) if *open == index => None,
                    _ => Some((index, Vec::new())),
                };
            }
            Message::HoverHeading(index) => {
                if let Some((open, path)) = &mut self.open {
                    if *open != index {
                        *open = index;
                        path.clear();
                    }
                }
            }
            Message::OpenSub(depth, index) => {
                if let Some((_, path)) = &mut self.open {
                    path.truncate(depth);
                    path.push(index);
                }
            }
            Message::HoverAction(depth) => {
                if let Some((_, path)) = &mut self.open {
                    path.truncate(depth);
                }
            }
            Message::Run(label) => {
                self.last = format!("押した: {label}");
                self.open = None;
            }
            Message::CloseMenu => self.open = None,
            Message::Look(os) => self.look = Look::of(os),
            Message::Dark(dark) => self.dark = dark,
            Message::Name(name) => self.name = name,
            Message::Remember(on) => self.remember = on,
            Message::NativeDialog => {
                let dialog = rfd::AsyncMessageDialog::new()
                    .set_title("mdview")
                    .set_description("保存されていない変更があります。保存しますか？")
                    .set_buttons(rfd::MessageButtons::YesNoCancel);
                return Task::perform(dialog.show(), |result| Message::DialogClosed(format!("{result:?}")));
            }
            Message::DialogClosed(result) => self.last = format!("OS のダイアログ: {result}"),
            Message::Init => {
                #[cfg(target_os = "macos")]
                match native_menu::NativeMenu::install() {
                    Ok(menu) => {
                        // CI の記録に残す（画面の上端は窓の写真に入らない）
                        println!("OS のメニューを組んだ");
                        self.native = Some(menu);
                    }
                    Err(error) => {
                        println!("OS のメニューを組めなかった: {error}");
                        self.last = format!("OS のメニューを組めなかった: {error}");
                    }
                }
            }
            Message::ShotTick => {
                let Some((_, step)) = &self.shot else {
                    return Task::none();
                };
                // 偶数の刻みで場面を作り、奇数の刻みで撮る（**同じ刻みで撮ると、描き終わる前の画面になる**）
                let Some((_, os, dark, menu, path)) = SHOTS.get(*step / 2) else {
                    return iced::exit();
                };
                if *step % 2 == 0 {
                    self.look = Look::of(*os);
                    self.dark = *dark;
                    self.open = Some((*menu, path.to_vec()));
                    if let Some((_, step)) = &mut self.shot {
                        *step += 1;
                    }
                    return Task::none();
                }
                return iced::window::oldest()
                    .and_then(iced::window::screenshot)
                    .map(Message::Shot);
            }
            Message::Shot(shot) => {
                if let Some((dir, step)) = &mut self.shot {
                    if let Some((name, ..)) = SHOTS.get(*step / 2) {
                        let path = dir.join(format!("{name}.png"));
                        let saved = image::save_buffer(
                            &path,
                            &shot.rgba,
                            shot.size.width,
                            shot.size.height,
                            image::ColorType::Rgba8,
                        );
                        println!("{} {:?}", path.display(), saved.err());
                    }
                    *step += 1;
                }
            }
            Message::PollNativeMenu => {
                #[cfg(target_os = "macos")]
                for label in native_menu::poll() {
                    self.last = format!("押した（OS のメニュー）: {label}");
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let body = column![
            self.controls(),
            rule::horizontal(1),
            self.sample_dialog(),
            text(&self.last).size(self.look.font_size).font(self.look.font),
        ]
        .spacing(16)
        .padding(16);

        let page: Element<'_, Message> = if self.native_menu_bar() {
            body.into()
        } else {
            column![self.menu_bar(), rule::horizontal(1), body].into()
        };
        // **重ねる層は、いちばん下の層の大きさで切られる**（iced の stack）。窓いっぱいに広げておかないと、
        // 長いメニューの下の方が見えなくなる（試作で踏んだ）
        let page: Element<'_, Message> = container(page).width(Length::Fill).height(Length::Fill).into();

        match &self.open {
            Some((index, path)) if !self.native_menu_bar() => stack![
                page,
                // 外を押したら閉じる
                mouse_area(Space::new().width(Length::Fill).height(Length::Fill))
                    .on_press(Message::CloseMenu),
                self.dropdown(*index, path),
            ]
            .into(),
            _ => page,
        }
    }

    /// 見出しの幅（字の幅 + 余白）
    fn heading_width(&self, heading: &Heading) -> f32 {
        text_width(&format!("{}({})", heading.label, heading.access), self.look.font_size, self.look.font)
            + HEADING_PAD * 2.0
    }

    fn menu_bar(&self) -> Element<'_, Message> {
        let look = self.look;
        row(self.menus.iter().enumerate().map(|(index, heading)| {
            let open = matches!(&self.open, Some((o, _)) if *o == index);
            let label = text(format!("{}({})", heading.label, heading.access))
                .size(look.font_size)
                .font(look.font);
            let head = button(label)
                .padding([4.0, HEADING_PAD])
                .width(Length::Fixed(self.heading_width(heading)))
                .style(move |theme, status| look.menu_row(theme, status, open))
                .on_press(Message::OpenMenu(index));
            mouse_area(head).on_enter(Message::HoverHeading(index)).into()
        }))
        .padding([2, 4])
        .into()
    }

    /// 1 段の幅: 名前の最大 + 打鍵の最大 + 印の列（**中身から決める**）
    fn panel_width(&self, items: &[Node]) -> f32 {
        let size = self.look.font_size;
        let font = self.look.font;
        let mut label = 0.0_f32;
        let mut shortcut = 0.0_f32;
        for node in items {
            match node {
                Node::Action { label: l, shortcut: s, .. } => {
                    label = label.max(text_width(l, size, font));
                    shortcut = shortcut.max(text_width(s, size * 0.9, font));
                }
                Node::Sub { label: l, .. } => label = label.max(text_width(l, size, font)),
                Node::Separator => {}
            }
        }
        let gap = if shortcut > 0.0 { SHORTCUT_GAP } else { 0.0 };
        MARK_COLUMN * 2.0 + label + gap + shortcut + PANEL_PAD * 2.0 + 16.0
    }

    /// 行の上端（段の中で `index` 番目まで）
    fn row_top(&self, items: &[Node], index: usize) -> f32 {
        items[..index]
            .iter()
            .map(|node| match node {
                Node::Separator => SEPARATOR,
                _ => self.look.menu_row,
            })
            .sum()
    }

    /// 開いたメニュー。**段ごとに横へ並べ、下の階層の段は親の行の高さへずらす**
    fn dropdown<'a>(&'a self, index: usize, path: &'a [usize]) -> Element<'a, Message> {
        let x: f32 = self.menus[..index].iter().map(|h| self.heading_width(h)).sum::<f32>() + 4.0;
        let bar = self.look.menu_row.max(self.look.font_size * 1.3 + 8.0) + 5.0;

        let mut panels = row![Space::new().width(Length::Fixed(x))];
        let mut items: &[Node] = &self.menus[index].items;
        let mut top = 0.0_f32;
        for depth in 0..=path.len() {
            let opened = path.get(depth).copied();
            panels = panels.push(column![
                Space::new().height(Length::Fixed(top)),
                self.panel(items, depth, opened)
            ]);
            let Some(open) = opened else { break };
            // 次の段は、開いた行の高さから始める（縁と余白の分だけ上へ）
            top += self.row_top(items, open);
            match &items[open] {
                Node::Sub { children, .. } => items = children,
                _ => break,
            }
        }
        column![Space::new().height(Length::Fixed(bar)), panels].into()
    }

    fn panel<'a>(&'a self, items: &'a [Node], depth: usize, opened: Option<usize>) -> Element<'a, Message> {
        let look = self.look;
        let width = self.panel_width(items);
        let rows = column(items.iter().enumerate().map(|(index, node)| -> Element<'a, Message> {
            match node {
                Node::Separator => container(rule::horizontal(1))
                    .height(Length::Fixed(SEPARATOR))
                    .center_y(Length::Fixed(SEPARATOR))
                    .padding([0.0, 8.0])
                    .into(),
                Node::Action {
                    label,
                    shortcut,
                    enabled,
                } => {
                    let line = row![
                        Space::new().width(Length::Fixed(MARK_COLUMN)),
                        text(*label).size(look.font_size).font(look.font),
                        Space::new().width(Length::Fill),
                        text(*shortcut).size(look.font_size * 0.9).font(look.font),
                        Space::new().width(Length::Fixed(MARK_COLUMN)),
                    ]
                    .align_y(iced::Alignment::Center);
                    let mut b = button(line)
                        .padding([0, 4])
                        .height(Length::Fixed(look.menu_row))
                        .width(Length::Fill)
                        .style(move |theme, status| look.menu_row(theme, status, false));
                    if *enabled {
                        b = b.on_press(Message::Run((*label).to_owned()));
                    }
                    mouse_area(b).on_enter(Message::HoverAction(depth + 1)).into()
                }
                Node::Sub { label, .. } => {
                    let open = opened == Some(index);
                    let line = row![
                        Space::new().width(Length::Fixed(MARK_COLUMN)),
                        text(*label).size(look.font_size).font(look.font),
                        Space::new().width(Length::Fill),
                        text("›").size(look.font_size * 1.2).font(look.font),
                        Space::new().width(Length::Fixed(6.0)),
                    ]
                    .align_y(iced::Alignment::Center);
                    let b = button(line)
                        .padding([0, 4])
                        .height(Length::Fixed(look.menu_row))
                        .width(Length::Fill)
                        .style(move |theme, status| look.menu_row(theme, status, open))
                        .on_press(Message::OpenSub(depth, index));
                    mouse_area(b).on_enter(Message::OpenSub(depth, index)).into()
                }
            }
        }));
        container(rows)
            .padding(PANEL_PAD)
            .width(Length::Fixed(width))
            .style(move |theme| look.panel(theme))
            .into()
    }

    fn controls(&self) -> Element<'_, Message> {
        let look = self.look;
        row![
            text("見た目").size(look.font_size).font(look.font),
            pick_list(Os::ALL, Some(look.os), Message::Look)
                .text_size(look.font_size)
                .font(look.font),
            checkbox(self.dark)
                .label("暗い配色")
                .on_toggle(Message::Dark)
                .text_size(look.font_size)
                .font(look.font),
            text(if self.native_menu_bar() {
                "メニューは画面の上端（OS のメニューバー）"
            } else {
                "メニューは窓の中（自前で描く。幅は中身から測る）"
            })
            .size(look.font_size * 0.9)
            .font(look.font),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center)
        .into()
    }

    /// 確認のダイアログの見本。**ボタンの並びを OS に合わせる**（Windows は主な操作が左、macOS・GNOME は右端）
    fn sample_dialog(&self) -> Element<'_, Message> {
        let look = self.look;
        let size = look.font_size;
        let pad = look.button_padding();
        let save = button(text("保存").size(size).font(look.font))
            .padding(pad)
            .style(move |t, s| look.primary(t, s))
            .on_press(Message::Run("保存".into()));
        let discard = button(text("保存しない").size(size).font(look.font))
            .padding(pad)
            .style(move |t, s| look.button(t, s))
            .on_press(Message::Run("保存しない".into()));
        let cancel = button(text("キャンセル").size(size).font(look.font))
            .padding(pad)
            .style(move |t, s| look.button(t, s))
            .on_press(Message::Run("キャンセル".into()));
        let buttons = if look.primary_last {
            row![discard, Space::new().width(Length::Fill), cancel, save]
        } else {
            row![Space::new().width(Length::Fill), save, discard, cancel]
        }
        .spacing(8);

        let card = column![
            text("保存されていない変更があります").size(size * 1.15).font(look.font),
            text("閉じる前に「README.md」を保存しますか？").size(size).font(look.font),
            text_input("ファイル名", &self.name)
                .on_input(Message::Name)
                .size(size)
                .font(look.font)
                .padding(pad)
                .style(move |t, s| look.input(t, s)),
            checkbox(self.remember)
                .label("次回も同じ操作をする")
                .on_toggle(Message::Remember)
                .text_size(size)
                .font(look.font),
            buttons,
        ]
        .spacing(12);

        column![
            container(card)
                .padding(20)
                .width(Length::Fixed(460.0))
                .style(move |theme| look.panel(theme)),
            button(text("同じ確認を OS のダイアログで出す（見比べる）").size(size).font(look.font))
                .padding(pad)
                .style(move |t, s| look.button(t, s))
                .on_press(Message::NativeDialog),
        ]
        .spacing(16)
        .into()
    }
}
