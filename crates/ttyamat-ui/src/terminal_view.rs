use iced::widget::text::{LineHeight, Shaping, Wrapping};
use iced::widget::{Column, container, stack, text};
use iced::{Element, Fill, Font, Pixels, Size, Theme};
use ttyamat_terminal::{TerminalCursorShape, TerminalFrame, TerminalSize};

use crate::{style, title_bar};

pub(crate) const HORIZONTAL_PADDING: f32 = 10.0;
pub(crate) const VERTICAL_PADDING: f32 = 8.0;
pub(crate) const CELL_WIDTH: f32 = 8.0;
pub(crate) const CELL_HEIGHT: f32 = 16.0;
const FONT_SIZE: f32 = 14.0;

#[derive(Clone, Debug)]
pub(crate) enum Message {}

pub(crate) fn view(frame: Option<&TerminalFrame>) -> Element<'_, Message> {
    let rows = frame.into_iter().flat_map(TerminalFrame::rows).map(|row| {
        text(row)
            .font(Font::MONOSPACE)
            .size(FONT_SIZE)
            .line_height(LineHeight::Absolute(Pixels(CELL_HEIGHT)))
            .shaping(Shaping::Basic)
            .wrapping(Wrapping::None)
            .width(Fill)
            .height(CELL_HEIGHT)
            .into()
    });
    let content = Column::with_children(rows).width(Fill).height(Fill);
    let cursor = text(cursor_overlay(frame))
        .font(Font::MONOSPACE)
        .size(FONT_SIZE)
        .line_height(LineHeight::Absolute(Pixels(CELL_HEIGHT)))
        .shaping(Shaping::Basic)
        .wrapping(Wrapping::None)
        .width(Fill)
        .height(Fill);
    let terminal = stack![content, cursor].width(Fill).height(Fill);

    container(terminal)
        .width(Fill)
        .height(Fill)
        .padding([VERTICAL_PADDING, HORIZONTAL_PADDING])
        .style(terminal_style)
        .into()
}

fn cursor_overlay(frame: Option<&TerminalFrame>) -> String {
    let Some(cursor) = frame.and_then(TerminalFrame::cursor) else {
        return String::new();
    };
    let mut overlay = String::with_capacity(usize::from(cursor.line() + cursor.column()) + 1);
    overlay.extend(std::iter::repeat_n('\n', usize::from(cursor.line())));
    overlay.extend(std::iter::repeat_n(' ', usize::from(cursor.column())));
    overlay.push(match cursor.shape() {
        TerminalCursorShape::Block => '▏',
        TerminalCursorShape::Underline => '_',
        TerminalCursorShape::Beam => '│',
        TerminalCursorShape::HollowBlock => '▏',
    });
    overlay
}

pub(crate) fn size_for_window(window_size: Size, scale_factor: f32) -> Option<TerminalSize> {
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        return None;
    }

    let usable_width = window_size.width - HORIZONTAL_PADDING * 2.0;
    let usable_height = window_size.height - title_bar::HEIGHT - VERTICAL_PADDING * 2.0;

    if !usable_width.is_finite()
        || !usable_height.is_finite()
        || usable_width < CELL_WIDTH
        || usable_height < CELL_HEIGHT
    {
        return None;
    }

    let columns = clamp_to_u16((usable_width / CELL_WIDTH).floor());
    let lines = clamp_to_u16((usable_height / CELL_HEIGHT).floor());
    let physical_cell_width = clamp_to_u16((CELL_WIDTH * scale_factor).round());
    let physical_cell_height = clamp_to_u16((CELL_HEIGHT * scale_factor).round());

    TerminalSize::new(columns, lines, physical_cell_width, physical_cell_height)
}

fn clamp_to_u16(value: f32) -> u16 {
    value.clamp(1.0, f32::from(u16::MAX)) as u16
}

fn terminal_style(_: &Theme) -> container::Style {
    container::Style::default()
        .background(style::TERMINAL_BACKGROUND)
        .color(style::PRIMARY_TEXT)
}
