//! OS に寄せた見た目（寸法・字体・強調色・ボタンの並び）。
//!
//! iced は部品を自分で描くので、OS の部品そのものにはならない。**寸法と色と並びを OS の手引きに合わせる**ことで寄せる。
//! 値は各 OS の公開の手引き（Windows 11 の WinUI 3・macOS の Human Interface Guidelines・GNOME の Adwaita）の
//! 代表的な値を手で写したもの（試作の値。本番では見比べて詰める）。

use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color, Font, Shadow, Theme, Vector};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Windows,
    Mac,
    Gnome,
}

impl Os {
    pub const ALL: [Os; 3] = [Os::Windows, Os::Mac, Os::Gnome];

    /// 動いている OS
    pub fn current() -> Os {
        if cfg!(target_os = "macos") {
            Os::Mac
        } else if cfg!(windows) {
            Os::Windows
        } else {
            Os::Gnome
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Os::Windows => "Windows 11",
            Os::Mac => "macOS",
            Os::Gnome => "GNOME",
        }
    }
}

impl std::fmt::Display for Os {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// 見た目の寸法と色
#[derive(Debug, Clone, Copy)]
pub struct Look {
    pub os: Os,
    /// UI の字体（OS の UI の字体。日本語の字形が出るもの）
    pub font: Font,
    pub font_size: f32,
    /// ボタンの高さ
    pub control_height: f32,
    pub control_radius: f32,
    pub panel_radius: f32,
    /// メニューの 1 行の高さ
    pub menu_row: f32,
    pub accent: Color,
    /// ダイアログのボタンを、主な操作が右端に来る順で並べるか（macOS・GNOME）
    pub primary_last: bool,
}

impl Look {
    pub fn of(os: Os) -> Look {
        match os {
            // WinUI 3: 本文 14px、ボタン 32px、角 4px、パネル 8px、メニューの行 32px 前後
            Os::Windows => Look {
                os,
                font: Font::with_name("Yu Gothic UI"),
                font_size: 14.0,
                control_height: 32.0,
                control_radius: 4.0,
                panel_radius: 8.0,
                menu_row: 30.0,
                accent: accent_color().unwrap_or(Color::from_rgb8(0x00, 0x5f, 0xb8)),
                primary_last: false,
            },
            // HIG: 本文 13pt、押しボタン 22pt 前後（ここでは押しやすさで 24）、角 6、メニューの行 22pt
            Os::Mac => Look {
                os,
                font: Font::with_name("Hiragino Sans"),
                font_size: 13.0,
                control_height: 24.0,
                control_radius: 6.0,
                panel_radius: 10.0,
                menu_row: 24.0,
                accent: Color::from_rgb8(0x00, 0x7a, 0xff),
                primary_last: true,
            },
            // Adwaita: 本文 15px 相当（ここでは 14）、ボタン 34px、角 6px、ポップオーバー 12px
            Os::Gnome => Look {
                os,
                font: Font::with_name("Noto Sans CJK JP"),
                font_size: 14.0,
                control_height: 34.0,
                control_radius: 6.0,
                panel_radius: 12.0,
                menu_row: 32.0,
                accent: Color::from_rgb8(0x35, 0x84, 0xe4),
                primary_last: true,
            },
        }
    }

    /// ボタンの上下の余白（字の大きさから、高さがそろうように決める）
    pub fn button_padding(&self) -> [f32; 2] {
        [((self.control_height - self.font_size * 1.3) / 2.0).max(2.0), 12.0]
    }

    pub fn panel(&self, theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();
        let light = !palette.is_dark;
        container::Style {
            background: Some(Background::Color(palette.background.base.color)),
            text_color: Some(palette.background.base.text),
            border: Border {
                radius: self.panel_radius.into(),
                width: 1.0,
                color: Color::from_rgba(0.0, 0.0, 0.0, if light { 0.12 } else { 0.4 }),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, if light { 0.16 } else { 0.5 }),
                offset: Vector::new(0.0, if self.os == Os::Mac { 10.0 } else { 6.0 }),
                blur_radius: if self.os == Os::Mac { 30.0 } else { 16.0 },
            },
            ..container::Style::default()
        }
    }

