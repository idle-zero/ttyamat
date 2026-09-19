use alacritty_terminal::{
    event::{Event, EventListener, WindowSize},
    event_loop::{EventLoop, EventLoopSender, Msg, State as EventLoopState},
    sync::FairMutex,
    term::{Config as TermConfig, Term},
    tty,
};
use std::{
    borrow::Cow,
    io,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
};
use thiserror::Error;

use crate::{TerminalEvent, TerminalFrame, TerminalRenderUpdate, TerminalSize};

struct AlacrittyDimensions {
    columns: usize,
    lines: usize,
}

impl alacritty_terminal::grid::Dimensions for AlacrittyDimensions {
    fn total_lines(&self) -> usize {
        self.lines
    }

    fn screen_lines(&self) -> usize {
        self.lines
    }

    fn columns(&self) -> usize {
        self.columns
    }
}

impl From<TerminalSize> for AlacrittyDimensions {
    fn from(size: TerminalSize) -> Self {
        Self {
            columns: usize::from(size.columns()),
            lines: usize::from(size.lines()),
        }
    }
}

fn alacritty_window_size(size: TerminalSize) -> WindowSize {
    WindowSize {
        num_lines: size.lines(),
        num_cols: size.columns(),
        cell_width: size.cell_width(),
        cell_height: size.cell_height(),
    }
}

#[derive(Clone)]
struct EventProxy {
    on_event: Arc<dyn Fn(TerminalEvent) + Send + Sync>,
    pty_sender: Arc<OnceLock<EventLoopSender>>,
    wakeup_pending: Arc<AtomicBool>,
}

impl EventProxy {
    fn new(on_event: impl Fn(TerminalEvent) + Send + Sync + 'static) -> Self {
        Self {
            on_event: Arc::new(on_event),
            pty_sender: Arc::new(OnceLock::new()),
            wakeup_pending: Arc::new(AtomicBool::new(false)),
        }
    }

    fn attach_pty_sender(&self, sender: EventLoopSender) {
        assert!(
            self.pty_sender.set(sender).is_ok(),
            "PTY sender must only be attached once"
        );
    }

    fn emit(&self, event: TerminalEvent) {
        (self.on_event)(event);
    }
}

impl EventListener for EventProxy {
    fn send_event(&self, event: Event) {
        match event {
            Event::Wakeup => {
                if !self.wakeup_pending.swap(true, Ordering::AcqRel) {
                    self.emit(TerminalEvent::Wakeup);
                }
            }

            Event::Title(title) => {
                self.emit(TerminalEvent::TitleChanged(Some(title)));
            }

            Event::ResetTitle => {
                self.emit(TerminalEvent::TitleChanged(None));
            }

            Event::Bell => {
                self.emit(TerminalEvent::Bell);
            }

            Event::ChildExit(status) => {
                self.emit(TerminalEvent::ChildExited(status));
            }

            Event::Exit => {
                self.emit(TerminalEvent::ExitRequested);
            }
            Event::PtyWrite(text) => {
                if let Some(sender) = self.pty_sender.get() {
                    let _ = sender.send(Msg::Input(Cow::Owned(text.into_bytes())));
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Error)]
pub enum TerminalSessionError {
    #[error("terminal I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("terminal event-loop error: {0}")]
    EventLoop(String),

    #[error("terminal worker thread panicked")]
    WorkerPanicked,
}

type AlacrittyEventLoop = EventLoop<tty::Pty, EventProxy>;
type PtyThread = JoinHandle<(AlacrittyEventLoop, EventLoopState)>;

pub struct TerminalSession {
    terminal: Arc<FairMutex<Term<EventProxy>>>,
    sender: EventLoopSender,
    io_thread: Option<PtyThread>,
    wakeup_pending: Arc<AtomicBool>,
}

impl TerminalSession {
    pub fn spawn_default(
        size: TerminalSize,
        on_event: impl Fn(TerminalEvent) + Send + Sync + 'static,
    ) -> Result<Self, TerminalSessionError> {
        let dimensions = AlacrittyDimensions::from(size);
        let window_size = alacritty_window_size(size);

        let mut tty_options = tty::Options::default();
        tty_options
            .env
            .insert("TERM".into(), "xterm-256color".into());
        tty_options
            .env
            .insert("COLORTERM".into(), "truecolor".into());
        tty_options
            .env
            .insert("TERM_PROGRAM".into(), "ttyamat".into());

        let event_proxy = EventProxy::new(on_event);
        let terminal = Arc::new(FairMutex::new(Term::new(
            TermConfig::default(),
            &dimensions,
            event_proxy.clone(),
        )));

        let pty = tty::new(&tty_options, window_size, 0)?;
        let event_loop = EventLoop::new(
            Arc::clone(&terminal),
            event_proxy.clone(),
            pty,
            tty_options.drain_on_exit,
            false,
        )?;

        let sender = event_loop.channel();
        event_proxy.attach_pty_sender(sender.clone());
        let io_thread = event_loop.spawn();
        let wakeup_pending = Arc::clone(&event_proxy.wakeup_pending);

        Ok(Self {
            terminal,
            sender,
            io_thread: Some(io_thread),
            wakeup_pending,
        })
    }

    pub fn write(&self, bytes: Vec<u8>) -> Result<(), TerminalSessionError> {
        self.send(Msg::Input(Cow::Owned(bytes)))
    }

    pub fn resize(&self, size: TerminalSize) -> Result<(), TerminalSessionError> {
        let dimensions = AlacrittyDimensions::from(size);
        let window_size = alacritty_window_size(size);

        self.terminal.lock().resize(dimensions);
        self.send(Msg::Resize(window_size))
    }

    pub fn initial_frame(&self) -> TerminalFrame {
        TerminalFrame::capture(&mut self.terminal.lock())
    }

    pub fn take_render_update(&self) -> Option<TerminalRenderUpdate> {
        TerminalRenderUpdate::capture(&mut self.terminal.lock())
    }

    pub fn acknowledge_wakeup(&self) {
        self.wakeup_pending.store(false, Ordering::Release);
    }

    pub fn shutdown(&mut self) -> Result<(), TerminalSessionError> {
        let Some(io_thread) = self.io_thread.take() else {
            return Ok(());
        };

        let _ = self.sender.send(Msg::Shutdown);
        io_thread
            .join()
            .map_err(|_| TerminalSessionError::WorkerPanicked)?;

        Ok(())
    }

    fn send(&self, message: Msg) -> Result<(), TerminalSessionError> {
        self.sender
            .send(message)
            .map_err(|error| TerminalSessionError::EventLoop(error.to_string()))
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
