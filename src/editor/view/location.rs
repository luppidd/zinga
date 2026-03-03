use crate::editor::terminal::Position;

#[derive(Default, Copy, Clone)]
pub struct Location {
    pub grapheme_index: usize,
    pub line_index: usize,
}

impl From<Location> for Position {
    fn from(loc: Location) -> Self {
        Self {
            col: loc.grapheme_index,
            row: loc.line_index,
        }
    }
}

impl Location {
    pub const fn subtract(&self, other: &Self) -> Self {
        Self {
            grapheme_index: self.grapheme_index.saturating_sub(other.grapheme_index),
            line_index: self.line_index.saturating_sub(other.line_index),
        }
    }
}
