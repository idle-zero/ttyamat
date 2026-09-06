use iced::futures::StreamExt;
use iced::futures::channel::mpsc;
use iced::widget::{mouse_area, row, space, stack};
use iced::{Element, Theme, widget::column};
use iced::{Fill, Length, Subscription, Task, mouse, window};
use ttyamat_terminal::{TerminalEvent, TerminalSession, TerminalSize};

use crate::tab::{Tab, TabId};
use crate::terminal_view;
use crate::title_bar;

const RESIZE_BORDER: f32 = 6.0;
const INITIAL_TERMINAL_COLUMNS: u16 = 80;
const INITIAL_TERMINAL_LINES: u16 = 24;
const INITIAL_CELL_WIDTH: u16 = 8;
const INITIAL_CELL_HEIGHT: u16 = 16;

#[derive(Debug, Clone)]
struct SessionEvent {
    tab_id: TabId,
    event: TerminalEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CloseTabOutcome {
    Closed,
    CloseWindow,
    NotFound,
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
    TerminalEvent(SessionEvent),
}

struct App {
    window_id: Option<window::Id>,
    tabs: Vec<Tab>,
    active_tab: TabId,
    hovered_tab: Option<TabId>,
    next_tab_id: u64,
    terminal_event_sender: mpsc::UnboundedSender<SessionEvent>,
}
impl App {
    fn new() -> (Self, Task<Message>) {
        let (sender, receiver) = mpsc::unbounded();
        let terminal_event_task = Task::stream(receiver.map(Message::TerminalEvent));
        let first_tab_id = TabId(1);
        let mut app = Self {
            active_tab: first_tab_id,
            tabs: Vec::new(),
            hovered_tab: None,
            window_id: None,
            next_tab_id: 1,
            terminal_event_sender: sender,
        };
        app.create_tab();
        (app, terminal_event_task)
    }

    fn create_tab(&mut self) {
        let id = TabId(self.next_tab_id);
        let sender = self.terminal_event_sender.clone();
        let fallback_title = format!("Tab {}", id.0);
        let size = initial_terminal_size();

        // TODO(error-reporting): Replace `expect` with graceful session-startup handling:
        // log `TerminalSessionError`, notify the user, and do not insert or activate a failed tab.
        let terminal_session = TerminalSession::spawn_default(size, move |event| {
            let _ = sender.unbounded_send(SessionEvent { tab_id: id, event });
        })
        .expect("failed to start terminal session");

        self.next_tab_id += 1;
        self.tabs.push(Tab {
            id,
            title: fallback_title.clone(),
            fallback_title,
            session: terminal_session,
        });

        self.active_tab = id;
        self.hovered_tab = None;
    }

    fn close_tab(&mut self, id: TabId) -> CloseTabOutcome {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == id) else {
            return CloseTabOutcome::NotFound;
        };

        if self.tabs.len() == 1 {
            return CloseTabOutcome::CloseWindow;
        }

        let was_active = self.active_tab == id;
        let closed_tab = self.tabs.remove(index);
        drop(closed_tab.session);

        if self.hovered_tab == Some(id) {
            self.hovered_tab = None;
        }

        if was_active {
            let replacement_idx = index.min(self.tabs.len() - 1);
            self.active_tab = self.tabs[replacement_idx].id
        }
        CloseTabOutcome::Closed
    }
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
            title_bar::Message::TabClosePressed(id) => close_tab(app, id),
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
        Message::TerminalEvent(session_event) => {
            handle_terminal_event(app, session_event.tab_id, session_event.event)
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

fn handle_terminal_event(app: &mut App, tab_id: TabId, event: TerminalEvent) -> Task<Message> {
    match event {
        TerminalEvent::Bell => {
            // TODO(notification): Surface terminal bells without blocking the UI thread.
            Task::none()
        }
        TerminalEvent::ChildExited(status) => {
            // TODO(error-reporting): Log the child exit status and notify the user when appropriate.
            let _ = status;
            close_tab(app, tab_id)
        }
        TerminalEvent::ExitRequested => close_tab(app, tab_id),
        TerminalEvent::TitleChanged(opt_title) => {
            let Some(tab) = app.tabs.iter_mut().find(|tab| tab.id == tab_id) else {
                return Task::none();
            };

            tab.title = opt_title.unwrap_or_else(|| tab.fallback_title.clone());
            Task::none()
        }
        TerminalEvent::Wakeup => Task::none(),
    }
}

fn close_tab(app: &mut App, tab_id: TabId) -> Task<Message> {
    if app.close_tab(tab_id) != CloseTabOutcome::CloseWindow {
        return Task::none();
    }

    let Some(window_id) = app.window_id else {
        return Task::none();
    };

    window::close(window_id)
}

fn initial_terminal_size() -> TerminalSize {
    TerminalSize::new(
        INITIAL_TERMINAL_COLUMNS,
        INITIAL_TERMINAL_LINES,
        INITIAL_CELL_WIDTH,
        INITIAL_CELL_HEIGHT,
    )
    .expect("initial terminal dimensions must be non-zero")
}
