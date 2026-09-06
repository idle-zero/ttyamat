#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    columns: u16,
    lines: u16,
    cell_width: u16,
    cell_height: u16,
}

impl TerminalSize {
    pub fn new(columns: u16, lines: u16, cell_width: u16, cell_height: u16) -> Option<Self> {
        if columns == 0 || lines == 0 || cell_width == 0 || cell_height == 0 {
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
