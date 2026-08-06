use std::fs::read_to_string;
use std::io::{Error, ErrorKind};

use std::fs::File;
use std::io::Write;

use super::line::Line;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
    pub modified: bool,
    pub file_name: Option<String>,
}

impl Buffer {
    pub fn load_file(file_name: &str) -> Result<Self, Error> {
        let contents = read_to_string(file_name)?;
        let mut lines = Vec::new();
        for value in contents.lines() {
            lines.push(Line::try_from(value).unwrap());
        }
        Ok(Self {
            lines: lines,
            modified: false,
            file_name: Some(file_name.to_string()),
        })
    }

    pub fn save_file(&self, file_name: Option<String>) -> Result<(), Error> {
        let file_name = match file_name {
            Some(name) => name,
            None => self.file_name.clone().ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidData,
                    "Cannot save file - no filename passsed and no filename associated with the current buffer".to_string()
                    )
            })?
        };

        let mut file = File::create(file_name)?;

        for line in &self.lines {
            writeln!(file, "{line}")?;
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn insert_char(&mut self, character: char, line_index: usize, grapheme_index: usize) {
        if let Some(line) = self.lines.get_mut(line_index) {
            line.insert_char(character, grapheme_index)
        } else if line_index == self.lines.len() {
            let mut line = Line::default();
            line.insert_char(character, 0);
            self.lines.push(line)
        }
        self.modified = true;
    }

    pub fn insert_line(&mut self, line_index: usize, grapheme_index: usize) {
        if line_index >= self.lines.len() {
            let line = Line::default();
            self.lines.push(line);
            return;
        }

        if let Some(line) = self.lines.get(line_index) {
            let line_end_index = line.len().saturating_sub(1);
            if grapheme_index > line.len() {
                self.lines
                    .insert(line_index.saturating_add(1), Line::default());
                return;
            }

            let left_string = line.get_fragments(0..grapheme_index);
            let left = Line::try_from(left_string).unwrap();

            let right_string = line.get_fragments(grapheme_index..line_end_index);
            let right = Line::try_from(right_string).unwrap();

            self.lines[line_index] = left;
            self.lines.insert(line_index.saturating_add(1), right);
        }
        self.modified = true;
    }

    pub fn delete_char(&mut self, line_index: usize, grapheme_index: usize) {
        // Guard condition
        if let None = self.lines.get(line_index) {
            return;
        } else if let Some(line) = self.lines.get(line_index) {
            if grapheme_index >= line.len() && self.lines.len() > line_index.saturating_add(1) {
                let next_line = self.lines.remove(line_index.saturating_add(1));
                let current_line = self
                    .lines
                    .get_mut(line_index)
                    .expect("Attemped to delete from a line out of bounds");
                current_line.append_other(next_line);
            } else if line.len() > grapheme_index {
                self.lines
                    .get_mut(line_index)
                    .expect("Attemped to delete from a line out of bounds")
                    .delete_char(grapheme_index);
            }
        }
        self.modified = true;
    }
}
