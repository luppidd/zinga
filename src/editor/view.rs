use super::{
    editorcommand::{Direction, EditorCommand},
    terminal::{Position, Size, Terminal},
};
use std::io::Error;

mod buffer;
mod line;
mod location;
use buffer::Buffer;
use location::Location;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    // The location of the cursor relative to the start of the document
    location: Location,
    // The location of the screen relative to the start of the document
    scroll_offset: Location,
}

impl View {
    // Returns the position of the caret on the screen
    pub fn get_postion(&self) -> Position {
        self.location.subtract(&self.scroll_offset).into()
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

        for current_row in 0..height {
            if let Some(line) = self.buffer.lines.get(current_row) {
                let truncated_line = if line.len() >= width {
                    &line[0..width]
                } else {
                    line
                };
                Terminal::print_row(current_row, truncated_line);
            } else if current_row == vertical_center && self.buffer.is_empty() {
                Terminal::print_row(current_row, &Self::build_welcome_message(width));
            } else {
                Terminal::print_row(current_row, "~");
            }
        }
        self.needs_redraw = false;
        Ok(())
    }

    fn move_point(&mut self, direction: &Direction) {
        let Location { x: mut x, y: mut y } = self.location;
        let size = self.size;

        match direction {
            Direction::Up => y = y.saturating_sub(1),
            Direction::Down => y = y.saturating_add(1),
            Direction::Left => x = x.saturating_sub(1),
            Direction::Right => x = x.saturating_add(1),
            Direction::PageUp => y = y.saturating_sub(size.height),
            Direction::PageDown => y = y.saturating_add(size.height),
            Direction::Home => x = 0,
            Direction::End => x = size.width.saturating_sub(1),
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

    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Move(direction) => self.move_point(&direction),
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Quit => {}
        }
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

    pub fn load(&mut self, file_name: &str) {
        if let Ok(buffer) = Buffer::load(file_name) {
            self.buffer = buffer;
            self.needs_redraw = true;
        }
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
