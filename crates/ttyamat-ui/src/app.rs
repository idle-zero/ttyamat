use iced::futures::StreamExt;
use iced::futures::channel::mpsc;
use iced::widget::stack;
use iced::{Element, Theme, widget::column};
use iced::{Fill, Subscription, Task, keyboard, window};
use ttyamat_terminal::{TerminalEvent, TerminalSession, TerminalSize};

use crate::tab::{Tab, TabId};
use crate::title_bar;
use crate::window::{Change, Command, WindowState};
use crate::{shortcuts, terminal_input, terminal_view, window_chrome};

const INITIAL_TERMINAL_COLUMNS: u16 = 80;
const INITIAL_TERMINAL_LINES: u16 = 24;

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
    Window {
        id: iced::window::Id,
        event: iced::window::Event,
    },
    Keyboard {
        window_id: iced::window::Id,
        event: iced::keyboard::Event,
        status: iced::event::Status,
    },
    WindowCommand(Command),
    TitleBar(title_bar::Message),
    Terminal(terminal_view::Message),
    TerminalEvent(SessionEvent),
}

struct App {
    window: WindowState,
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
            window: WindowState::new(),
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
        let terminal_frame = terminal_session.initial_frame();

        self.next_tab_id += 1;
        self.tabs.push(Tab {
            id,
            title: fallback_title.clone(),
            fallback_title,
            session: terminal_session,
            terminal_frame,
            terminal_frame_dirty: false,
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
        drop(self.tabs.remove(index));

        if self.hovered_tab == Some(id) {
            self.hovered_tab = None;
        }

        if was_active {
            let replacement_idx = index.min(self.tabs.len() - 1);
            self.active_tab = self.tabs[replacement_idx].id;
        }
        CloseTabOutcome::Closed
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Window { id, event } => self.handle_window_event(id, event),
            Message::Keyboard {
                window_id,
                event,
                status,
            } => self.handle_keyboard_event(window_id, event, status),
            Message::WindowCommand(command) => self.window.command(command),
            Message::TitleBar(message) => self.handle_title_bar(message),
            Message::Terminal(message) => match message {},
            Message::TerminalEvent(event) => handle_terminal_event(self, event.tab_id, event.event),
        }
    }

    fn handle_window_event(&mut self, id: window::Id, event: window::Event) -> Task<Message> {
        match self.window.handle_event(id, event) {
            Change::GeometryChanged => resize_terminal_sessions(self),
            Change::None => Task::none(),
        }
    }

    fn handle_title_bar(&mut self, message: title_bar::Message) -> Task<Message> {
        match message {
            title_bar::Message::MinimizeWindow => self.window.command(Command::Minimize),
            title_bar::Message::ToggleMaximize => self.window.command(Command::ToggleMaximize),
            title_bar::Message::CloseWindow => self.window.command(Command::Close),
            title_bar::Message::StartWindowDrag => self.window.command(Command::Drag),
            title_bar::Message::NewTabPressed => {
                self.create_tab();
                Task::none()
            }
            title_bar::Message::TabClosePressed(id) => close_tab(self, id),
            title_bar::Message::TabPressed(id) => {
                if self.tabs.iter().any(|tab| tab.id == id) {
                    self.active_tab = id;
                    refresh_tab_terminal_frame(self, id);
                }
                Task::none()
            }
            title_bar::Message::TabHoverChanged { id, is_hovered } => {
                if is_hovered {
                    self.hovered_tab = Some(id);
                } else if self.hovered_tab == Some(id) {
                    self.hovered_tab = None;
                }
                Task::none()
            }
        }
    }

