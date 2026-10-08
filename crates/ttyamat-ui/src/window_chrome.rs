use iced::{
    Element,
    Length::{self, Fill},
    Size, mouse,
    widget::{column, mouse_area, row, space},
    window,
};

use crate::workspace::title_bar;

const RESIZE_BORDER: f32 = 6.0;

pub(crate) fn resize_handles() -> Element<'static, crate::window::Command> {
    let border = Length::Fixed(RESIZE_BORDER);

    let north_west = resize_handle(
        window::Direction::NorthWest,
        border,
        border,
        mouse::Interaction::ResizingDiagonallyDown,
    );

    let north = resize_handle(
        window::Direction::North,
        Fill,
        border,
        mouse::Interaction::ResizingVertically,
    );

    let north_east = resize_handle(
        window::Direction::NorthEast,
        border,
        border,
        mouse::Interaction::ResizingDiagonallyUp,
    );

    let west = resize_handle(
        window::Direction::West,
        border,
        Fill,
        mouse::Interaction::ResizingHorizontally,
    );

    let east = resize_handle(
        window::Direction::East,
        border,
        Fill,
        mouse::Interaction::ResizingHorizontally,
    );

    let south_west = resize_handle(
        window::Direction::SouthWest,
        border,
        border,
        mouse::Interaction::ResizingDiagonallyUp,
    );

    let south = resize_handle(
        window::Direction::South,
        Fill,
        border,
        mouse::Interaction::ResizingVertically,
    );

    let south_east = resize_handle(
        window::Direction::SouthEast,
        border,
        border,
        mouse::Interaction::ResizingDiagonallyDown,
    );

    let top = row![north_west, north, north_east]
        .width(Fill)
        .height(border);

    let center = row![west, space::Space::new().width(Fill).height(Fill), east]
        .width(Fill)
        .height(Fill);

    let bottom = row![south_west, south, south_east]
        .width(Fill)
        .height(border);

    column![top, center, bottom].width(Fill).height(Fill).into()
}

fn resize_handle(
    direction: iced::window::Direction,
    width: Length,
    height: Length,
    interaction: mouse::Interaction,
) -> Element<'static, crate::window::Command> {
    mouse_area(space::Space::new().width(width).height(height))
        .on_press(crate::window::Command::BeginResize(direction))
        .interaction(interaction)
        .into()
}

/// Returns the terminal pane's logical bounds below the title bar, including padding.
pub(crate) fn terminal_viewport(window_size: Size) -> Size {
    Size::new(
        window_size.width,
        (window_size.height - title_bar::HEIGHT).max(0.0),
    )
}
