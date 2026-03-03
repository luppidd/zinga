use super::{
    editorcommand::{Direction, EditorCommand},
    terminal::{Position, Size, Terminal},
};
use std::io::Error;

mod buffer;
mod line;
mod location;

use buffer::Buffer;
use line::Line;
use location::Location;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    location: Location,
    scroll_offset: Location,
}

// Okay right now zinga works using single characters and dots for some graphemes. We've moved to
// using grapheme clusters.
//
// One we need to make sure that everytime we move the character we move by the factor of one
// grapheme. So at any point in time when we have a move command we need to fetch the existing
// grapheme index. Use width until to get the width of the characters from index i to index k and
// offset the caret by that width.
//
impl View {
    // Returns the position of the caret on the screen
    pub fn get_postion(&self) -> Position {
        self.location.subtract(&self.scroll_offset).into()
    }

    fn render_line(at: usize, line_text: &str) {
        Terminal::print_row(at, line_text);
    }

    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Move(direction) => self.move_point(&direction),
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Quit => {}
        }
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
        let top = self.scroll_offset.y;

        for current_row in 0..height {
            if let Some(line) = self.buffer.lines.get(current_row.saturating_add(top)) {
                let left = self.scroll_offset.x;
                let right = self.scroll_offset.x.saturating_add(width);
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

    fn move_point(&mut self, direction: &Direction) {
        let Location { mut x, mut y } = self.location;

        let Size { height, .. } = self.size;

        match direction {
            Direction::Up => y = y.saturating_sub(1),
            Direction::Down => y = y.saturating_add(1),

            Direction::Left => {
                if x > 0 {
                    x = x.saturating_sub(1)
                } else if y > 0 {
                    y -= 1;
                    let line = self.buffer.lines.get(y);
                    match line {
                        Some(line) => x = line.len(),
                        None => x = 0,
                    }
                }
            }

            Direction::Right => {
                if let Some(line) = self.buffer.lines.get(y) {
                    if x > line.len() {
                        y = y.saturating_add(1);
                        x = 0;
                    } else {
                        x = x.saturating_add(1);
                    }
                };
            }

            Direction::PageUp => y = y.saturating_sub(height - 1),
            Direction::PageDown => y = y.saturating_add(height - 1),
            Direction::Home => x = 0,
            Direction::End => x = self.buffer.lines.get(y).map_or(0, Line::len),
        }
        self.location = Location { x, y };

        self.scroll_view_location();
    }

    fn scroll_view_location(&mut self) {
        let Location { x, y } = self.location;
        let Size { width, height } = self.size;
        let Location {
            x: x_offset,
            y: y_offset,
        } = &mut self.scroll_offset;

        let mut offset_changed = true;

        // Let's handle vertical scrolling first
        //
        //I've borrowed the field scroll offset from self. I need to dereference to modify the
        //underlying location

        if y > y_offset.saturating_add(height) {
            *y_offset = y.saturating_sub(height);
        } else if y < *y_offset {
            *y_offset = y;
        } else {
            offset_changed = false;
        }

        // Let's handle horizontal scrolling

        if x > x_offset.saturating_add(width) {
            *x_offset = x_offset.saturating_sub(width);
        } else if x < *x_offset {
            *x_offset = y;
        } else {
            offset_changed = false
        }

        self.needs_redraw = offset_changed;
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
            scroll_offset: Location::default(),
        }
    }
}
