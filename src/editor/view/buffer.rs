use std::fs::read_to_string;
use std::io::Error;

use super::line::Line;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
}

impl Buffer {
    pub fn load_file(file_name: &str) -> Result<Self, Error> {
        let contents = read_to_string(file_name)?;
        let mut lines = Vec::new();
        for value in contents.lines() {
            lines.push(Line::from(value));
        }
        Ok(Self { lines })
    }

    pub fn is_empty(&self) -> bool {
        // Vec has a builtin is_empty
        self.lines.is_empty()
    }

    // The tutorial just used the Location structs instead of individual line_index, grapheme_index
    // parameters,
    //
    // not a big deal can refactor later.
    pub fn insert_char(&mut self, character: char, line_index: usize, grapheme_index: usize) {
        // if statements that contain let Some can have different conditions in the
        // other conditional arms that are not related to the option in the let statements
        // I first understood that if with let Some(T) conditions need to be tied to Options or
        // enums

        if let Some(line) = self.lines.get_mut(line_index) {
            line.insert_char(character, grapheme_index)
        } else if line_index == self.lines.len() {
            let mut line = Line::default();
            line.insert_char(character, 0);
            self.lines.push(line)
        }
    }

    // Inserts a line at line_index keeping fragments left of grapheme index in current line
    // and taking the fragments from the right inserting them into a new line
    pub fn insert_line(&mut self, line_index: usize, grapheme_index: usize) {
        // First boundary condition = early return
        if line_index >= self.lines.len() {
            let line = Line::default();
            self.lines.push(line);
            return;
        }

        if let Some(line) = self.lines.get(line_index) {
            let line_end_index = line.len().saturating_sub(1);
            if grapheme_index > line.len() {
                //Second boundary condition where the index passed is greater than the length of
                //the line we just insert a new empty line in the next line index position
                self.lines
                    .insert(line_index.saturating_add(1), Line::default());
                return;
            }

            let left_string = line.get_fragments(0..grapheme_index);
            let left = Line::from(&left_string);


            let right_string = line.get_fragments(grapheme_index..line_end_index);
            let right = Line::from(&right_string);

            self.lines[line_index] = left;
            self.lines.insert(line_index.saturating_add(1), right);
        }
    }

    pub fn delete_char(&mut self, line_index: usize, grapheme_index: usize) {
        // Guard condition
        //
        if let None = self.lines.get(line_index){
            return
        } else if let Some(line) = self.lines.get(line_index){
            // There's always a next line in this case and I can't do something like some next line 
            // because I need to take ownership via remove. Can't use get mut for that same reason.
            // IE I need to get mut lines and remove the next line and append the current line
            // independently.
            if grapheme_index >= line.len() && self.lines.len() > line_index.saturating_add(1){
                let next_line = self.lines.remove(line_index.saturating_add(1));
                let current_line = self.lines.get_mut(line_index).expect("Attemped to delete from a line out of bounds");
                current_line.append_other(next_line);
            } else if line.len() > grapheme_index {
                self.lines.get_mut(line_index).expect("Attemped to delete from a line out of bounds").delete_char(grapheme_index);
            }
        }

    }

}
