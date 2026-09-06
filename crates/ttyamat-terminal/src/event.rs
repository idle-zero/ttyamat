use std::process::ExitStatus;

#[derive(Debug, Clone)]
pub enum TerminalEvent {
    Wakeup,
    TitleChanged(Option<String>),
    Bell,
    ChildExited(ExitStatus),
    ExitRequested,
}
