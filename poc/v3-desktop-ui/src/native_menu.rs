//! macOS の OS のメニュー（画面の上端のメニューバー）。`muda` で `NSMenu` を組む。
//!
//! - **中身は自前のメニューと同じ並び**（`model::menus`）から作る。下の階層は OS の下の階層になる
//! - macOS の作法で、先頭にアプリの名前のメニュー（このアプリについて・設定・隠す・終了）を足し、
//!   「設定…」「終了」「このアプリについて…」はそちらへ移す
//! - 打鍵は `Ctrl` を `⌘`（Cmd）へ読み替える。**OS のメニューが打鍵を先に取る**ので、アプリの側で同じ打鍵を
//!   二重に扱わないようにする（本番での注意。§ 報告書）
//! - 押されたものは `MenuEvent` で届く。iced の購読で数十 ms ごとに受け取る
//!
//! **組み立ては、アプリ（NSApplication）ができてからにする**（`init_for_nsapp`）。iced の最初の `update` で呼ぶ。

use muda::accelerator::Accelerator;
use muda::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu};

use crate::model::{self, Node};

/// アプリの名前のメニューへ移すもの
const APP_MENU_ITEMS: [&str; 3] = ["設定…", "終了", "このアプリについて…"];

/// 作った OS のメニュー（落とすと消えるので、アプリが持ち続ける）
pub struct NativeMenu {
    _menu: Menu,
}

impl NativeMenu {
    pub fn install() -> Result<NativeMenu, muda::Error> {
        let menu = Menu::new();

        let app = Submenu::new("mdview", true);
        app.append_items(&[
            &PredefinedMenuItem::about(
                Some("mdview について"),
                Some(AboutMetadata {
                    name: Some("mdview".into()),
                    ..AboutMetadata::default()
                }),
            ),
            &PredefinedMenuItem::separator(),
            &item("設定…", "Ctrl+,", true),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::services(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(Some("mdview を終了")),
        ])?;
        menu.append(&app)?;

        for heading in model::menus() {
            let submenu = Submenu::new(heading.label, true);
            fill(&submenu, &heading.items)?;
            menu.append(&submenu)?;
        }

        // macOS の作法の「ウインドウ」メニュー
        let window = Submenu::new("ウインドウ", true);
        window.append_items(&[
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::maximize(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::bring_all_to_front(None),
        ])?;
        menu.append(&window)?;

        menu.init_for_nsapp();
        window.set_as_windows_menu_for_nsapp();
        Ok(NativeMenu { _menu: menu })
    }
}

/// 押された項目の名前（届いていれば）
pub fn poll() -> Vec<String> {
    let mut out = Vec::new();
    while let Ok(event) = muda::MenuEvent::receiver().try_recv() {
        out.push(event.id().0.clone());
    }
    out
}

fn fill(submenu: &Submenu, nodes: &[Node]) -> Result<(), muda::Error> {
    for node in nodes {
        match node {
            Node::Action { label, .. } if APP_MENU_ITEMS.contains(label) => {}
            Node::Action {
                label,
                shortcut,
                enabled,
            } => submenu.append(&item(label, shortcut, *enabled))?,
            Node::Sub { label, children } => {
                let child = Submenu::new(*label, true);
                fill(&child, children)?;
                submenu.append(&child)?;
            }
            Node::Separator => submenu.append(&PredefinedMenuItem::separator())?,
        }
    }
    Ok(())
}

/// 項目。**名前を id にする**（押されたときに名前で受け取る）
fn item(label: &str, shortcut: &str, enabled: bool) -> MenuItem {
    MenuItem::with_id(label, label, enabled, accelerator(shortcut))
}

/// `Ctrl+Shift+S` を macOS の打鍵（⌘⇧S）へ読み替える。読めないものは付けない
fn accelerator(shortcut: &str) -> Option<Accelerator> {
    if shortcut.is_empty() || shortcut.starts_with("Alt+F4") {
        return None;
    }
    let mac = shortcut.replace("Ctrl", "CmdOrCtrl");
    mac.parse().ok()
}
