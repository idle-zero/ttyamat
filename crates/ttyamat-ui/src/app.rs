use iced::widget::{mouse_area, row, space, stack};
use iced::{Element, Theme, widget::column};
use iced::{Fill, Length, Subscription, Task, mouse, task, window};

use crate::tab::{Tab, TabId};
use crate::terminal_view;
use crate::title_bar;

const RESIZE_BORDER: f32 = 6.0;

struct App {
    window_id: Option<window::Id>,
    tabs: Vec<Tab>,
    active_tab: TabId,
    hovered_tab: Option<TabId>,
    next_tab_id: u64,
}

impl App {
    fn new() -> Self {
        let new_tab = Tab {
            id: TabId(1),
            title: String::from("Command"),
        };

        Self {
            active_tab: new_tab.id,
            tabs: vec![new_tab],
            hovered_tab: None,
            window_id: None,
            next_tab_id: 2,
        }
    }

    fn create_tab(&mut self) {
        let id = TabId(self.next_tab_id);
        self.next_tab_id += 1;

        self.tabs.push(Tab {
            id,
            title: format!("Terminal {}", id.0),
        });

        self.active_tab = id;
        self.hovered_tab = None;
    }
}

#[derive(Debug, Clone)]
enum Message {
    Event {
        window_id: window::Id,
        event: iced::Event,
        status: iced::event::Status,
    },
    StartWindowResize(window::Direction),
    TitleBar(title_bar::Message),
    Terminal(terminal_view::Message),
}

pub fn run() -> iced::Result {
    iced::application(App::new, update, view)
        .title("ttyamat")
        .theme(theme)
        .subscription(subscription)
        .decorations(false)
        .resizable(true)
        .window_size((1000, 700))
        .centered()
        .run()
}

fn theme(_: &App) -> Theme {
    Theme::Dark
}

fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::TitleBar(tb_message) => match tb_message {
            title_bar::Message::MinimizeWindow => {
                let Some(window_id) = app.window_id else {
                    return Task::none();
                };
                window::minimize(window_id, true)
            }
            title_bar::Message::ToggleMaximize => {
                let Some(window_id) = app.window_id else {
                    return Task::none();
                };
                window::toggle_maximize(window_id)
            }
            title_bar::Message::CloseWindow => {
                let Some(window_id) = app.window_id else {
                    return Task::none();
                };
                window::close(window_id)
            }
            title_bar::Message::StartWindowDrag => {
                let Some(window_id) = app.window_id else {
                    return Task::none();
                };
                window::drag(window_id)
            }
            title_bar::Message::NewTabPressed => {
                app.create_tab();
                Task::none()
            }
            title_bar::Message::TabPressed(id) => {
                if app.tabs.iter().any(|tab| tab.id == id) {
                    app.active_tab = id;
                }
                Task::none()
            }
            title_bar::Message::TabHoverChanged { id, is_hovered } => {
                if is_hovered {
                    app.hovered_tab = Some(id);
                } else if app.hovered_tab == Some(id) {
                    app.hovered_tab = None;
                }
                Task::none()
            }
            title_bar::Message::TabClosePressed(id) => {
                println!("tab close pressed {:?}", id);
                Task::none()
            }
        },
        Message::Event {
            window_id,
            event,
            status: _,
        } => {
            if let iced::Event::Window(window::Event::Opened { .. }) = event {
                app.window_id = Some(window_id)
            }
            Task::none()
        }
        Message::StartWindowResize(direction) => {
            let Some(window_id) = app.window_id else {
                return Task::none();
            };
            window::drag_resize(window_id, direction)
        }
    }
}

fn subscription(_: &App) -> Subscription<Message> {
    iced::event::listen_with(|event, status, window_id| {
        Some(Message::Event {
            window_id,
            event,
            status,
        })
    })
}

fn view(app: &App) -> Element<'_, Message> {
    let title_bar =
        title_bar::view(&app.tabs, app.active_tab, app.hovered_tab).map(Message::TitleBar);
    let terminal = terminal_view::view().map(Message::Terminal);

    let content = column![title_bar, terminal].width(Fill).height(Fill);

    stack![content, resize_handles()]
        .width(Fill)
        .height(Fill)
        .into()
}

fn resize_handles() -> Element<'static, Message> {
    let border = Length::Fixed(RESIZE_BORDER);

    let north_west = resize_handle(
        window::Direction::NorthWest,
        border,
        border,
        mouse::Interaction::ResizingDiagonallyDown,
    );

    let north = resize_handle(
        window::Direction::North,
        Fill,
        border,
        mouse::Interaction::ResizingVertically,
    );

    let north_east = resize_handle(
        window::Direction::NorthEast,
        border,
        border,
        mouse::Interaction::ResizingDiagonallyUp,
    );

    let west = resize_handle(
        window::Direction::West,
        border,
        Fill,
        mouse::Interaction::ResizingHorizontally,
    );

    let east = resize_handle(
        window::Direction::East,
        border,
        Fill,
        mouse::Interaction::ResizingHorizontally,
    );

    let south_west = resize_handle(
        window::Direction::SouthWest,
        border,
        border,
        mouse::Interaction::ResizingDiagonallyUp,
    );

    let south = resize_handle(
        window::Direction::South,
        Fill,
        border,
        mouse::Interaction::ResizingVertically,
    );

    let south_east = resize_handle(
        window::Direction::SouthEast,
        border,
        border,
        mouse::Interaction::ResizingDiagonallyDown,
    );

    let top = row![north_west, north, north_east]
        .width(Fill)
        .height(border);

    let center = row![west, space::Space::new().width(Fill).height(Fill), east]
        .width(Fill)
        .height(Fill);

    let bottom = row![south_west, south, south_east]
        .width(Fill)
        .height(border);

    column![top, center, bottom].width(Fill).height(Fill).into()
}

fn resize_handle(
    direction: window::Direction,
    width: Length,
    height: Length,
    interaction: mouse::Interaction,
) -> Element<'static, Message> {
    mouse_area(space::Space::new().width(width).height(height))
        .on_press(Message::StartWindowResize(direction))
        .interaction(interaction)
        .into()
}
