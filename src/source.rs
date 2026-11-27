use crate::token::Span;

/// Maps UTF-8 byte offsets to one-based source locations.
///
/// Spans are always half-open byte ranges: [start, end). Line and column
/// values are derived for display and are never used as canonical offsets.
#[derive(Debug, Clone)]
pub struct SourceMap<'a> {
    source: &'a str,
    line_starts: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    /// One-based Unicode scalar-value column (not terminal display width).
    pub column: usize,
    /// Zero-based UTF-8 byte offset from the start of the line.
    pub byte_column: usize,
}

impl<'a> SourceMap<'a> {
    pub fn new(source: &'a str) -> Self {
        let bytes = source.as_bytes();
        let mut line_starts = vec![0];

        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'\r' if bytes.get(i + 1) == Some(&b'\n') => {
                    line_starts.push(i + 2);
                    i += 2;
                }
                b'\n' | b'\r' => {
                    line_starts.push(i + 1);
                    i += 1;
                }
                _ => i += 1,
            }
        }

        Self { source, line_starts }
    }

    pub fn source(&self) -> &'a str {
        self.source
    }

    pub fn location(&self, byte_offset: usize) -> Option<SourceLocation> {
        if byte_offset > self.source.len() || !self.source.is_char_boundary(byte_offset) {
            return None;
        }

        let line_index = self.line_starts.partition_point(|&start| start <= byte_offset) - 1;
        let line_start = self.line_starts[line_index];
        let prefix = &self.source[line_start..byte_offset];

        Some(SourceLocation {
            line: line_index + 1,
            column: prefix.chars().count() + 1,
            byte_column: byte_offset - line_start,
        })
    }

    pub fn line_text(&self, one_based_line: usize) -> Option<&'a str> {
        if one_based_line == 0 || one_based_line > self.line_starts.len() {
            return None;
        }

        let start = self.line_starts[one_based_line - 1];
        let end = self.line_starts.get(one_based_line).copied().unwrap_or(self.source.len());
        let text = &self.source[start..end];
        Some(text.trim_end_matches(['\\r', '\\n']))
    }

    pub fn slice(&self, span: Span) -> Option<&'a str> {
        if span.start > span.end
            || span.end > self.source.len()
            || !self.source.is_char_boundary(span.start)
            || !self.source.is_char_boundary(span.end)
        {
            return None;
        }
        self.source.get(span.start..span.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_utf8_offsets_and_multibyte_text() {
        let source = "a\nTiếng Việt";
        let map = SourceMap::new(source);
        let offset = source.find("Việt").unwrap();
        let location = map.location(offset).unwrap();

        assert_eq!(location.line, 2);
        assert_eq!(location.column, 7);
        assert_eq!(map.slice(Span { start: offset, end: source.len() }), Some("Việt"));
    }

    #[test]
    fn handles_crlf_and_bare_cr_line_endings() {
        let source = "one\r\ntwo\rthree";
        let map = SourceMap::new(source);

        assert_eq!(map.location(5).unwrap().line, 2);
        assert_eq!(map.location(9).unwrap().line, 3);
        assert_eq!(map.line_text(2), Some("two"));
    }

    #[test]
    fn rejects_non_boundary_and_out_of_range_offsets() {
        let source = "ế";
        let map = SourceMap::new(source);

        assert!(map.location(1).is_none());
        assert!(map.location(3).is_none());
    }

    #[test]
    fn handles_combining_marks_without_corrupting_byte_offsets() {
        let source = "e\u{301}x";
        let map = SourceMap::new(source);
        let x = source.find('x').unwrap();

        assert_eq!(map.location(x).unwrap().column, 3);
        assert_eq!(map.slice(Span { start: x, end: x + 1 }), Some("x"));
    }
}
