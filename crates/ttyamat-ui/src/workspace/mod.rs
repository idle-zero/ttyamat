use iced::{Element, Task, keyboard};
use ttyamat_terminal::{TerminalSessionError, TerminalSize};

use crate::shortcuts;
use crate::terminal::{
    self, TerminalPane,
    metrics::{self, CellMetrics},
};
use crate::window::Command;

use self::session_launcher::{SessionEvent, SessionLauncher};
use self::tab::{Tab, TabId};
use self::tabs::Tabs;

mod session_launcher;
mod tab;
mod tabs;
pub(crate) mod title_bar;
mod view;

pub(crate) struct Workspace {
    tabs: Tabs,
    launcher: SessionLauncher,
    metrics: CellMetrics,
    viewport: Option<Viewport>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Viewport {
    pub(crate) size: iced::Size,
    pub(crate) scale_factor: f32,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    TitleBar(title_bar::Message),
    Keyboard(keyboard::Event),
    SessionEvent(SessionEvent),
    Terminal(TabId, terminal::Message),
    ViewportChanged(Viewport),
}

pub(crate) enum Action {
    Window(Command),
}

pub(crate) struct Update {
    pub(crate) task: Task<Message>,
    pub(crate) action: Option<Action>,
}

impl Update {
    fn none() -> Self {
        Self {
            task: Task::none(),
            action: None,
        }
    }

    fn window(command: Command) -> Self {
        Self {
            task: Task::none(),
            action: Some(Action::Window(command)),
        }
    }
}

struct Failure {
    tab_id: TabId,
    operation: Operation,
    error: TerminalSessionError,
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    Spawn,
    Write,
    Resize,
}

impl From<terminal::Operation> for Operation {
    fn from(operation: terminal::Operation) -> Self {
        match operation {
            terminal::Operation::Write => Self::Write,
            terminal::Operation::Resize => Self::Resize,
        }
    }
}

impl Workspace {
    pub(crate) fn new() -> (Self, Task<Message>) {
        let (launcher, event_task) = SessionLauncher::new();
        let mut workspace = Self {
            tabs: Tabs::new(),
            launcher,
            metrics: CellMetrics::default(),
            viewport: None,
        };
        let initial_update = workspace.open();
        (
            workspace,
            Task::batch([event_task.map(Message::SessionEvent), initial_update.task]),
        )
    }

    pub(crate) fn update(&mut self, message: Message) -> Update {
        match message {
            Message::TitleBar(message) => self.handle_title_bar(message),
            Message::Keyboard(event) => self.handle_keyboard(event),
            Message::SessionEvent(event) => self.handle_session_event(event),
            Message::Terminal(id, message) => self.update_terminal(id, message),
            Message::ViewportChanged(viewport) => self.set_viewport(viewport),
        }
    }

    pub(crate) fn view(&self) -> Element<'_, Message> {
        view::view(&self.tabs, self.metrics)
    }

    fn open(&mut self) -> Update {
        let id = self.tabs.reserve_id();
        let size = self.terminal_size_for_new_tab();
        match self.launcher.spawn(id, size) {
            Ok(session) => {
                let pane = TerminalPane::new(session, size);
                self.tabs.insert(Tab::new(id, pane));
                self.activate(id);
            }
            Err(error) => self.report_failure(&Failure {
                tab_id: id,
                operation: Operation::Spawn,
                error,
            }),
        }
        Update::none()
    }

    fn close(&mut self, id: TabId) -> Update {
        self.close_many(std::iter::once(id))
    }

    fn close_many(&mut self, ids: impl IntoIterator<Item = TabId>) -> Update {
        let mut removed_any = false;
        let mut active_changed = false;
        for id in ids {
            if let Some(removal) = self.tabs.remove(id) {
                removed_any = true;
                active_changed |= removal.active_changed;
                drop(removal.tab);
            }
        }

        if !removed_any {
            return Update::none();
        }

        let Some(active_id) = self.tabs.active_id() else {
            return Update::window(Command::Close);
        };
        if active_changed {
            self.activate(active_id);
        }
        Update::none()
    }

    fn activate(&mut self, id: TabId) {
        if self.tabs.activate(id)
            && let Some(tab) = self.tabs.get_mut(id)
        {
            tab.activate();
        }
    }

