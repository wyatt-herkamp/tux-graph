pub mod graphiz;
#[derive(Debug, Clone)]
pub(crate) struct FormattedStringBuilder {
    target: String,
    indent: usize,
}
impl FormattedStringBuilder {
    pub fn new(start: impl Into<String>, indent: usize) -> Self {
        Self {
            target: start.into(),
            indent,
        }
    }
    pub fn push(&mut self, line: impl AsRef<str>) {
        for _ in 0..self.indent {
            self.target.push(' ');
        }
        self.target.push_str(line.as_ref());
        self.target.push('\n');
    }
    pub fn push_no_indent(&mut self, line: impl AsRef<str>) {
        self.target.push_str(line.as_ref());
        self.target.push('\n');
    }
    pub fn finish(self) -> String {
        self.target
    }
}
