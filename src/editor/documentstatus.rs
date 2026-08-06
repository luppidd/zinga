#[derive(Clone, Default, Debug)]
pub struct DocumentStatus {
    pub lines: usize,
    pub current_line: usize,
    pub is_modified: bool,
    pub file_name: Option<String>,
}
