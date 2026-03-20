use std::{cmp, ops::Range};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[repr(usize)]
enum GraphemeWidth {
    Half = 1,
    Full = 2,
}

impl GraphemeWidth {
    fn get_replacement(&self) -> Option<char> {
        match self {
            Self::Half => Some('.'),
            Self::Full => None,
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
struct TextFragment {
    string: String,
    rendered_width: GraphemeWidth,
    replacement: Option<char>,
}

impl TextFragment {}

#[derive(Default)]
pub struct Line {
    fragments: Vec<TextFragment>,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        let graphemes = line_str.graphemes(true).collect::<Vec<&str>>();
        let mut fragments = vec![];

        for text_fragment in graphemes {
            let width = text_fragment.width();

            let width = GraphemeWidth::try_from(width).expect("Invalid conversion of width");
            let replacement = width.get_replacement();

            let fragment = TextFragment {
                string: text_fragment.to_string(),
                rendered_width: width,
                replacement,
            };

            fragments.push(fragment);
        }
        Line { fragments }
    }

    pub fn get(&self, range: Range<usize>) -> String {
        let start = range.start;
        let end = cmp::min(range.end, self.fragments.len());
        let graphemes = self.fragments.get(start..end).unwrap_or_default();

        graphemes.iter().map(|x| x.string.clone()).collect()
    }

    pub fn length(&self) -> usize {
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
