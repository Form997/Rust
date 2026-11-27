use crate::{source::SourceMap, token::Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
    pub span: Option<Span>,
}

impl Diagnostic {
    pub fn error(code: &'static str, message: impl Into<String>, span: Option<Span>) -> Self {
        Self { code, severity: Severity::Error, message: message.into(), span }
    }

    pub fn render(&self, source_map: &SourceMap<'_>) -> String {
        let label = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        };

        let mut output = format!("{label}[{}]: {}", self.code, self.message);
        if let Some(span) = self.span {
            if let Some(location) = source_map.location(span.start) {
                output.push_str(&format!("\n --> {}:{}", location.line, location.column));
                if let Some(line) = source_map.line_text(location.line) {
                    output.push_str(&format!("\n  |\n{:>2} | {}", location.line, line));
                    output.push_str(&format!("\n  | {}^", " ".repeat(location.column.saturating_sub(1))));
                }
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_location_for_unicode_source() {
        let source = "OUTPUT tên\n";
        let map = SourceMap::new(source);
        let start = source.find("tên").unwrap();
        let diagnostic = Diagnostic::error(
            "AL1001",
            "unknown name",
            Some(Span { start, end: start + "tên".len() }),
        );

        let rendered = diagnostic.render(&map);
        assert!(rendered.contains(" --> 1:8"));
        assert!(rendered.contains("error[AL1001]: unknown name"));
    }
}
