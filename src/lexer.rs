use crate::token::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub position: usize,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lexical error at character {}: {}", self.position, self.message)
    }
}

impl std::error::Error for LexError {}

pub struct Lexer {
    source: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            position: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.position).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.position + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let current = self.peek()?;
        self.position += 1;
        Some(current)
    }

    fn skip_horizontal_whitespace(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\r')) {
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
        self.advance(); // Opening quote.
        let mut value = String::new();

        loop {
            match self.advance() {
                Some('"') => return Ok(value),
                Some('\\') => match self.advance() {
                    Some('n') => value.push('\n'),
                    Some('r') => value.push('\r'),
                    Some('t') => value.push('\t'),
                    Some('"') => value.push('"'),
                    Some('\\') => value.push('\\'),
                    Some(other) => {
                        return Err(LexError {
                            message: format!("unsupported escape sequence: \\{other}"),
                            position: self.position.saturating_sub(2),
                        });
                    }
                    None => {
                        return Err(LexError {
                            message: "unterminated escape sequence".to_string(),
                            position: self.position,
                        });
                    }
                },
                Some('\n') => {
                    return Err(LexError {
                        message: "newline inside string literal".to_string(),
                        position: self.position.saturating_sub(1),
                    });
                }
                Some(ch) => value.push(ch),
                None => {
                    return Err(LexError {
                        message: "unterminated string literal".to_string(),
                        position: start,
                    });
                }
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

        while self.position < self.source.len() {
            self.skip_horizontal_whitespace();

            let Some(ch) = self.peek() else {
                break;
            };

            let start = self.position;

            let kind = match ch {
                '\n' => {
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
                '"' => {
                    let value = self.read_string(start)?;
                    Some(TokenKind::StringLiteral(value))
                }
                '/' if self.peek_next() == Some('/') => {
                    while let Some(current) = self.peek() {
                        if current == '\n' {
                            break;
                        }
                        self.advance();
                    }
                    None
                }
                c if c.is_ascii_digit() => {
                    Some(TokenKind::Number(self.read_number()))
                }
                c if c.is_ascii_alphabetic() || c == '_' => {
                    Some(Self::keyword_or_identifier(self.read_identifier()))
                }
                other => {
                    return Err(LexError {
                        message: format!("unexpected character {other:?}"),
                        position: start,
                    });
                }
            };

            if let Some(kind) = kind {
                tokens.push(Token::new(kind, start, self.position));
            }
        }

        tokens.push(Token::new(
            TokenKind::Eof,
            self.position,
            self.position,
        ));

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
        assert!(tokens.iter().any(|token| {
            token.kind == TokenKind::Identifier("count".into())
        }));
        assert!(tokens.iter().any(|token| {
            token.kind == TokenKind::Number("12".into())
        }));
        assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
    }

    #[test]
    fn decodes_string_escapes() {
        let tokens = Lexer::new(r#"OUTPUT "line\nnext""#)
            .tokenize()
            .expect("valid escape");

        assert_eq!(
            tokens[1].kind,
            TokenKind::StringLiteral("line\nnext".into())
        );
    }

    #[test]
    fn rejects_unterminated_strings() {
        let error = Lexer::new(r#"OUTPUT "unfinished"#)
            .tokenize()
            .expect_err("unterminated string must fail");

        assert!(error.message.contains("unterminated"));
    }

    #[test]
    fn skips_line_comments_but_keeps_newline() {
        let tokens = Lexer::new("OUTPUT value // comment\nEND")
            .tokenize()
            .expect("valid comment");

        assert!(tokens.iter().any(|token| token.kind == TokenKind::Newline));
        assert!(tokens.iter().any(|token| token.kind == TokenKind::End));
    }
}
