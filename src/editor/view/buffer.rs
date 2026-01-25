use std::fs::read_to_string;
use std::io::Error;

pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn load(file_name: &str) -> Result<Self, Error> {
        let contents = read_to_string(file_name)?;
        let mut lines = Vec::new();
        for value in contents.lines() {
            lines.push(String::from(value));
        }
        Ok(Self { lines })
    }
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn push_char(&mut self, character: char) {
        let size = self.lines.len();

        if let Some(current_line) = self.lines.get_mut(size - 1) {
            current_line.push(character);
        }
    }

    pub fn push_new_line(&mut self) {
        self.lines.push(String::new());
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            lines: vec!["".to_string()],
        }
    }
}
