use std::process::ExitStatus;

#[derive(Debug)]
pub enum TerminalEvent {
    Wakeup,
    TitleChanged(Option<String>),
    Bell,
    ChildExited(ExitStatus),
    ExitRequested,
}
