//! iced の窓と描画の部品（iced_winit・iced_wgpu）を、Android 向けに組めるかだけを見る。
//! 窓を起こす入口（`iced::application`）まで型が通れば、組めると判断する

fn view(_: &()) -> iced::Element<'_, ()> {
    iced::widget::text("mdview").into()
}

pub fn run() -> iced::Result {
    iced::application(|| (), |_: &mut (), _: ()| {}, view).run()
}
