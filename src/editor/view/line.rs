use std::ops::Range; // Cool that Rust has a range type in std
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[repr(usize)]
enum GraphemeWidth {
    Half = 1,
    Full = 2,
}

impl GraphemeWidth {
    // Example doesn't borrow here but we don't want to take ownership when we take this
    // and use it in other calculations
    const fn saturating_add(&self, other: usize) -> usize {
        match self {
            GraphemeWidth::Half => other.saturating_add(1),
            GraphemeWidth::Full => other.saturating_add(2),
        }
    }
}

impl TryFrom<usize> for GraphemeWidth {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 | 1 => Ok(GraphemeWidth::Half),
            _ => Ok(GraphemeWidth::Full),
        }
    }
}

#[allow(dead_code)]
pub struct TextFragment {
    grapheme: String,
    rendered_width: GraphemeWidth,
    replacement: Option<char>,
}

impl TextFragment {
    pub fn new(fragment: &str) -> Self {
        let unicode_width = fragment.width(); // This function comes from UnicodeWidthStr
        let replacement = Self::replacement_for(fragment);
        let rendered_width = match replacement {
            Some(ch) => GraphemeWidth::try_from(ch.to_string().width()).unwrap(),
            None => GraphemeWidth::try_from(unicode_width).unwrap(),
        };

        let grapheme = fragment.to_string(); // Convert &str to String

        Self {
            grapheme,
            rendered_width,
            replacement,
        }
    }

    fn replacement_for(text: &str) -> Option<char> {
        let unicode_width = text.width();
        // Some review on syntax note that chars are represented using single quotations
        // but string srae represented as double quoted characters
        match text {
            " " => None,
            "\t" => Some(' '), // Replace tab with space

            // Note this new pattern for match arms, this is plagiarized right from the demo
            // Can use a guard which basically uses an if condition, the _ means we are ignoring
            // the item that we are matching for.
            // https://doc.rust-lang.org/rust-by-example/flow_control/match/guard.html
            // Non-zero whitespaces we don't want to see so we replace them with boxes
            _ if unicode_width > 0 && text.trim().is_empty() => Some('␣'),
            _ if unicode_width == 0 => {
                let mut chars = text.chars();
                if let Some(ch) = chars.next() {
                    if ch.is_control() && chars.next().is_none() {
                        return Some('▯');
                    }
                }
                Some('·')
            }
            _ => None,
        }
    }
}

impl TryFrom<&str> for TextFragment {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(TextFragment::new(value))
    }
}

#[derive(Default)]
pub struct Line {
    fragments: Vec<TextFragment>,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        Line {
            fragments: Self::str_to_fragments(line_str),
        }
    }

    pub fn str_to_fragments(line_str: &str) -> Vec<TextFragment> {
        let graphemes = line_str.graphemes(true).collect::<Vec<&str>>();
        let mut fragments = vec![];

        for text_fragment in graphemes {
            let fragment = TextFragment::try_from(text_fragment).unwrap();
            fragments.push(fragment);
        }
        fragments
    }

    pub fn insert_char(&mut self, character: char, grapheme_index: usize) {
        let mut new_string = String::new();

        for (index, fragment) in self.fragments.iter().enumerate() {
            if index == grapheme_index {
                new_string.push(character);
            }
            new_string.push_str(&fragment.grapheme);
        }
        if grapheme_index >= self.fragments.len() {
            new_string.push(character);
        }

        // We have to re-evaluate identifiable fragments in the the string everytime
        // we have to edit the string otherwise we can run into funky scenarios where
        // we don't render complex graphemes correctly as we are editing text
        self.fragments = Self::str_to_fragments(&new_string)
    }

    // used to manipulate text fragments in the buffer
    pub fn get_fragments(&self, range: Range<usize>) -> String {
        // if we are trying to get the fragment outsie of bounds return an empty string
        if range.start >= range.end {
            return String::new();
        }

        if range.start >= self.fragments.len() {
            return String::new();
        };

        let mut result = String::new();
        let mut current_position = 0;

        for fragment in self.fragments.iter() {
            if current_position > range.end {
                break;
            }

            if current_position >= range.start {
                result.push_str(&fragment.grapheme);
            }

            current_position = current_position.saturating_add(1);
        }

        result
    }

    pub fn delete_char(&mut self, grapheme_index: usize) {
        // Invariant here, I don't want to create a new vec or nothing
        // Plus I'll let view move the char left anyways
        if grapheme_index > self.fragments.len() {
            return;
        }

        let mut new_string = String::new();

        for (index, fragment) in self.fragments.iter().enumerate() {
            if index == grapheme_index {
                continue; //skip the current character
            }
            new_string.push_str(&fragment.grapheme);
        }
        self.fragments = Self::str_to_fragments(&new_string)
    }

    // used in the display of the editor
    pub fn get_display_graphemes(&self, range: Range<usize>) -> String {
        if range.start >= range.end {
            return String::new();
        }

        let mut result = String::new();
        let mut current_position = 0;

        // We now have graphemes that we aren't going to display and they will have rendered widths
        // of 0,1, or 2. For these zero space widths we are replacing with a 1 spcae character '.'
        //
        for fragment in &self.fragments {
            let fragment_end = fragment.rendered_width.saturating_add(current_position);

            if current_position >= range.end {
                // If we go past the right most bound while iterating then we exit
                break;
            }
            if fragment_end > range.start {
                if fragment_end > range.end || current_position < range.start {
                    // This will only occur in cases where we have characters at boundaries and
                    // they are more than 1 width in size -- in this case clip them and replace
                    // with -
                    //
                    result.push('-');
                } else if let Some(char) = fragment.replacement {
                    // if fragment has a replacement display impl
                    result.push(char);
                } else {
                    result.push_str(&fragment.grapheme);
                }
            }

            current_position = fragment_end
        }
        result
    }

    pub fn len(&self) -> usize {
        self.fragments.len()
    }

    pub fn width_until(&self, grapheme_index: usize) -> usize {
        // new function and taking implementation from tutorial to learn about iterators and their
        // functions
        self.fragments
            .iter()
            .take(grapheme_index)
            .map(|fragment| match fragment.rendered_width {
                GraphemeWidth::Half => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }
}
