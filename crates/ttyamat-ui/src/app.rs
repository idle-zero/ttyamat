use iced::Fill;
use iced::{Element, Theme, widget::column};

use crate::terminal_view;
use crate::title_bar;

#[derive(Default)]
struct App {}

#[derive(Debug, Clone)]
enum Message {
    Event(iced::Event),
    TitleBar(title_bar::Message),
    Terminal(terminal_view::Message),
}

pub fn run() -> iced::Result {
    iced::application(App::default, update, view)
        .title("ttyamat")
        .theme(theme)
        .decorations(false)
        .window_size((1000, 700))
        .centered()
        .run()
}

fn theme(_: &App) -> Theme {
    Theme::Dark
}

fn update(_: &mut App, message: Message) {
    match message {
        Message::TitleBar(tb_message) => match tb_message {
            title_bar::Message::MinimizeWindow => println!("minimize press"),
            title_bar::Message::ToggleMaximize => println!("toggle maximize press"),
            title_bar::Message::CloseWindow => println!("close press"),
        },
        _ => println!("not implemented {:?}", message),
    }
}

fn view(_: &App) -> Element<'_, Message> {
    let title_bar = title_bar::view().map(Message::TitleBar);
    let terminal = terminal_view::view().map(Message::Terminal);

    column![title_bar, terminal].width(Fill).height(Fill).into()
}
