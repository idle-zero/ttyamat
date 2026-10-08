use iced::event::Status;
use iced::window::{self, Event, Id};
use iced::{Size, Task};

pub(crate) struct WindowState {
    id: Option<Id>,
    focused: bool,
    size: Option<Size>,
    scale_factor: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Command {
    Minimize,
    ToggleMaximize,
    Close,
    BeginDrag,
    BeginResize(window::Direction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Change {
    None,
    GeometryChanged,
}

impl WindowState {
    pub(crate) fn new() -> Self {
        Self {
            id: None,
            focused: false,
            size: None,
            scale_factor: 1.0,
        }
    }

    pub(crate) fn size(&self) -> Option<Size> {
        self.size
    }

    pub(crate) fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    pub(crate) fn accepts_keyboard(&self, id: Id, status: Status) -> bool {
        self.id == Some(id) && self.focused && status != Status::Captured
    }

    pub(crate) fn handle_event(&mut self, id: Id, event: Event) -> Change {
        match self.id {
            Some(known_id) if known_id != id => return Change::None,
            None if !matches!(event, Event::Opened { .. }) => return Change::None,
            _ => {}
        }

        match event {
            Event::Opened { size, .. } => {
                self.id = Some(id);
                self.size = Some(size);
                Change::GeometryChanged
            }
            Event::Resized(size) => {
                self.size = Some(size);
                Change::GeometryChanged
            }
            Event::Rescaled(scale) => {
                self.scale_factor = scale;
                Change::GeometryChanged
            }
            Event::Focused => {
                self.focused = true;
                Change::None
            }
            Event::Unfocused => {
                self.focused = false;
                Change::None
            }

            _ => Change::None,
        }
    }

    pub(crate) fn execute_command<M: 'static>(&self, command: Command) -> Task<M> {
        let Some(id) = self.id else {
            return Task::none();
        };

        match command {
            Command::ToggleMaximize => window::toggle_maximize(id),
            Command::Minimize => window::minimize(id, true),
            Command::Close => window::close(id),
            Command::BeginDrag => window::drag(id),
            Command::BeginResize(direction) => window::drag_resize(id, direction),
        }
    }
}
