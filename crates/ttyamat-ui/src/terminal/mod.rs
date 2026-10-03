use ttyamat_terminal::{
    TerminalEvent, TerminalFrame, TerminalSession, TerminalSessionError, TerminalSize,
};

mod input;
pub(crate) mod metrics;
mod view;

pub(crate) struct TerminalPane {
    session: TerminalSession,
    frame: TerminalFrame,
    dirty: bool,
    size: TerminalSize,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Event(TerminalEvent),
    Keyboard(iced::keyboard::Event),
    Resize(TerminalSize),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Operation {
    Write,
    Resize,
}

pub(crate) enum Outcome {
    None,
    TitleChanged(Option<String>),
    Bell,
    ChildExited(std::process::ExitStatus),
    ExitRequested,
    Failed {
        operation: Operation,
        error: TerminalSessionError,
    },
}

impl TerminalPane {
    pub(crate) fn new(session: TerminalSession, size: TerminalSize) -> Self {
        let frame = session.initial_frame();
        Self {
            session,
            frame,
            dirty: false,
            size,
        }
    }

    /// Handle one pane input and return any metadata, lifecycle, or failure outcome.
    pub(crate) fn update(&mut self, message: Message, visible: bool) -> Outcome {
        match message {
            Message::Event(event) => self.handle_event(event, visible),
            Message::Keyboard(event) => self.handle_keyboard(&event),
            Message::Resize(size) => self.resize(size, visible),
        }
    }

    /// Render the displayed snapshot without reading or locking the backend session.
    pub(crate) fn view(&self, metrics: metrics::TerminalMetrics) -> iced::Element<'_, Message> {
        view::view(&self.frame, metrics)
    }

    /// Consume pending frame damage when visible or selected; clean panes do no work.
    pub(crate) fn refresh_frame(&mut self) {
        if !self.dirty {
            return;
        }

        // Acknowledge before capture so output arriving during capture can wake us again.
        self.session.acknowledge_wakeup();
        if let Some(update) = self.session.take_render_update() {
            self.frame.apply(update);
        }
        self.dirty = false;
    }

    /// Refresh visible wakeups and report events whose policy belongs to the parent.
    fn handle_event(&mut self, event: TerminalEvent, visible: bool) -> Outcome {
        match event {
            TerminalEvent::Wakeup => {
                self.dirty = true;
                if visible {
                    self.refresh_frame();
                }
                Outcome::None
            }
            TerminalEvent::TitleChanged(title) => Outcome::TitleChanged(title),
            TerminalEvent::Bell => Outcome::Bell,
            TerminalEvent::ChildExited(status) => Outcome::ChildExited(status),
            TerminalEvent::ExitRequested => Outcome::ExitRequested,
        }
    }

    /// Encode admitted terminal input and preserve any session write error.
    fn handle_keyboard(&mut self, event: &iced::keyboard::Event) -> Outcome {
        let Some(bytes) = input::encode(event) else {
            return Outcome::None;
        };

        match self.session.write(bytes) {
            Ok(()) => Outcome::None,
            Err(error) => Outcome::Failed {
                operation: Operation::Write,
                error,
            },
        }
    }

    /// Resize only when the successful size changes and defer hidden frame refreshes.
    fn resize(&mut self, size: TerminalSize, visible: bool) -> Outcome {
        if size == self.size {
            return Outcome::None;
        }

        match self.session.resize(size) {
            Ok(()) => {
                self.size = size;
                self.dirty = true;
                if visible {
                    self.refresh_frame();
                }
                Outcome::None
            }
            Err(error) => Outcome::Failed {
                operation: Operation::Resize,
                error,
            },
        }
    }
}
