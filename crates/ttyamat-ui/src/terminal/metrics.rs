use iced::Size;
use ttyamat_terminal::TerminalSize;

const INITIAL_TERMINAL_COLUMNS: u16 = 80;
const INITIAL_TERMINAL_LINES: u16 = 24;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TerminalMetrics {
    pub(crate) cell_width: f32,
    pub(crate) cell_height: f32,
    pub(crate) font_size: f32,
}

impl Default for TerminalMetrics {
    fn default() -> Self {
        Self {
            cell_width: 8.0,
            cell_height: 16.0,
            font_size: 14.0,
        }
    }
}

impl TerminalMetrics {
    fn is_valid(self) -> bool {
        [self.cell_width, self.cell_height, self.font_size]
            .into_iter()
            .all(|dimension| dimension.is_finite() && dimension > 0.0)
    }
}

pub(crate) const HORIZONTAL_PADDING: f32 = 10.0;
pub(crate) const VERTICAL_PADDING: f32 = 8.0;

pub(crate) fn initial_size(metrics: TerminalMetrics) -> TerminalSize {
    assert!(metrics.is_valid(), "initial terminal metrics must be valid");

    TerminalSize::new(
        INITIAL_TERMINAL_COLUMNS,
        INITIAL_TERMINAL_LINES,
        clamp_cell_pixels(metrics.cell_width.round()),
        clamp_cell_pixels(metrics.cell_height.round()),
    )
    .expect("initial terminal dimensions must be valid")
}

pub(crate) fn size_for_viewport(
    viewport: Size,
    scale_factor: f32,
    metrics: TerminalMetrics,
) -> Option<TerminalSize> {
    if !metrics.is_valid() || !scale_factor.is_finite() || scale_factor <= 0.0 {
        return None;
    }

    let usable_width = viewport.width - HORIZONTAL_PADDING * 2.0;
    let usable_height = viewport.height - VERTICAL_PADDING * 2.0;

    if !usable_width.is_finite()
        || !usable_height.is_finite()
        || usable_width < metrics.cell_width
        || usable_height < metrics.cell_height
    {
        return None;
    }

    let columns = (usable_width / metrics.cell_width).floor().clamp(
        f32::from(TerminalSize::MIN_COLUMNS),
        f32::from(TerminalSize::MAX_COLUMNS),
    ) as u16;
    let lines = (usable_height / metrics.cell_height).floor().clamp(
        f32::from(TerminalSize::MIN_LINES),
        f32::from(TerminalSize::MAX_LINES),
    ) as u16;
    let physical_cell_width = clamp_cell_pixels((metrics.cell_width * scale_factor).round());
    let physical_cell_height = clamp_cell_pixels((metrics.cell_height * scale_factor).round());

    TerminalSize::new(columns, lines, physical_cell_width, physical_cell_height)
}

fn clamp_cell_pixels(value: f32) -> u16 {
    value.clamp(1.0, f32::from(u16::MAX)) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn narrow_viewport_keeps_two_columns() {
        let size = size_for_viewport(Size::new(28.0, 32.0), 1.0, TerminalMetrics::default())
            .expect("a usable narrow viewport should clamp to the minimum");
        assert_eq!(size.columns(), 2);
        assert_eq!(size.lines(), 1);
        assert_eq!(size.cell_width(), 8);
        assert_eq!(size.cell_height(), 16);
    }
}
