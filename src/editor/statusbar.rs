use super::{documentstatus, terminal};
use documentstatus::DocumentStatus;
use terminal::{Size, Terminal};

#[derive(Default)]
pub struct StatusBar {
    status: DocumentStatus,
    needs_redraw: bool,
    margin_bottom: usize,
    width: usize,
    position_y: usize,
}

impl StatusBar {
    pub fn new(margin_bottom: usize) -> Self {
        let size = Terminal::size().unwrap_or_default();
        Self {
            status: DocumentStatus::default(),
            needs_redraw: false,
            margin_bottom: margin_bottom,
            width: size.width,
            position_y: size.height.saturating_sub(margin_bottom).saturating_sub(1),
        }
    }

    pub fn resize(&mut self, size: Size) {
        self.width = size.width;
        self.position_y = size.height.saturating_sub(self.margin_bottom);
        self.needs_redraw = true;
    }
    pub fn update_status(&mut self, new_status: DocumentStatus) {
        self.status = new_status;
        self.needs_redraw = true;
    }
    // renders the text to be displayed on the terminal
    pub fn render(&mut self) {
        // Interesting inversion observed in code example
        if !self.needs_redraw {
            return;
        }
        // format is implemented with the Debug macro
        let mut status: String = format!("{:?}", self.status);
        // Formats the string slice
        // Default method for String also available to vectors.
        // Lets user keep only the items from the beginning of the vector to the specified length
        status.truncate(self.width);
        let result = Terminal::print_row(self.position_y, &status);
        debug_assert!(result.is_ok(), "Failed to render status bar");
        self.needs_redraw = false;
    }
}
