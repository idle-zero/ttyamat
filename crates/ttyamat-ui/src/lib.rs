use iced::{Element, Theme, widget::text};

#[derive(Default)]
struct App;

#[derive(Debug, Clone)]
enum Message {}

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
    match message {}
}

fn view(_: &App) -> Element<'_, Message> {
    text("ttyamat terminal viewport").into()
}
