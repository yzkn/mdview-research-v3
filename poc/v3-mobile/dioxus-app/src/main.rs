//! 閲覧中心＋軽い編集のモバイル版の試作（v3 の実現可能性検証）。
//!
//! - 描くのは本体の HTML の書き出し（`export::html`）。図（Mermaid）は SVG、数式も書き出しの中身のまま
//! - 編集は原文の `<textarea>`。**日本語の入力は端末の WebView と IME に任せる**（自前の編集器を持ち込まない）
//! - 幅が広い（タブレットの横置き）ときは原文とプレビューを並べ、狭いときは切り替える
use std::time::Instant;

use dioxus::prelude::*;
use mdview_core::document::Document;
use mdview_core::export::{html, Silent};

/// 見本の文書（本体の samples/sample.md）
const SAMPLE: &str = include_str!("../../../../samples/sample.md");

/// 画面の作り。端末の幅で並べ方を変える（900px 以上は 2 面）
const CSS: &str = r#"
* { box-sizing: border-box; }
html, body { margin: 0; height: 100%; }
body { font-family: "Noto Sans CJK JP", "Hiragino Sans", "Yu Gothic UI", sans-serif; background: #fff; color: #1b1b1b; }
#shell { display: flex; flex-direction: column; height: 100vh; height: 100dvh; }
header { display: flex; gap: 8px; align-items: center; padding: 8px 12px; padding-top: max(8px, env(safe-area-inset-top));
         border-bottom: 1px solid #ddd; background: #f6f6f6; }
header .title { font-weight: bold; flex: 1; }
header button { min-height: 44px; min-width: 64px; font-size: 16px; border-radius: 8px; border: 1px solid #bbb; background: #fff; }
header button.on { background: #1f6feb; color: #fff; border-color: #1f6feb; }
#panes { flex: 1; display: flex; min-height: 0; }
#panes > section { flex: 1; min-width: 0; overflow: auto; -webkit-overflow-scrolling: touch; }
textarea { width: 100%; height: 100%; border: 0; padding: 12px; font: 16px/1.6 ui-monospace, "Noto Sans Mono CJK JP", monospace; resize: none; }
.preview main { max-width: none; padding: 12px 16px; padding-bottom: max(12px, env(safe-area-inset-bottom)); }
.preview img, .preview svg { max-width: 100%; height: auto; }
.preview pre { overflow-x: auto; }
footer { font-size: 12px; color: #666; padding: 4px 12px; border-top: 1px solid #eee; }
@media (max-width: 899px) { #panes.view .source { display: none; } #panes.edit .preview { display: none; } }
@media (min-width: 900px) { #panes .source { border-right: 1px solid #ddd; } header .mode { display: none; } }
"#;

/// 書き出しの HTML から、埋め込み用に CSS と本文だけを取る
fn render(text: &str) -> (String, String, f64) {
    let started = Instant::now();
    let document = Document::from_text(text.to_owned());
    let out = match html::export(&document, "mdview", None, &Silent) {
        Ok(out) => out.text,
        Err(_) => return (String::new(), "<p>描けませんでした</p>".into(), 0.0),
    };
    let between = |open: &str, close: &str| {
        let start = out.find(open).map(|at| at + open.len()).unwrap_or(0);
        let end = out[start..].find(close).map(|at| start + at).unwrap_or(out.len());
        out[start..end].to_owned()
    };
    let style = between("<style>", "</style>");
    let body = format!("<main>{}</main>", between("<main>", "</main>"));
    (style, body, started.elapsed().as_secs_f64() * 1000.0)
}

#[component]
fn App() -> Element {
    let mut text = use_signal(|| SAMPLE.to_owned());
    let mut editing = use_signal(|| false);
    let rendered = use_memo(move || render(&text.read()));
    let (style, body, millis) = rendered();
    let panes = if editing() { "edit" } else { "view" };
    rsx! {
        style { {CSS} }
        style { {style} }
        div { id: "shell",
            header {
                span { class: "title", "mdview（v3 試作）" }
                button { class: if !editing() { "mode on" } else { "mode" }, onclick: move |_| editing.set(false), "閲覧" }
                button { class: if editing() { "mode on" } else { "mode" }, onclick: move |_| editing.set(true), "編集" }
            }
            div { id: "panes", class: "{panes}",
                section { class: "source",
                    textarea { value: "{text}", spellcheck: "false", oninput: move |event| text.set(event.value()) }
                }
                section { class: "preview", dangerous_inner_html: "{body}" }
            }
            footer { "{text.read().len()} バイト・描画 {millis:.1} ms" }
        }
    }
}

fn main() {
    dioxus::launch(App);
}
