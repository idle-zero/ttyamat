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
        clamp_to_u16(metrics.cell_width.round()),
        clamp_to_u16(metrics.cell_height.round()),
    )
    .expect("initial terminal dimensions must be non-zero")
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

    let columns = clamp_to_u16((usable_width / metrics.cell_width).floor());
    let lines = clamp_to_u16((usable_height / metrics.cell_height).floor());
    let physical_cell_width = clamp_to_u16((metrics.cell_width * scale_factor).round());
    let physical_cell_height = clamp_to_u16((metrics.cell_height * scale_factor).round());

    TerminalSize::new(columns, lines, physical_cell_width, physical_cell_height)
}

fn clamp_to_u16(value: f32) -> u16 {
    value.clamp(1.0, f32::from(u16::MAX)) as u16
}
