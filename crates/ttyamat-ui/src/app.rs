use iced::Fill;
use iced::{Element, Theme, widget::column};

use crate::tab::{Tab, TabId};
use crate::terminal_view;
use crate::title_bar;

struct App {
    tabs: Vec<Tab>,
    active_tab: TabId,
    hovered_tab: Option<TabId>,
}

impl App {
    fn new() -> Self {
        let tabs = vec![
            Tab {
                id: TabId(1),
                title: String::from("PowerShell"),
            },
            Tab {
                id: TabId(2),
                title: String::from("Command"),
            },
            Tab {
                id: TabId(3),
                title: String::from("Ubuntu"),
            },
            Tab {
                id: TabId(4),
                title: String::from("Rust"),
            },
            Tab {
                id: TabId(5),
                title: String::from("Server"),
            },
        ];

        Self {
            active_tab: tabs[0].id,
            tabs,
            hovered_tab: None,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Event(iced::Event),
    TitleBar(title_bar::Message),
    Terminal(terminal_view::Message),
}

pub fn run() -> iced::Result {
    iced::application(App::new, update, view)
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

fn update(app: &mut App, message: Message) {
    match message {
        Message::TitleBar(tb_message) => match tb_message {
            title_bar::Message::MinimizeWindow => println!("minimize press"),
            title_bar::Message::ToggleMaximize => println!("toggle maximize press"),
            title_bar::Message::CloseWindow => println!("close press"),
            title_bar::Message::TabPressed(id) => {
                if app.tabs.iter().any(|tab| tab.id == id) {
                    app.active_tab = id;
                }
            }
            title_bar::Message::TabHoverChanged { id, is_hovered } => {
                if is_hovered {
                    app.hovered_tab = Some(id);
                } else if app.hovered_tab == Some(id) {
                    app.hovered_tab = None;
                }
            }
            title_bar::Message::TabClosePressed(id) => println!("tab close pressed {:?}", id),
        },
        _ => println!("not implemented {:?}", message),
    }
}

fn view(app: &App) -> Element<'_, Message> {
    let title_bar =
        title_bar::view(&app.tabs, app.active_tab, app.hovered_tab).map(Message::TitleBar);
    let terminal = terminal_view::view().map(Message::Terminal);

    column![title_bar, terminal].width(Fill).height(Fill).into()
}
