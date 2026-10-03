use iced::Element;
use ttyamat_terminal::TerminalSessionError;

use crate::terminal::{self, TerminalPane, metrics::CellMetrics};

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
    Exited(Option<std::process::ExitStatus>),
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

    pub(super) fn activate(&mut self) {
        self.terminal.refresh();
    }

    pub(super) fn update(&mut self, message: terminal::Message, visible: bool) -> Outcome {
        match self.terminal.update(message, visible) {
            terminal::Outcome::None => Outcome::None,
            terminal::Outcome::TitleChanged(title) => {
                self.title = title.unwrap_or_else(|| self.fallback_title.clone());
                Outcome::None
            }
            terminal::Outcome::Bell => Outcome::Bell,
            terminal::Outcome::Exited(status) => Outcome::Exited(status),
            terminal::Outcome::Failed { operation, error } => Outcome::Failed { operation, error },
        }
    }

    pub(super) fn view(&self, metrics: CellMetrics) -> Element<'_, terminal::Message> {
        self.terminal.view(metrics)
    }
}
