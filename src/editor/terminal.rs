use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::Print;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::{queue, Command};
use std::io::{stdout, Error, Write};

#[derive(Copy, Clone, Default)]
pub struct Size {
    pub height: usize,
    pub width: usize,
}
#[derive(Copy, Clone, Default)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}
/// Represents the Terminal.
/// Edge Case for platforms where `usize` < `u16`:
/// Regardless of the actual size of the Terminal, this representation
/// only spans over at most `usize::MAX` or `u16::size` rows/columns, whichever is smaller.
/// Each size returned truncates to min(`usize::MAX`, `u16::MAX`)
/// And should you attempt to set the caret out of these bounds, it will also be truncated.
pub struct Terminal;

impl Terminal {
    pub fn terminate() -> Result<(), Error> {
        Self::execute();
        Self::leave_alternate_screeen();
        disable_raw_mode()?;
        Ok(())
    }
    pub fn initialize() -> Result<(), Error> {
        enable_raw_mode()?;
        Self::enter_alternate_screen();
        Self::clear_screen()?;
        Self::execute();
        Ok(())
    }
    pub fn clear_screen() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::All));
        Ok(())
    }
    pub fn clear_line() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::CurrentLine));
        Ok(())
    }
    /// Moves the caret to the given Position.
    /// # Arguments
    /// * `Position` - the  `Position`to move the caret to. Will be truncated to `u16::MAX` if bigger.
    pub fn move_caret_to(position: Position) -> Result<(), Error> {
        // clippy::as_conversions: See doc above
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        Self::queue_command(MoveTo(position.col as u16, position.row as u16));
        Ok(())
    }
    pub fn hide_caret() {
        Self::queue_command(Hide);
    }
    pub fn show_caret() {
        Self::queue_command(Show);
    }

    pub fn enter_alternate_screen() {
        Self::queue_command(EnterAlternateScreen);
    }

    pub fn leave_alternate_screeen() {
        Self::queue_command(LeaveAlternateScreen);
    }
    pub fn print(string: &str) {
        Self::queue_command(Print(string));
    }

    pub fn print_row(row: usize, line_text: &str) {
        Self::move_caret_to(Position { col: 0, row });
        Self::clear_line();
        Self::print(line_text);
    }

    /// Returns the current size of this Terminal.
    /// Edge Case for systems with `usize` < `u16`:
    /// * A `Size` representing the terminal size. Any coordinate `z` truncated to `usize` if `usize` < `z` < `u16`
    pub fn size() -> Result<Size, Error> {
        let (width_u16, height_u16) = size()?;
        // clippy::as_conversions: See doc above
        #[allow(clippy::as_conversions)]
        let height = height_u16 as usize;
        // clippy::as_conversions: See doc above
        #[allow(clippy::as_conversions)]
        let width = width_u16 as usize;
        Ok(Size { height, width })
    }

    pub fn execute() {
        let _ = stdout().flush();
    }

    fn queue_command<T: Command>(command: T) {
        let _ = queue!(stdout(), command);
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
    }
}
