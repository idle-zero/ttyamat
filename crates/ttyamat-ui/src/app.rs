use iced::widget::stack;
use iced::{Element, Fill, Subscription, Task, Theme, keyboard, window};

use crate::window::{Change, Command, WindowState};
use crate::window_chrome;
use crate::workspace::{self, TerminalViewport, Workspace};

struct App {
    window: WindowState,
    workspace: Workspace,
}

#[derive(Debug, Clone)]
enum Message {
    Window {
        id: window::Id,
        event: window::Event,
    },
    Keyboard {
        window_id: window::Id,
        event: keyboard::Event,
        status: iced::event::Status,
    },
    WindowCommand(Command),
    Workspace(workspace::Message),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let (workspace, task) = Workspace::new();
        (
            Self {
                window: WindowState::new(),
                workspace,
            },
            task.map(Message::Workspace),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Window { id, event } => self.handle_window_event(id, event),
            Message::Keyboard {
                window_id,
                event,
                status,
            } => self.handle_keyboard_event(window_id, event, status),
            Message::WindowCommand(command) => self.window.execute_command(command),
            Message::Workspace(message) => {
                let update = self.workspace.update(message);
                self.apply_workspace_update(update)
            }
        }
    }

    fn handle_window_event(&mut self, id: window::Id, event: window::Event) -> Task<Message> {
        let change = self.window.handle_event(id, event);
        if change == Change::GeometryChanged
            && let Some(size) = self.window.size()
        {
            let viewport = TerminalViewport {
                size: window_chrome::terminal_viewport(size),
                scale_factor: self.window.scale_factor(),
            };
            let update = self
                .workspace
                .update(workspace::Message::TerminalViewportChanged(viewport));
            return self.apply_workspace_update(update);
        }

        Task::none()
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

        let update = self.workspace.update(workspace::Message::Keyboard(event));
        self.apply_workspace_update(update)
    }

    fn apply_workspace_update(&self, update: workspace::Update) -> Task<Message> {
        let window_task = match update.action {
            Some(workspace::Action::Window(command)) => self.window.execute_command(command),
            None => Task::none(),
        };

        Task::batch([update.task.map(Message::Workspace), window_task])
    }

    fn view(&self) -> Element<'_, Message> {
        stack![
            self.workspace.view().map(Message::Workspace),
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