    /// 標準のボタン
    pub fn button(&self, theme: &Theme, status: button::Status) -> button::Style {
        let palette = theme.extended_palette();
        let base = palette.background.base.color;
        let rest = match self.os {
            // WinUI の標準ボタンは地より少し明るい面に細い縁
            Os::Windows => mix(base, Color::WHITE, if palette.is_dark { 0.08 } else { 0.7 }),
            // macOS の押しボタンは白い面に影
            Os::Mac => mix(base, Color::WHITE, if palette.is_dark { 0.18 } else { 1.0 }),
            // Adwaita は地より少し濃い面、縁なし
            Os::Gnome => {
                if palette.is_dark {
                    mix(base, Color::WHITE, 0.1)
                } else {
                    mix(base, Color::BLACK, 0.08)
                }
            }
        };
        let face = match status {
            button::Status::Hovered => mix(rest, palette.background.base.text, 0.05),
            button::Status::Pressed => mix(rest, palette.background.base.text, 0.12),
            _ => rest,
        };
        button::Style {
            background: Some(Background::Color(face)),
            text_color: if status == button::Status::Disabled {
                Color { a: 0.4, ..palette.background.base.text }
            } else {
                palette.background.base.text
            },
            border: Border {
                radius: self.control_radius.into(),
                width: if self.os == Os::Gnome { 0.0 } else { 1.0 },
                color: Color::from_rgba(0.0, 0.0, 0.0, if palette.is_dark { 0.3 } else { 0.14 }),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, if self.os == Os::Mac { 0.15 } else { 0.0 }),
                offset: Vector::new(0.0, 0.5),
                blur_radius: 1.0,
            },
            snap: true,
        }
    }

    /// 主な操作のボタン（強調色で塗る）
    pub fn primary(&self, _theme: &Theme, status: button::Status) -> button::Style {
        let face = match status {
            button::Status::Hovered => mix(self.accent, Color::WHITE, 0.1),
            button::Status::Pressed => mix(self.accent, Color::BLACK, 0.15),
            _ => self.accent,
        };
        button::Style {
            background: Some(Background::Color(face)),
            text_color: Color::WHITE,
            border: Border {
                radius: self.control_radius.into(),
                ..Border::default()
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }

    /// メニューの行（ホバーで強調色。macOS は強調色で塗り、Windows・GNOME は薄い地）
    pub fn menu_row(&self, theme: &Theme, status: button::Status, open: bool) -> button::Style {
        let palette = theme.extended_palette();
        let hot = open || matches!(status, button::Status::Hovered | button::Status::Pressed);
        let (background, text) = match (self.os, hot) {
            (_, false) => (Color::TRANSPARENT, palette.background.base.text),
            (Os::Mac, true) => (self.accent, Color::WHITE),
            (_, true) => (
                mix(palette.background.base.color, palette.background.base.text, 0.08),
                palette.background.base.text,
            ),
        };
        button::Style {
            background: Some(Background::Color(background)),
            text_color: if status == button::Status::Disabled {
                Color { a: 0.35, ..palette.background.base.text }
            } else {
                text
            },
            border: Border {
                radius: (self.control_radius).into(),
                ..Border::default()
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }

    pub fn input(&self, theme: &Theme, status: text_input::Status) -> text_input::Style {
        let mut style = text_input::default(theme, status);
        style.border.radius = self.control_radius.into();
        // Windows の入力欄は、焦点があると下端だけ強調色になる。ここでは縁全体で代える（iced は縁の 1 辺だけを塗れない）
        if matches!(status, text_input::Status::Focused { .. }) {
            style.border.color = self.accent;
            style.border.width = if self.os == Os::Mac { 3.0 } else { 1.5 };
        }
        style
    }
}

/// `a` と `b` を `t` の割合で混ぜる
fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::from_rgba(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a,
    )
}

/// OS の強調色。**Windows だけ取る**（DWM の色。macOS・GNOME は別の口が要る。§ 報告書）
#[cfg(windows)]
fn accent_color() -> Option<Color> {
    use windows_sys::Win32::Graphics::Dwm::DwmGetColorizationColor;
    let mut argb: u32 = 0;
    let mut opaque: i32 = 0;
    // SAFETY: 2 つの出力先は生きている局所の変数
    let result = unsafe { DwmGetColorizationColor(&mut argb, &mut opaque) };
    if result != 0 {
        return None;
    }
    Some(Color::from_rgb8((argb >> 16) as u8, (argb >> 8) as u8, argb as u8))
}

#[cfg(not(windows))]
fn accent_color() -> Option<Color> {
    None
}
