use iced::widget::text::{LineHeight, Shaping, Wrapping};
use iced::widget::{Column, container, stack, text};
use iced::{Element, Fill, Font, Pixels, Theme};
use ttyamat_terminal::{TerminalCursorShape, TerminalFrame};

use super::metrics::{CellMetrics, HORIZONTAL_PADDING, VERTICAL_PADDING};
use crate::style;

pub(super) fn view(frame: &TerminalFrame, metrics: CellMetrics) -> Element<'_, super::Message> {
    let rows = frame.rows().iter().map(|row| {
        text(row)
            .font(Font::MONOSPACE)
            .size(metrics.font_size)
            .line_height(LineHeight::Absolute(Pixels(metrics.height)))
            .shaping(Shaping::Basic)
            .wrapping(Wrapping::None)
            .width(Fill)
            .height(metrics.height)
            .into()
    });
    let content = Column::with_children(rows).width(Fill).height(Fill);
    let cursor = text(cursor_overlay(frame))
        .font(Font::MONOSPACE)
        .size(metrics.font_size)
        .line_height(LineHeight::Absolute(Pixels(metrics.height)))
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

fn cursor_overlay(frame: &TerminalFrame) -> String {
    let Some(cursor) = frame.cursor() else {
        return String::new();
    };

    let mut overlay =
        String::with_capacity(usize::from(cursor.line()) + usize::from(cursor.column()) + 1);
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

fn terminal_style(_: &Theme) -> container::Style {
    container::Style::default()
        .background(style::TERMINAL_BACKGROUND)
        .color(style::PRIMARY_TEXT)
}
