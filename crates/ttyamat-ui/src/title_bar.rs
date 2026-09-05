use iced::{Center, Fill};
use iced::{
    Element,
    widget::{button, row, text},
};

#[derive(Clone, Debug)]
pub(crate) enum Message {
    MinimizeWindow,
    ToggleMaximize,
    CloseWindow,
}

pub(crate) fn view() -> Element<'static, Message> {
    let minimize = button("-").on_press(Message::MinimizeWindow);
    let maximize = button("□").on_press(Message::ToggleMaximize);
    let close = button("x").on_press(Message::CloseWindow);

    row![text("ttyamat"), minimize, maximize, close,]
        .width(Fill)
        .height(40)
        .align_y(Center)
        .into()
}
