use alacritty_terminal::{
    event::EventListener,
    grid::Dimensions,
    index::{Column, Line, Point},
    term::{LineDamageBounds, Term, TermDamage, cell::Cell as AlacrittyCell, cell::Flags},
    vte::ansi::CursorShape,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalCell {
    character: char,
    zero_width: Option<Box<[char]>>,
    hidden: bool,
    wide_spacer: bool,
}

impl Default for TerminalCell {
    fn default() -> Self {
        Self {
            character: ' ',
            zero_width: None,
            hidden: false,
            wide_spacer: false,
        }
    }
}

impl TerminalCell {
    fn capture(cell: &AlacrittyCell) -> Self {
        Self {
            character: cell.c,
            zero_width: cell.zerowidth().map(Into::into),
            hidden: cell.flags.contains(Flags::HIDDEN),
            wide_spacer: cell.flags.contains(Flags::WIDE_CHAR_SPACER),
        }
    }

    pub fn character(&self) -> char {
        self.character
    }

    pub fn zero_width(&self) -> &[char] {
        self.zero_width.as_deref().unwrap_or_default()
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub fn is_wide_spacer(&self) -> bool {
        self.wide_spacer
    }

    fn append_text(&self, output: &mut String) {
        if self.wide_spacer {
            return;
        }

        output.push(if self.hidden { ' ' } else { self.character });
        output.extend(self.zero_width());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalCursorShape {
    Block,
    Underline,
    Beam,
    HollowBlock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalCursor {
    line: u16,
    column: u16,
    shape: TerminalCursorShape,
}

impl TerminalCursor {
    pub fn line(self) -> u16 {
        self.line
    }

    pub fn column(self) -> u16 {
        self.column
    }

    pub fn shape(self) -> TerminalCursorShape {
        self.shape
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TerminalFrame {
    columns: u16,
    lines: u16,
    cells: Vec<TerminalCell>,
    cursor: Option<TerminalCursor>,
    rows: Vec<String>,
}

impl TerminalFrame {
    pub(crate) fn capture<T: EventListener>(terminal: &mut Term<T>) -> Self {
        let update = TerminalRenderUpdate::capture_full(terminal);
        terminal.reset_damage();

        let mut frame = Self::default();
        frame.apply(update);
        frame
    }

    pub fn apply(&mut self, update: TerminalRenderUpdate) {
        let dimensions_changed = self.columns != update.columns || self.lines != update.lines;

        let damaged_rows = match update.damage {
            TerminalRenderDamage::Full(cells) => {
                self.columns = update.columns;
                self.lines = update.lines;
                self.cells = cells;
                self.rows.resize_with(usize::from(self.lines), String::new);
                None
            }
            TerminalRenderDamage::Partial(spans) if !dimensions_changed => {
                let columns = usize::from(self.columns);
                let mut damaged_rows = Vec::with_capacity(spans.len());

                for span in spans {
                    let start = usize::from(span.line) * columns + usize::from(span.column);
                    let end = start + span.cells.len();

                    if let Some(destination) = self.cells.get_mut(start..end) {
                        destination.clone_from_slice(&span.cells);
                        damaged_rows.push(span.line);
                    }
                }

                Some(damaged_rows)
            }
            TerminalRenderDamage::Partial(_) => {
                self.columns = update.columns;
                self.lines = update.lines;
                self.cells = vec![
                    TerminalCell::default();
                    usize::from(update.columns) * usize::from(update.lines)
                ];
                self.rows = vec![String::new(); usize::from(self.lines)];
                None
            }
        };

        self.cursor = update.cursor;
        match damaged_rows {
            None => self.rebuild_all_rows(),
            Some(mut rows) => {
                rows.sort_unstable();
                rows.dedup();
                for row in rows {
                    self.rebuild_row(row);
                }
            }
        }
    }

    pub fn columns(&self) -> u16 {
        self.columns
    }

    pub fn lines(&self) -> u16 {
        self.lines
    }

    pub fn cell(&self, line: u16, column: u16) -> Option<&TerminalCell> {
        if column >= self.columns {
            return None;
        }

        let index = usize::from(line) * usize::from(self.columns) + usize::from(column);
        self.cells.get(index)
    }

    pub fn cursor(&self) -> Option<TerminalCursor> {
        self.cursor
    }

    pub fn rows(&self) -> &[String] {
        &self.rows
    }

    fn rebuild_all_rows(&mut self) {
        for line in 0..self.lines {
            self.rebuild_row(line);
        }
    }

    fn rebuild_row(&mut self, line: u16) {
        let columns = usize::from(self.columns);
        let start = usize::from(line) * columns;
        let end = start + columns;
        let Some(cells) = self.cells.get(start..end) else {
            return;
        };
        let Some(row) = self.rows.get_mut(usize::from(line)) else {
            return;
        };

        row.clear();
        row.reserve(columns);
        for cell in cells {
            cell.append_text(row);
        }

        row.truncate(row.trim_end_matches(' ').len());
    }
}

#[derive(Debug)]
pub struct TerminalRenderUpdate {
    columns: u16,
    lines: u16,
    cursor: Option<TerminalCursor>,
    damage: TerminalRenderDamage,
}

impl TerminalRenderUpdate {
    pub(crate) fn capture<T: EventListener>(terminal: &mut Term<T>) -> Option<Self> {
        let damage = match terminal.damage() {
            TermDamage::Full => CapturedDamage::Full,
            TermDamage::Partial(lines) => {
                let lines = lines.collect::<Vec<_>>();
                if lines.is_empty() {
                    terminal.reset_damage();
                    return None;
                }
                CapturedDamage::Partial(lines)
            }
        };

        let update = match damage {
            CapturedDamage::Full => Self::capture_full(terminal),
            CapturedDamage::Partial(lines) => Self::capture_partial(terminal, lines),
        };
        terminal.reset_damage();
        Some(update)
    }

    fn capture_full<T: EventListener>(terminal: &Term<T>) -> Self {
        let columns = terminal.columns();
        let lines = terminal.screen_lines();
        let display_offset = terminal.grid().display_offset();
        let cells = (0..lines)
            .flat_map(|line| {
                (0..columns).map(move |column| capture_cell(terminal, display_offset, line, column))
            })
            .collect();

        Self {
            columns: dimension(columns),
            lines: dimension(lines),
            cursor: capture_cursor(terminal, display_offset),
            damage: TerminalRenderDamage::Full(cells),
        }
    }

    fn capture_partial<T: EventListener>(
        terminal: &Term<T>,
        damaged_lines: Vec<LineDamageBounds>,
    ) -> Self {
        let columns = terminal.columns();
        let lines = terminal.screen_lines();
        let display_offset = terminal.grid().display_offset();
        let spans = damaged_lines
            .into_iter()
            .filter_map(|damage| {
                if damage.line >= lines || damage.left >= columns {
                    return None;
                }

                let right = damage.right.min(columns - 1);
                let cells = (damage.left..=right)
                    .map(|column| capture_cell(terminal, display_offset, damage.line, column))
                    .collect();

                Some(TerminalRenderSpan {
                    line: dimension(damage.line),
                    column: dimension(damage.left),
                    cells,
                })
            })
            .collect();

        Self {
            columns: dimension(columns),
            lines: dimension(lines),
            cursor: capture_cursor(terminal, display_offset),
            damage: TerminalRenderDamage::Partial(spans),
        }
    }
}

#[derive(Debug)]
enum TerminalRenderDamage {
    Full(Vec<TerminalCell>),
    Partial(Vec<TerminalRenderSpan>),
}

#[derive(Debug)]
struct TerminalRenderSpan {
    line: u16,
    column: u16,
    cells: Vec<TerminalCell>,
}

enum CapturedDamage {
    Full,
    Partial(Vec<LineDamageBounds>),
}

fn capture_cell<T>(
    terminal: &Term<T>,
    display_offset: usize,
    line: usize,
    column: usize,
) -> TerminalCell {
    let point = Point::new(Line(line as i32 - display_offset as i32), Column(column));
    TerminalCell::capture(&terminal.grid()[point])
}

fn capture_cursor<T: EventListener>(
    terminal: &Term<T>,
    display_offset: usize,
) -> Option<TerminalCursor> {
    let cursor = terminal.renderable_content().cursor;
    let shape = match cursor.shape {
        CursorShape::Block => TerminalCursorShape::Block,
        CursorShape::Underline => TerminalCursorShape::Underline,
        CursorShape::Beam => TerminalCursorShape::Beam,
        CursorShape::HollowBlock => TerminalCursorShape::HollowBlock,
        CursorShape::Hidden => return None,
    };
    let line = cursor.point.line.0 + display_offset as i32;
    let Ok(line) = usize::try_from(line) else {
        return None;
    };

    Some(TerminalCursor {
        line: dimension(line),
        column: dimension(cursor.point.column.0),
        shape,
    })
}

fn dimension(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}
