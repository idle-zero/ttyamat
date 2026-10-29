#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    columns: u16,
    lines: u16,
    cell_width: u16,
    cell_height: u16,
}

impl TerminalSize {
    pub const MIN_COLUMNS: u16 = 2;
    pub const MIN_LINES: u16 = 1;

    pub const MAX_COLUMNS: u16 = i16::MAX as u16;
    pub const MAX_LINES: u16 = i16::MAX as u16;

    pub fn new(columns: u16, lines: u16, cell_width: u16, cell_height: u16) -> Option<Self> {
        if !(Self::MIN_COLUMNS..=Self::MAX_COLUMNS).contains(&columns)
            || !(Self::MIN_LINES..=Self::MAX_LINES).contains(&lines)
            || cell_width == 0
            || cell_height == 0
        {
            return None;
        }

        Some(Self {
            columns,
            lines,
            cell_width,
            cell_height,
        })
    }

    pub fn columns(&self) -> u16 {
        self.columns
    }

    pub fn lines(&self) -> u16 {
        self.lines
    }

    pub fn cell_width(&self) -> u16 {
        self.cell_width
    }

    pub fn cell_height(&self) -> u16 {
        self.cell_height
    }
}

#[cfg(test)]
mod tests {
    use super::TerminalSize;

    #[test]
    fn rejects_one_column() {
        assert!(
            TerminalSize::new(1, 24, 8, 16).is_none(),
            "a one-column grid cannot safely accept wide characters"
        )
    }

    #[test]
    fn accepts_minimum_columns() {
        assert!(TerminalSize::new(2, 24, 8, 16).is_some());
    }
}
