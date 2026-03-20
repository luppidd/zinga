use std::cmp::min;

use super::{
    editorcommand::{Direction, EditorCommand},
    terminal::{Position, Size, Terminal},
};
use std::io::Error;

mod buffer;
mod line;
use buffer::Buffer;
use line::Line;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default, Copy, Clone)]
pub struct Location {
    pub grapheme_index: usize,
    pub line_index: usize,
}

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    location: Location,
    scroll_offset: Position,
}

impl View {
    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Move(direction) => self.move_caret(&direction),
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Quit => {}
        }
    }

    pub fn text_location_to_position(&self) -> Position {
        let row = self.location.line_index;
        let line = self.buffer.lines.get(row).unwrap();
        let text_width = line.width_until(self.location.grapheme_index);

        Position {
            col: text_width,
            row,
        }
    }

    pub fn snap_to_valid_grapheme(&mut self) {
        let col = self.location.grapheme_index;
        let line = self.buffer.lines.get(self.location.line_index);
        self.location.grapheme_index = match line {
            Some(x) => min(x.length(), col),
            None => 0,
        }
    }

    pub fn snap_to_valid_line(&mut self) {
        self.location.line_index = min(self.location.line_index, self.buffer.lines.len())
    }

    fn render_line(at: usize, line_text: &str) {
        Terminal::print_row(at, line_text);
    }

    pub fn load(&mut self, file_name: &str) {
        if let Ok(buffer) = Buffer::load(file_name) {
            self.buffer = buffer;
            self.needs_redraw = true;
        }
    }

    pub fn resize(&mut self, to: Size) {
        self.size = to;
        self.scroll_view_location();
        self.needs_redraw = true;
    }

    pub fn render(&mut self) -> Result<(), Error> {
        if !self.needs_redraw {
            return Ok(());
        }
        let Size { height, width } = self.size;
        if height == 0 || width == 0 {
            return Ok(());
        }
        #[allow(clippy::integer_division)]
        let vertical_center = height / 3;
        let top = self.scroll_offset.row;

        for current_row in 0..height {
            if let Some(line) = self.buffer.lines.get(current_row.saturating_add(top)) {
                let left = self.scroll_offset.col;
                let right = self.scroll_offset.col.saturating_add(width);
                Self::render_line(current_row, &line.get(left..right));
            } else if current_row == vertical_center && self.buffer.is_empty() {
                Self::render_line(current_row, &Self::build_welcome_message(width));
            } else {
                Self::render_line(current_row, "~");
            }
        }
        self.needs_redraw = false;
        Ok(())
    }

    fn move_to_end_of_line(&mut self) {
        let width = self
            .buffer
            .lines
            .get(self.location.grapheme_index)
            .map_or(0, Line::length);
        self.location.grapheme_index = width;
    }

    fn move_to_start_of_line(&mut self) {
        self.location.grapheme_index = 0;
    }

    fn move_up(&mut self, distance: usize) {
        let mut line_index: usize = self.location.line_index;
        line_index = line_index.saturating_sub(distance);
        self.location.line_index = line_index;
        self.snap_to_valid_grapheme();
    }

    fn move_down(&mut self, distance: usize) {
        let line_index = &mut self.location.line_index;
        *line_index = line_index.saturating_add(distance);
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }

    fn move_left(&mut self, distance: usize) {
        let width = &self.location.grapheme_index;
        let mut x = distance;
        if *width < distance {
            x = x.saturating_sub(*width);
            self.move_up(1);
            self.move_to_end_of_line();
        }
        self.location.grapheme_index = self.location.grapheme_index.saturating_sub(x);
        self.snap_to_valid_grapheme();
    }

    // Stepwise move to support further move commands like vim in the future
    fn move_right(&mut self, distance: usize) {
        let Location {
            mut grapheme_index,
            line_index,
        } = self.location;

        let line_size = self.buffer.lines.get(line_index).map_or(0, Line::length);

        if grapheme_index > line_size {
            self.move_to_start_of_line();
            self.move_down(1);
        } else {
            grapheme_index = grapheme_index.saturating_sub(distance);
            self.location.grapheme_index = grapheme_index;
        }
        self.snap_to_valid_grapheme();
    }

    fn move_caret(&mut self, direction: &Direction) {
        let height = self.size.height;
        match direction {
            Direction::Up => self.move_up(1),
            Direction::Down => self.move_down(1),
            Direction::Left => self.move_left(1),
            Direction::Right => self.move_right(1),
            Direction::PageUp => self.move_up(height.saturating_sub(1)),
            Direction::PageDown => self.move_down(height.saturating_sub(1)),
            Direction::Home => self.move_to_start_of_line(),
            Direction::End => self.move_to_end_of_line(),
        }

        self.scroll_view_location();
    }

    fn scroll_view_location(&mut self) {
        let Position { col, row } = self.text_location_to_position();
        let mut offset_changed = false;

        if self.scroll_offset.row == row {
        } else {
            offset_changed = true;
        }
        if self.scroll_offset.col == col {
        } else {
            offset_changed = true;
        }

        self.needs_redraw = offset_changed || self.needs_redraw;
    }

    fn build_welcome_message(width: usize) -> String {
        if width == 0 {
            return " ".to_string();
        }
        let welcome_message = format!("{NAME} editor -- version {VERSION}");
        let len = welcome_message.len();
        if width <= len {
            return "~".to_string();
        }
        // we allow this since we don't care if our welcome message is put _exactly_ in the middle.
        // it's allowed to be a bit to the left or right.
        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(len).saturating_sub(1)) / 2;

        let mut full_message = format!("~{}{}", " ".repeat(padding), welcome_message);
        full_message.truncate(width);
        full_message
    }
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::size().unwrap_or_default(),
            location: Location::default(),
            scroll_offset: Position::default(),
        }
    }
}
