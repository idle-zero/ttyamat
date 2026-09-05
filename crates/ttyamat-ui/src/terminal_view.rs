use iced::Fill;
use iced::{
    Element,
    widget::{container, text},
};

#[derive(Clone, Debug)]
pub(crate) enum Message {}

pub(crate) fn view() -> Element<'static, Message> {
    container(text("ttyamat terminal viewport"))
        .width(Fill)
        .height(Fill)
        .into()
}
