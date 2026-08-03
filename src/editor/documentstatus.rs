#[derive(Clone, Default, Debug)]
pub struct DocumentStatus {
    lines: usize,
    current_line: usize,
    is_modified: bool,
    file_name: Option<String>,
}
