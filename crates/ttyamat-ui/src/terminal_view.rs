use iced::{
    Element,
    widget::{container, text},
};
use iced::{Fill, Font, Theme};

use crate::style;

#[derive(Clone, Debug)]
pub(crate) enum Message {}

pub(crate) fn view() -> Element<'static, Message> {
    let placeholder = text("terminal").font(Font::MONOSPACE).size(18);

    container(placeholder)
        .width(Fill)
        .height(Fill)
        .padding([8, 10])
        .style(terminal_style)
        .into()
}

fn terminal_style(_: &Theme) -> container::Style {
    container::Style::default()
        .background(style::TERMINAL_BACKGROUND)
        .color(style::SECONDARY_TEXT)
}
