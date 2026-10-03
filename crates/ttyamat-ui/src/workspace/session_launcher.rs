use super::tab::TabId;
use iced::{Task, futures::channel::mpsc};
use ttyamat_terminal::{TerminalEvent, TerminalSession, TerminalSessionError, TerminalSize};

#[derive(Debug, Clone)]
pub(crate) struct TabSessionEvent {
    pub(crate) tab_id: TabId,
    pub(crate) event: TerminalEvent,
}

pub(super) struct SessionLauncher {
    event_sender: mpsc::UnboundedSender<TabSessionEvent>,
}

impl SessionLauncher {
    pub(super) fn new() -> (Self, Task<TabSessionEvent>) {
        let (sender, receiver) = mpsc::unbounded();

        (
            Self {
                event_sender: sender,
            },
            Task::stream(receiver),
        )
    }

    pub(super) fn spawn(
        &self,
        tab_id: TabId,
        size: TerminalSize,
    ) -> Result<TerminalSession, TerminalSessionError> {
        let sender = self.event_sender.clone();
        TerminalSession::spawn_default(size, move |event| {
            let _ = sender.unbounded_send(TabSessionEvent { tab_id, event });
        })
    }
}
