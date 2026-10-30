//! メニューの中身（自前で描くメニューと、macOS の OS のメニューで共有する）。
//!
//! 本体のファイルメニューは 25 項目ほどが 1 列に並ぶ（本体 `src/app/menu.rs`）。ここでは、
//! **似た操作を下の階層にまとめた案**を置く（開く・比べる・出力）。2 段目の下の階層（変換 ▸ 文字の種類 ▸）も置き、
//! 深さが 2 段でも開けることを見る。

/// メニューの 1 項目
#[derive(Debug, Clone)]
pub enum Node {
    /// 押すと動く。`shortcut` は Windows・Linux の書き方（macOS では ⌘ などに直す）
    Action {
        label: &'static str,
        shortcut: &'static str,
        enabled: bool,
    },
    /// 下の階層（横へ開く）
    Sub {
        label: &'static str,
        children: Vec<Node>,
    },
    Separator,
}

/// 見出し（メニューバーに並ぶもの）
#[derive(Debug, Clone)]
pub struct Heading {
    pub label: &'static str,
    /// `Alt` と合わせて押す文字（Windows の作法）
    pub access: char,
    pub items: Vec<Node>,
}

fn act(label: &'static str, shortcut: &'static str) -> Node {
    Node::Action {
        label,
        shortcut,
        enabled: true,
    }
}

fn off(label: &'static str) -> Node {
    Node::Action {
        label,
        shortcut: "",
        enabled: false,
    }
}

fn sub(label: &'static str, children: Vec<Node>) -> Node {
    Node::Sub { label, children }
}

const SEP: Node = Node::Separator;

/// 本体のメニューを、似た操作でまとめ直したもの
pub fn menus() -> Vec<Heading> {
    vec![
        Heading {
            label: "ファイル",
            access: 'F',
            items: vec![
                act("新規", "Ctrl+N"),
                act("新しいウィンドウ", "Ctrl+Shift+N"),
                SEP,
                sub(
                    "開く",
                    vec![
                        act("開く…", "Ctrl+O"),
                        act("新しいウィンドウで開く…", "Ctrl+Alt+O"),
                        act("URL を開く…", ""),
                        act("アプリ内で開く…", ""),
                    ],
                ),
                sub(
                    "最近使ったファイル",
                    vec![
                        act("README.md", ""),
                        act("設計メモ.md", ""),
                        act("議事録 2026-10-27.md", ""),
                        SEP,
                        act("一覧を消す", ""),
                    ],
                ),
                sub(
                    "比べる",
                    vec![
                        act("ファイルを比べる…", ""),
                        act("アプリ内で選んで比べる…", ""),
                        act("保存してある元と比べる", ""),
                        SEP,
                        act("フォルダーを比べる…", ""),
                        act("3 つのファイルをマージする…", ""),
                        act("パッチを当てる…", ""),
                    ],
                ),
                SEP,
                act("上書き保存", "Ctrl+S"),
                act("名前を付けて保存…", "Ctrl+Shift+S"),
                act("文字コード・改行コード…", ""),
                SEP,
                sub(
                    "出力",
                    vec![
                        act("PDF に出力…", ""),
                        act("HTML に出力…", ""),
                        act("PPTX に出力（スライド）…", ""),
                        act("スライドを書き出す…", ""),
                    ],
                ),
                SEP,
                act("設定…", "Ctrl+,"),
                act("終了", "Alt+F4"),
            ],
        },
        Heading {
            label: "編集",
            access: 'E',
            items: vec![
                act("元に戻す", "Ctrl+Z"),
                off("やり直し"),
                SEP,
                act("切り取り", "Ctrl+X"),
                act("コピー", "Ctrl+C"),
                act("貼り付け", "Ctrl+V"),
                act("すべて選択", "Ctrl+A"),
                SEP,
                sub(
                    "書式",
                    vec![
                        act("太字", "Ctrl+B"),
                        act("斜体", "Ctrl+I"),
                        act("インラインコード", "Ctrl+E"),
                        act("リンク", "Ctrl+K"),
                    ],
                ),
                sub(
                    "表",
                    vec![
                        act("表を整形", "Shift+Alt+F"),
                        act("行を足す", ""),
                        act("列を足す", ""),
                        SEP,
                        act("表のセルをつなぐ", ""),
                        act("表のセルを分ける", ""),
                    ],
                ),
                sub(
                    "選んだ範囲を変換",
                    vec![
                        act("大文字にする", ""),
                        act("小文字にする", ""),
                        sub(
                            "文字の種類",
                            vec![
                                act("半角にする", ""),
                                act("全角にする", ""),
                                act("ひらがなにする", ""),
                                act("カタカナにする", ""),
                            ],
                        ),
                        act("タブを空白にする", ""),
                    ],
                ),
                SEP,
                act("検索・置換…", "Ctrl+F"),
                act("行コメントの切替", "Ctrl+/"),
                act("ブロックコメントの切替", "Shift+Alt+A"),
            ],
        },
        Heading {
            label: "表示",
            access: 'V',
            items: vec![
                act("編集", "Ctrl+1"),
                act("プレビュー", "Ctrl+2"),
                act("分割", "Ctrl+3"),
                SEP,
                act("拡大", "Ctrl++"),
                act("縮小", "Ctrl+-"),
                act("等倍に戻す", "Ctrl+0"),
                SEP,
                act("集中モード", "F11"),
            ],
        },
        Heading {
            label: "ヘルプ",
            access: 'H',
            items: vec![act("このアプリについて…", "")],
        },
    ]
}

/// `path`（下の階層の番号の並び）で辿った項目の並び
#[cfg_attr(not(test), allow(dead_code))]
pub fn children_at<'a>(items: &'a [Node], path: &[usize]) -> Option<&'a [Node]> {
    let mut list = items;
    for index in path {
        match list.get(*index)? {
            Node::Sub { children, .. } => list = children,
            _ => return None,
        }
    }
    Some(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ファイルメニューの 1 段目は本体（約 25 項目）より短い。下の階層をまとめたため
    #[test]
    fn grouping_shortens_the_file_menu() {
        let file = &menus()[0];
        let rows = file
            .items
            .iter()
            .filter(|node| !matches!(node, Node::Separator))
            .count();
        assert!(rows <= 12, "{rows}");
    }

    /// 2 段の下の階層まで辿れる
    #[test]
    fn nested_submenus_are_reachable() {
        let edit = &menus()[1].items;
        let transform = edit
            .iter()
            .position(|node| matches!(node, Node::Sub { label: "選んだ範囲を変換", .. }))
            .unwrap();
        let kinds = children_at(edit, &[transform, 2]).expect("文字の種類");
        assert_eq!(kinds.len(), 4);
    }
}
