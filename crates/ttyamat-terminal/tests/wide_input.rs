use alacritty_terminal::{
    event::VoidListener,
    grid::Dimensions,
    term::{Config, Term},
    vte::ansi::Processor,
};
use ttyamat_terminal::TerminalSize;

struct TestDimensions(TerminalSize);

impl Dimensions for TestDimensions {
    fn total_lines(&self) -> usize {
        usize::from(self.0.lines())
    }

    fn screen_lines(&self) -> usize {
        usize::from(self.0.lines())
    }

    fn columns(&self) -> usize {
        usize::from(self.0.columns())
    }
}

#[test]
fn accepted_narrow_sizes_can_parse_wide_text_after_resize() {
    for columns in [2, 1] {
        // A rejected size cannot reach the backend. Two columns must stay valid.
        let Some(narrow) = TerminalSize::new(columns, 24, 8, 16) else {
            assert_eq!(columns, 1, "two columns must remain supported");
            continue;
        };

        let normal = TerminalSize::new(80, 24, 8, 16).unwrap();
        let mut terminal = Term::new(Config::default(), &TestDimensions(normal), VoidListener);
        let mut parser: Processor = Processor::new();

        terminal.resize(TestDimensions(narrow));
        parser.advance(&mut terminal, "\u{754c}".as_bytes());
    }
}
