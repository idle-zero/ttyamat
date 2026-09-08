use ttyamat_terminal::{TerminalFrame, TerminalSession};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TabId(pub u64);

pub(crate) struct Tab {
    pub id: TabId,
    pub title: String,
    pub fallback_title: String,
    pub session: TerminalSession,
    pub terminal_frame: TerminalFrame,
    pub terminal_frame_dirty: bool,
}
