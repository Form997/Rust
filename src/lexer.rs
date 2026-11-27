use crate::token::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    /// UTF-8 byte offset in the original source.
    pub position: usize,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lexical error at byte {}: {}", self.position, self.message)
    }
}

impl std::error::Error for LexError {}

pub struct Lexer<'a> {
    source: &'a str,
    chars: Vec<(usize, char)>,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().collect(),
            position: 0,
        }
    }

    fn byte_offset_at(&self, char_index: usize) -> usize {
        self.chars.get(char_index).map(|(byte, _)| *byte).unwrap_or(self.source.len())
    }

    fn byte_position(&self) -> usize {
        self.byte_offset_at(self.position)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.position).map(|(_, ch)| *ch)
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.position + 1).map(|(_, ch)| *ch)
    }

    fn advance(&mut self) -> Option<char> {
        let current = self.peek()?;
        self.position += 1;
        Some(current)
    }

    fn skip_horizontal_whitespace(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t')) {
            self.advance();
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        value
    }

    fn read_number(&mut self) -> String {
        let mut value = String::new();
        let mut seen_decimal_point = false;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                value.push(ch);
                self.advance();
            } else if ch == '.' && !seen_decimal_point {
                seen_decimal_point = true;
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        value
    }

    fn read_string(&mut self, start: usize) -> Result<String, LexError> {
        self.advance();
        let mut value = String::new();

        loop {
            match self.advance() {
                Some('"') => return Ok(value),
                Some('\\') => {
                    let escape_start = self.byte_offset_at(self.position - 1);
                    match self.advance() {
                        Some('n') => value.push('\n'),
                        Some('r') => value.push('\r'),
                        Some('t') => value.push('\t'),
                        Some('"') => value.push('"'),
                        Some('\\') => value.push('\\'),
                        Some(other) => return Err(LexError {
                            message: format!("unsupported escape sequence: \\{other}"),
                            position: escape_start,
                        }),
                        None => return Err(LexError {
                            message: "unterminated escape sequence".to_string(),
                            position: escape_start,
                        }),
                    }
                }
                Some('\n') | Some('\r') => return Err(LexError {
                    message: "newline inside string literal".to_string(),
                    position: self.byte_offset_at(self.position - 1),
                }),
                Some(ch) => value.push(ch),
                None => return Err(LexError {
                    message: "unterminated string literal".to_string(),
                    position: start,
                }),
            }
        }
    }

    fn keyword_or_identifier(word: String) -> TokenKind {
        match word.as_str() {
            "PROGRAM" => TokenKind::Program,
            "VERSION" => TokenKind::Version,
            "IMPORT" => TokenKind::Import,
            "DATA" => TokenKind::Data,
            "RULE" => TokenKind::Rule,
            "FUNCTION" => TokenKind::Function,
            "TASK" => TokenKind::Task,
            "OUTPUT" => TokenKind::Output,
            "END" => TokenKind::End,
            _ => TokenKind::Identifier(word),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        while self.position < self.chars.len() {
            self.skip_horizontal_whitespace();
            let Some(ch) = self.peek() else { break };
            let start = self.byte_position();

            let kind = match ch {
                '\n' => {
                    self.advance();
                    Some(TokenKind::Newline)
                }
                '\r' if self.peek_next() == Some('\n') => {
                    self.advance();
                    self.advance();
                    Some(TokenKind::Newline)
                }
                '\r' => {
                    self.advance();
                    Some(TokenKind::Newline)
                }
                ':' => {
                    self.advance();
                    Some(TokenKind::Colon)
                }
                '=' => {
                    self.advance();
                    Some(TokenKind::Equal)
                }
                '"' => Some(TokenKind::StringLiteral(self.read_string(start)?)),
                '/' if self.peek_next() == Some('/') => {
                    while let Some(current) = self.peek() {
                        if matches!(current, '\n' | '\r') { break; }
                        self.advance();
                    }
                    None
                }
                c if c.is_ascii_digit() => Some(TokenKind::Number(self.read_number())),
                c if c.is_ascii_alphabetic() || c == '_' => {
                    Some(Self::keyword_or_identifier(self.read_identifier()))
                }
                other => return Err(LexError {
                    message: format!("unexpected character {other:?}"),
                    position: start,
                }),
            };

            if let Some(kind) = kind {
                tokens.push(Token::new(kind, start, self.byte_position()));
            }
        }

        let eof = self.source.len();
        tokens.push(Token::new(TokenKind::Eof, eof, eof));
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenKind;

    #[test]
    fn recognizes_keywords_identifiers_and_values() {
        let source = r#"PROGRAM "Demo"
VERSION "0.1.0"
DATA:
    count: INTEGER = 12
END
OUTPUT count
"#;
        let tokens = Lexer::new(source).tokenize().expect("source should tokenize");
        assert_eq!(tokens[0].kind, TokenKind::Program);
        assert_eq!(tokens[1].kind, TokenKind::StringLiteral("Demo".into()));
        assert!(tokens.iter().any(|token| token.kind == TokenKind::Identifier("count".into())));
        assert!(tokens.iter().any(|token| token.kind == TokenKind::Number("12".into())));
        assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
    }

    #[test]
    fn token_spans_are_utf8_byte_offsets() {
        let source = "OUTPUT \"Tiếng Việt\"";
        let tokens = Lexer::new(source).tokenize().unwrap();
        let token = tokens.iter().find(|t| matches!(t.kind, TokenKind::StringLiteral(_))).unwrap();
        assert_eq!(&source[token.span.start..token.span.end], "\"Tiếng Việt\"");
    }

    #[test]
    fn decodes_string_escapes() {
        let tokens = Lexer::new(r#"OUTPUT "line\nnext""#).tokenize().expect("valid escape");
        assert_eq!(tokens[1].kind, TokenKind::StringLiteral("line\nnext".into()));
    }

    #[test]
    fn rejects_unterminated_strings() {
        let error = Lexer::new(r#"OUTPUT "unfinished"#).tokenize().expect_err("unterminated string must fail");
        assert!(error.message.contains("unterminated"));
    }

    #[test]
    fn skips_line_comments_but_keeps_newline() {
        let tokens = Lexer::new("OUTPUT value // comment\nEND").tokenize().expect("valid comment");
        assert!(tokens.iter().any(|token| token.kind == TokenKind::Newline));
        assert!(tokens.iter().any(|token| token.kind == TokenKind::End));
    }

    #[test]
    fn treats_crlf_as_one_newline_token() {
        let tokens = Lexer::new("OUTPUT x\r\nEND").tokenize().unwrap();
        assert_eq!(tokens.iter().filter(|t| t.kind == TokenKind::Newline).count(), 1);
    }

    #[test]
    fn lexical_error_position_is_a_utf8_byte_offset() {
        let source = "OUTPUT \"á\" @";
        let error = Lexer::new(source).tokenize().expect_err("unexpected symbol");
        assert_eq!(error.position, source.find('@').unwrap());
        assert!(source.is_char_boundary(error.position));
    }
}