    fn handle_keyboard_event(
        &mut self,
        window_id: window::Id,
        event: keyboard::Event,
        status: iced::event::Status,
    ) -> Task<Message> {
        if !self.window.accepts_keyboard(window_id, status) {
            return Task::none();
        }

        match shortcuts::resolve(&event) {
            shortcuts::Resolution::Execute(action) => execute_shortcut(self, action),
            shortcuts::Resolution::Suppress => Task::none(),
            shortcuts::Resolution::Unhandled => match terminal_input::encode(&event) {
                Some(bytes) => write_to_active_terminal(self, bytes),
                None => Task::none(),
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let title_bar =
            title_bar::view(&self.tabs, self.active_tab, self.hovered_tab).map(Message::TitleBar);
        let active_frame = self
            .tabs
            .iter()
            .find(|tab| tab.id == self.active_tab)
            .map(|tab| &tab.terminal_frame);
        let terminal = terminal_view::view(active_frame).map(Message::Terminal);

        let content = column![title_bar, terminal].width(Fill).height(Fill);

        stack![
            content,
            window_chrome::resize_handles().map(Message::WindowCommand)
        ]
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::event::listen_with(|event, status, window_id| match event {
            iced::Event::Window(event) => Some(Message::Window {
                id: window_id,
                event,
            }),
            iced::Event::Keyboard(event) => Some(Message::Keyboard {
                window_id,
                event,
                status,
            }),
            _ => None,
        })
    }
}

pub fn run() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("ttyamat")
        .theme(App::theme)
        .subscription(App::subscription)
        .decorations(false)
        .resizable(true)
        .window_size((1000, 700))
        .centered()
        .run()
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
        TerminalEvent::Wakeup => {
            if let Some(tab) = app.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                tab.terminal_frame_dirty = true;
            }

            if app.active_tab == tab_id {
                refresh_tab_terminal_frame(app, tab_id);
            }
            Task::none()
        }
    }
}

fn close_tab(app: &mut App, tab_id: TabId) -> Task<Message> {
    if app.close_tab(tab_id) != CloseTabOutcome::CloseWindow {
        return Task::none();
    }

    app.window.command(Command::Close)
}

fn initial_terminal_size() -> TerminalSize {
    TerminalSize::new(
        INITIAL_TERMINAL_COLUMNS,
        INITIAL_TERMINAL_LINES,
        terminal_view::CELL_WIDTH as u16,
        terminal_view::CELL_HEIGHT as u16,
    )
    .expect("initial terminal dimensions must be non-zero")
}

fn write_to_active_terminal(app: &mut App, bytes: Vec<u8>) -> Task<Message> {
    let active_tab = app.active_tab;
    let write_result = app
        .tabs
        .iter()
        .find(|tab| tab.id == active_tab)
        .map(|tab| tab.session.write(bytes));

    match write_result {
        Some(Ok(())) | None => Task::none(),
        Some(Err(_error)) => close_tab(app, active_tab),
    }
}

fn execute_shortcut(app: &mut App, shortcut: shortcuts::Action) -> Task<Message> {
    match shortcut {
        shortcuts::Action::OpenTab => {
            app.create_tab();
            Task::none()
        }
        shortcuts::Action::CloseActiveTab => close_tab(app, app.active_tab),
        shortcuts::Action::NextTab => {
            select_next_tab(app);
            Task::none()
        }
        shortcuts::Action::PreviousTab => {
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
    refresh_tab_terminal_frame(app, app.active_tab);
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
    refresh_tab_terminal_frame(app, app.active_tab);
}

fn refresh_tab_terminal_frame(app: &mut App, tab_id: TabId) {
    let Some(tab) = app.tabs.iter_mut().find(|tab| tab.id == tab_id) else {
        return;
    };

    if !tab.terminal_frame_dirty {
        return;
    }

    tab.session.acknowledge_wakeup();
    if let Some(update) = tab.session.take_render_update() {
        tab.terminal_frame.apply(update);
    }
    tab.terminal_frame_dirty = false;
}

fn resize_terminal_sessions(app: &mut App) -> Task<Message> {
    let Some(window_size) = app.window.size() else {
        return Task::none();
    };
    let viewport = window_chrome::terminal_viewport(window_size);
    let Some(size) = terminal_view::size_for_viewport(viewport, app.window.scale_factor()) else {
        return Task::none();
    };

    if size == app.terminal_size {
        return Task::none();
    }

    app.terminal_size = size;
    let mut failed_tabs = Vec::new();

    for tab in &mut app.tabs {
        if tab.session.resize(size).is_err() {
            failed_tabs.push(tab.id);
        }
        tab.terminal_frame_dirty = true;
    }

    refresh_tab_terminal_frame(app, app.active_tab);

    Task::batch(failed_tabs.into_iter().map(|tab_id| close_tab(app, tab_id)))
}
