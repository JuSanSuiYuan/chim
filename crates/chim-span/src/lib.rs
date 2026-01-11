use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpanId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub file_id: FileId,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(file_id: FileId, start: usize, end: usize, line: usize, column: usize) -> Self {
        Span { file_id, start, end, line, column }
    }

    pub fn is_valid(&self) -> bool {
        self.start <= self.end && self.file_id.0 != usize::MAX
    }

    pub fn contains(&self, pos: usize) -> bool {
        pos >= self.start && pos <= self.end
    }

    pub fn merge(&self, other: &Span) -> Span {
        let start = self.start.min(other.start);
        let end = self.end.max(other.end);
        let line = if self.start <= other.start { self.line } else { other.line };
        let column = if self.start <= other.start { self.column } else { other.column };
        Span::new(self.file_id, start, end, line, column)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file_id.0, self.line + 1, self.column + 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    id: FileId,
    name: Arc<str>,
    content: Arc<str>,
    line_offsets: Vec<usize>,
}

impl SourceFile {
    pub fn new(id: FileId, name: Arc<str>, content: Arc<str>) -> Self {
        let line_offsets = Self::compute_line_offsets(&content);
        SourceFile { id, name, content, line_offsets }
    }

    fn compute_line_offsets(content: &str) -> Vec<usize> {
        let mut offsets = vec![0];
        for (idx, _) in content.char_indices() {
            if content[idx..].starts_with('\n') {
                offsets.push(idx + 1);
            }
        }
        offsets.push(content.len());
        offsets
    }

    pub fn id(&self) -> FileId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn line_offset(&self, line: usize) -> Option<usize> {
        self.line_offsets.get(line).copied()
    }

    pub fn line_count(&self) -> usize {
        self.line_offsets.len().saturating_sub(1)
    }

    pub fn location(&self, byte_pos: usize) -> Option<(usize, usize)> {
        let line = self.line_offsets.partition_point(|&offset| offset <= byte_pos).saturating_sub(1);
        let line_start = self.line_offsets[line];
        let column = byte_pos - line_start;
        Some((line + 1, column + 1))
    }

    pub fn snippet(&self, span: &Span) -> Option<&str> {
        if span.start < self.content.len() && span.end <= self.content.len() {
            Some(&self.content[span.start..span.end])
        } else {
            None
        }
    }

    pub fn snippet_with_context(&self, span: &Span, context_lines: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let start_line = span.line.saturating_sub(context_lines);
        let end_line = (span.line + context_lines).min(self.line_count());

        for line in start_line..=end_line {
            if let Some(offset) = self.line_offset(line.saturating_sub(1)) {
                let line_content = self.content[offset..]
                    .lines()
                    .next()
                    .unwrap_or("");
                let marker = if line == span.line {
                    format!("> {} | {}", line + 1, line_content)
                } else {
                    format!("  {} | {}", line + 1, line_content)
                };
                lines.push(marker);
            }
        }

        lines
    }
}

#[derive(Debug, Default)]
pub struct SourceMap {
    files: HashMap<FileId, SourceFile>,
    file_names: HashMap<Arc<str>, FileId>,
    next_id: usize,
}

impl SourceMap {
    pub fn new() -> Self {
        SourceMap::default()
    }

    pub fn add_file(&mut self, name: Arc<str>, content: Arc<str>) -> FileId {
        let id = FileId(self.next_id);
        self.next_id += 1;
        let file = SourceFile::new(id, name.clone(), content);
        self.files.insert(id, file);
        self.file_names.insert(name, id);
        id
    }

    pub fn get_file(&self, id: FileId) -> Option<&SourceFile> {
        self.files.get(&id)
    }

    pub fn get_file_by_name(&self, name: &str) -> Option<&SourceFile> {
        self.file_names.get(name).and_then(|id| self.files.get(id))
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_contains() {
        let span = Span::new(FileId(0), 10, 20, 1, 10);
        assert!(span.contains(10));
        assert!(span.contains(15));
        assert!(span.contains(20));
        assert!(!span.contains(9));
        assert!(!span.contains(21));
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(FileId(0), 10, 20, 1, 10);
        let span2 = Span::new(FileId(0), 15, 30, 2, 5);
        let merged = span1.merge(&span2);
        assert_eq!(merged.start, 10);
        assert_eq!(merged.end, 30);
    }

    #[test]
    fn test_source_file_line_count() {
        let content = "line1\nline2\nline3\n";
        let file = SourceFile::new(FileId(0), "test".into(), content.into());
        assert_eq!(file.line_count(), 3);
    }

    #[test]
    fn test_source_file_location() {
        let content = "line1\nline2\nline3\n";
        let file = SourceFile::new(FileId(0), "test".into(), content.into());
        assert_eq!(file.location(0), Some((1, 1)));
        assert_eq!(file.location(6), Some((2, 1)));
        assert_eq!(file.location(12), Some((3, 1)));
    }
}
