use crate::editor::Size;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::convert::TryFrom;

pub enum Direction {
    PageUp,
    PageDown,
    Home,
    End,
    Up,
    Left,
    Right,
    Down,
}

pub enum EditorCommand {
    Move(Direction),
    Resize(Size),
    Insert(char),
    // Implement Enter before Delete, basically an implementation where multiple Lines are modified
    // by the buffer.
    // Think about the methods that need to be implemented here.
    Enter,
    Delete,
    Backspace,
    Quit,
}

// We're basically converting Events into editor commands here

impl TryFrom<Event> for EditorCommand {
    type Error = String;
    fn try_from(event: Event) -> Result<Self, String> {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match (code, modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => Ok(Self::Quit),
                (KeyCode::Char(character), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                    Ok(Self::Insert(character))
                }
                (KeyCode::Enter, _) => Ok(Self::Enter),
                (KeyCode::Backspace, _) => Ok(Self::Backspace),
                (KeyCode::Delete, _) => Ok(Self::Delete),
                (KeyCode::Up, _) => Ok(Self::Move(Direction::Up)),
                (KeyCode::Left, _) => Ok(Self::Move(Direction::Left)),
                (KeyCode::Right, _) => Ok(Self::Move(Direction::Right)),
                (KeyCode::Down, _) => Ok(Self::Move(Direction::Down)),
                (KeyCode::PageUp, _) => Ok(Self::Move(Direction::PageUp)),
                (KeyCode::PageDown, _) => Ok(Self::Move(Direction::PageDown)),
                (KeyCode::End, _) => Ok(Self::Move(Direction::End)),
                (KeyCode::Home, _) => Ok(Self::Move(Direction::Home)),

                _ => Err(format!("Key Code not supported: {code:?}")),
            },
            Event::FocusGained => Err(format!("FocusGained not supported")),
            Event::FocusLost => Err(format!("FocusLost not supported")),
            Event::Paste(character) => Err(format!("Paste not suported:{character:?}")),
            Event::Resize(width_u16, height_u16) => {
                let height = height_u16 as usize;
                let width = width_u16 as usize;
                Ok(Self::Resize(Size { height, width }))
            }
            Event::Mouse(_mouse_event) => Err(format!("Mouse event not supported")),
        }
    }
}
