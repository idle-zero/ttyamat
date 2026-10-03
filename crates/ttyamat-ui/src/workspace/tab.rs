use iced::Element;
use ttyamat_terminal::TerminalSessionError;

use crate::terminal::{self, TerminalPane, metrics::TerminalMetrics};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TabId(pub(crate) u64);

pub(super) struct Tab {
    id: TabId,
    title: String,
    fallback_title: String,
    terminal: TerminalPane,
}

pub(super) enum Outcome {
    None,
    Bell,
    ChildExited(std::process::ExitStatus),
    ExitRequested,
    Failed {
        operation: terminal::Operation,
        error: TerminalSessionError,
    },
}

impl Tab {
    pub(super) fn new(id: TabId, terminal: TerminalPane) -> Self {
        let fallback_title = format!("Tab {}", id.0);
        Self {
            id,
            title: fallback_title.clone(),
            fallback_title,
            terminal,
        }
    }

    pub(super) fn title(&self) -> &str {
        &self.title
    }

    pub(super) fn id(&self) -> TabId {
        self.id
    }

    pub(super) fn refresh(&mut self) {
        self.terminal.refresh_frame();
    }

    pub(super) fn update(&mut self, message: terminal::Message, visible: bool) -> Outcome {
        match self.terminal.update(message, visible) {
            terminal::Outcome::None => Outcome::None,
            terminal::Outcome::TitleChanged(title) => {
                self.title = title.unwrap_or_else(|| self.fallback_title.clone());
                Outcome::None
            }
            terminal::Outcome::Bell => Outcome::Bell,
            terminal::Outcome::ChildExited(status) => Outcome::ChildExited(status),
            terminal::Outcome::ExitRequested => Outcome::ExitRequested,
            terminal::Outcome::Failed { operation, error } => Outcome::Failed { operation, error },
        }
    }

    pub(super) fn view(&self, metrics: TerminalMetrics) -> Element<'_, terminal::Message> {
        self.terminal.view(metrics)
    }
}
