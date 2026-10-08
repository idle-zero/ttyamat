use iced::Element;

use crate::terminal::{self, TerminalPane, metrics::TerminalMetrics};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TabId(pub(crate) u64);

pub(super) struct Tab {
    id: TabId,
    title: String,
    fallback_title: String,
    terminal: TerminalPane,
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

    pub(super) fn terminal_mut(&mut self) -> &mut TerminalPane {
        &mut self.terminal
    }

    pub(super) fn set_title(&mut self, title: Option<String>) {
        self.title = title.unwrap_or_else(|| self.fallback_title.clone());
    }

    pub(super) fn view(&self, metrics: TerminalMetrics) -> Element<'_, terminal::Message> {
        self.terminal.view(metrics)
    }
}