    fn handle_keyboard(&mut self, event: keyboard::Event) -> Update {
        match shortcuts::resolve(&event) {
            shortcuts::Resolution::Execute(action) => self.execute_shortcut(action),
            shortcuts::Resolution::Suppress => Update::none(),
            shortcuts::Resolution::Unhandled => match self.tabs.active_id() {
                Some(id) => self.update_terminal(id, terminal::Message::Keyboard(event)),
                None => Update::none(),
            },
        }
    }

    fn execute_shortcut(&mut self, action: shortcuts::Action) -> Update {
        match action {
            shortcuts::Action::OpenTab => self.open(),
            shortcuts::Action::CloseActiveTab => match self.tabs.active_id() {
                Some(id) => self.close(id),
                None => Update::none(),
            },
            shortcuts::Action::NextTab | shortcuts::Action::PreviousTab => {
                let candidate = if action == shortcuts::Action::NextTab {
                    self.tabs.next_candidate_id()
                } else {
                    self.tabs.previous_candidate_id()
                };
                if let Some(id) = candidate {
                    self.activate(id);
                }
                Update::none()
            }
        }
    }

    fn handle_title_bar(&mut self, message: title_bar::Message) -> Update {
        match message {
            title_bar::Message::MinimizeWindow => Update::window(Command::Minimize),
            title_bar::Message::ToggleMaximize => Update::window(Command::ToggleMaximize),
            title_bar::Message::CloseWindow => Update::window(Command::Close),
            title_bar::Message::StartWindowDrag => Update::window(Command::Drag),
            title_bar::Message::NewTabPressed => self.open(),
            title_bar::Message::TabPressed(id) => {
                self.activate(id);
                Update::none()
            }
            title_bar::Message::TabClosePressed(id) => self.close(id),
            title_bar::Message::TabHoverChanged { id, is_hovered } => {
                self.tabs.set_hovered(id, is_hovered);
                Update::none()
            }
        }
    }

    fn handle_session_event(&mut self, event: SessionEvent) -> Update {
        self.update_terminal(event.tab_id, terminal::Message::Event(event.event))
    }

    fn update_terminal(&mut self, id: TabId, message: terminal::Message) -> Update {
        let visible = self.tabs.active_id() == Some(id);
        let Some(tab) = self.tabs.get_mut(id) else {
            return Update::none();
        };
        let outcome = tab.update(message, visible);
        self.apply_terminal_outcome(id, outcome)
    }

    fn apply_terminal_outcome(&mut self, id: TabId, outcome: tab::Outcome) -> Update {
        match outcome {
            tab::Outcome::None => Update::none(),
            tab::Outcome::Bell => {
                // TODO(notification): Surface terminal bells without blocking the UI thread.
                Update::none()
            }
            tab::Outcome::Exited(status) => {
                if let Some(status) = status {
                    eprintln!("terminal tab {} exited: {status}", id.0);
                }
                self.close(id)
            }
            tab::Outcome::Failed { operation, error } => {
                self.report_failure(&Failure {
                    tab_id: id,
                    operation: operation.into(),
                    error,
                });
                self.close(id)
            }
        }
    }

    fn set_viewport(&mut self, viewport: Viewport) -> Update {
        self.viewport = Some(viewport);
        match metrics::size_for_viewport(viewport.size, viewport.scale_factor, self.metrics) {
            Some(size) => self.resize_all(size),
            None => Update::none(),
        }
    }

    fn terminal_size_for_new_tab(&self) -> TerminalSize {
        self.viewport
            .and_then(|viewport| {
                metrics::size_for_viewport(viewport.size, viewport.scale_factor, self.metrics)
            })
            .unwrap_or_else(|| metrics::initial_size(self.metrics))
    }

    fn resize_all(&mut self, size: TerminalSize) -> Update {
        let mut failures = Vec::new();
        for tab in self.tabs.items_mut() {
            if let tab::Outcome::Failed { operation, error } =
                tab.update(terminal::Message::Resize(size), false)
            {
                failures.push(Failure {
                    tab_id: tab.id(),
                    operation: operation.into(),
                    error,
                });
            }
        }

        for failure in &failures {
            self.report_failure(failure);
        }
        let update = self.close_many(failures.into_iter().map(|failure| failure.tab_id));
        if let Some(id) = self.tabs.active_id() {
            self.activate(id);
        }
        update
    }

    fn report_failure(&self, failure: &Failure) {
        eprintln!(
            "terminal tab {} {:?} failed: {}",
            failure.tab_id.0, failure.operation, failure.error
        );
    }
}
