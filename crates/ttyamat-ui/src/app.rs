use iced::futures::StreamExt;
use iced::futures::channel::mpsc;
use iced::widget::{mouse_area, row, space, stack};
use iced::{Element, Theme, widget::column};
use iced::{Fill, Length, Subscription, Task, keyboard, mouse, window};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApplicationShortcut {
    New,
    CloseActive,
    SelectNext,
    SelectPrevious,
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
    window_focused: bool,
    window_size: Option<iced::Size>,
    scale_factor: f32,
    tabs: Vec<Tab>,
    active_tab: TabId,
    hovered_tab: Option<TabId>,
    next_tab_id: u64,
    terminal_event_sender: mpsc::UnboundedSender<SessionEvent>,
    terminal_size: TerminalSize,
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
            window_focused: false,
            window_size: None,
            scale_factor: 1.0,
            next_tab_id: 1,
            terminal_event_sender: sender,
            terminal_size: initial_terminal_size(),
        };
        app.create_tab();
        (app, terminal_event_task)
    }

    fn create_tab(&mut self) {
        let id = TabId(self.next_tab_id);
        let sender = self.terminal_event_sender.clone();
        let fallback_title = format!("Tab {}", id.0);
        let size = self.terminal_size;

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
            status,
        } => match event {
            iced::Event::Keyboard(keyboard_event) => {
                handle_keyboard_event(app, window_id, keyboard_event, status)
            }
            iced::Event::Window(window_event) => match window_event {
                window::Event::Opened { size, .. } => {
                    app.window_size = Some(size);
                    app.window_id = Some(window_id);
                    Task::none()
                }
                window::Event::Resized(size) => {
                    app.window_size = Some(size);
                    Task::none()
                }
                window::Event::Rescaled(scale_factor) => {
                    app.scale_factor = scale_factor;
                    Task::none()
                }
                window::Event::Focused => {
                    app.window_focused = true;
                    Task::none()
                }
                window::Event::Unfocused => {
                    app.window_focused = false;
                    Task::none()
                }
                _ => Task::none(),
            },
            _ => Task::none(),
        },
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

fn application_shortcut(event: &keyboard::Event) -> Option<ApplicationShortcut> {
    let keyboard::Event::KeyPressed {
        key,
        physical_key,
        modifiers,
        repeat,
        ..
    } = event
    else {
        return None;
    };

    if *repeat {
        return None;
    }

    let command_shift = keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT;
    let latin_key = key
        .to_latin(*physical_key)
        .map(|character| character.to_ascii_lowercase());

    if *modifiers == command_shift {
        match latin_key {
            Some('t') => return Some(ApplicationShortcut::New),
            Some('w') => return Some(ApplicationShortcut::CloseActive),
            _ => {}
        }
    }

    if !matches!(key, keyboard::Key::Named(keyboard::key::Named::Tab)) {
        return None;
    }

    if *modifiers == keyboard::Modifiers::CTRL {
        Some(ApplicationShortcut::SelectNext)
    } else if *modifiers == keyboard::Modifiers::CTRL | keyboard::Modifiers::SHIFT {
        Some(ApplicationShortcut::SelectPrevious)
    } else {
        None
    }
}

fn handle_keyboard_event(
    app: &mut App,
    event_window_id: window::Id,
    event: keyboard::Event,
    status: iced::event::Status,
) -> Task<Message> {
    if app.window_id != Some(event_window_id)
        || !app.window_focused
        || status == iced::event::Status::Captured
    {
        return Task::none();
    }

    let Some(shortcut) = application_shortcut(&event) else {
        return Task::none();
    };

    execute_application_shortcut(app, shortcut)
}

fn execute_application_shortcut(app: &mut App, shortcut: ApplicationShortcut) -> Task<Message> {
    match shortcut {
        ApplicationShortcut::New => {
            app.create_tab();
            Task::none()
        }
        ApplicationShortcut::CloseActive => close_tab(app, app.active_tab),
        ApplicationShortcut::SelectNext => {
            select_next_tab(app);
            Task::none()
        }
        ApplicationShortcut::SelectPrevious => {
            select_previous_tab(app);
            Task::none()
        }
    }
}

fn select_next_tab(app: &mut App) {
    let Some(active_index) = app.tabs.iter().position(|tab| tab.id == app.active_tab) else {
        return;
    };

    let next_index = (active_index + 1) % app.tabs.len();
    app.active_tab = app.tabs[next_index].id;
}

fn select_previous_tab(app: &mut App) {
    let Some(active_index) = app.tabs.iter().position(|tab| tab.id == app.active_tab) else {
        return;
    };

    let previous_index = if active_index == 0 {
        app.tabs.len() - 1
    } else {
        active_index - 1
    };
    app.active_tab = app.tabs[previous_index].id;
}
