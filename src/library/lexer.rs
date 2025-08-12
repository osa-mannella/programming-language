#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Identifier,
    Number,
    String,
    InterpolatedString,

    Let,
    Func,
    If,
    Else,
    True,
    False,
    Match,
    Fn,
    Async,
    Await,
    Import,
    Enum,
    Return,

    Equal,
    EqualEqual,
    BangEqual,
    GreaterEqual,
    LessEqual,
    Greater,
    Less,
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Comma,
    Semicolon,
    Colon,
    DoubleColon,
    Bang,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Dot,
    And,
    Or,
    Arrow,
    Question,
    Reflect,
    Pipe,
    Pipeline,
    LArrow,
    Dollar,
    Underscore,

    Eof,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    None,
    Identifier(String),
    Number(f64),
    String(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub value: TokenValue,
    pub line: usize,
}

impl Token {
    pub fn eof() -> Self {
        Token {
            kind: TokenKind::Eof,
            value: TokenValue::None,
            line: 0,
        }
    }
}

pub struct Lexer<'a> {
    chars: std::str::Chars<'a>,
    current: Option<char>,
    line: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars();
        let current = chars.next();
        Self {
            chars,
            current,
            line: 1,
        }
    }

    fn advance(&mut self) -> Option<char> {
        match self.current {
            Some('\n') => {
                self.line += 1;
            }
            _ => {}
        }
        self.current = self.chars.next();
        self.current
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.as_str().chars().next()
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.current == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comments(&mut self) -> bool {
        match self.current {
            Some('/') => {
                match self.peek() {
                    Some('/') => {
                        self.advance(); // consume first /
                        self.advance(); // consume second /
                        while let Some(ch) = self.current {
                            if ch == '\n' {
                                break;
                            }
                            self.advance();
                        }
                        true
                    }
                    Some('*') => {
                        self.advance(); // consume /
                        self.advance(); // consume *
                        while let Some(ch) = self.current {
                            if ch == '*' && self.peek() == Some('/') {
                                self.advance(); // consume *
                                self.advance(); // consume /
                                break;
                            }
                            self.advance();
                        }
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn make_number(&mut self) -> Option<Token> {
        let mut number = String::new();

        while let Some(ch) = self.current {
            if ch.is_ascii_digit() || ch == '.' {
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if let Ok(value) = number.parse::<f64>() {
            Some(Token {
                kind: TokenKind::Number,
                value: TokenValue::Number(value),
                line: self.line,
            })
        } else {
            Some(Token {
                kind: TokenKind::Error,
                value: TokenValue::Error(format!("Invalid number: {}", number)),
                line: self.line,
            })
        }
    }

    fn make_string(&mut self, quote_char: char) -> Option<Token> {
        self.make_regular_string(quote_char)
    }

    fn make_regular_string(&mut self, quote_char: char) -> Option<Token> {
        let mut string = String::new();
        self.advance(); // consume opening quote

        while let Some(ch) = self.current {
            if ch == quote_char {
                self.advance(); // consume closing quote
                return Some(Token {
                    kind: TokenKind::String,
                    value: TokenValue::String(string),
                    line: self.line,
                });
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current {
                    match escaped {
                        'n' => string.push('\n'),
                        't' => string.push('\t'),
                        'r' => string.push('\r'),
                        '\\' => string.push('\\'),
                        '"' => string.push('"'),
                        '\'' => string.push('\''),
                        _ => {
                            string.push('\\');
                            string.push(escaped);
                        }
                    }
                    self.advance();
                }
            } else {
                string.push(ch);
                self.advance();
            }
        }

        Some(Token {
            kind: TokenKind::Error,
            value: TokenValue::Error("Unterminated string".to_string()),
            line: self.line,
        })
    }

    fn make_interpolated_string(&mut self) -> Option<Token> {
        let mut string = String::new();
        self.advance(); // consume opening quote

        while let Some(ch) = self.current {
            if ch == '"' {
                self.advance(); // consume closing quote
                return Some(Token {
                    kind: TokenKind::InterpolatedString,
                    value: TokenValue::String(string),
                    line: self.line,
                });
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current {
                    match escaped {
                        'n' => {
                            string.push('\\');
                            string.push('n');
                        },
                        't' => {
                            string.push('\\');
                            string.push('t');
                        },
                        'r' => {
                            string.push('\\');
                            string.push('r');
                        },
                        '\\' => {
                            string.push('\\');
                            string.push('\\');
                        },
                        '"' => {
                            string.push('\\');
                            string.push('"');
                        },
                        '$' => {
                            string.push('\\');
                            string.push('$');
                        },
                        _ => {
                            string.push('\\');
                            string.push(escaped);
                        }
                    }
                    self.advance();
                }
            } else {
                string.push(ch);
                self.advance();
            }
        }

        Some(Token {
            kind: TokenKind::Error,
            value: TokenValue::Error("Unterminated interpolated string".to_string()),
            line: self.line,
        })
    }

    fn make_identifier(&mut self) -> Option<Token> {
        let mut identifier = String::new();

        while let Some(ch) = self.current {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let kind = match identifier.as_str() {
            "let" => TokenKind::Let,
            "func" => TokenKind::Func,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "match" => TokenKind::Match,
            "fn" => TokenKind::Fn,
            "async" => TokenKind::Async,
            "await" => TokenKind::Await,
            "import" => TokenKind::Import,
            "enum" => TokenKind::Enum,
            "return" => TokenKind::Return,
            _ => TokenKind::Identifier,
        };

        Some(Token {
            kind: kind.clone(),
            value: if kind == TokenKind::Identifier {
                TokenValue::Identifier(identifier)
            } else {
                TokenValue::None
            },
            line: self.line,
        })
    }

    pub fn next(&mut self) -> Option<Token> {
        loop {
            self.skip_whitespace();
            if !self.skip_comments() {
                break;
            }
        }

        let ch = self.current?;

        match ch {
            // Single character tokens
            '(' => {
                self.advance();
                Some(Token {
                    kind: TokenKind::LParen,
                    value: TokenValue::None,
                    line: self.line,
                })
            }
            ')' => {
                self.advance();
                Some(Token {
                    kind: TokenKind::RParen,
                    value: TokenValue::None,
                    line: self.line,
                })
            }
            '{' => {
                self.advance();
                Some(Token {
                    kind: TokenKind::LBrace,
                    value: TokenValue::None,
                    line: self.line,
                })
            }
            '}' => {
                self.advance();
                Some(Token {
                    kind: TokenKind::RBrace,
                    value: TokenValue::None,
                    line: self.line,
                })
            }
            '[' => {
                self.advance();
                Some(Token {
                    kind: TokenKind::LBracket,
                    value: TokenValue::None,
                    line: self.line,
                })
            }
            ']' => {
                self.advance();
                Some(Token {
                    kind: TokenKind::RBracket,
                    value: TokenValue::None,
                    line: self.line,
                })
            }
            ',' => {
                self.advance();
                Some(Token {
                    kind: TokenKind::Comma,
                    value: TokenValue::None,
                    line: self.line,
                })
            }
            ';' => {
                self.advance();
                Some(Token {
                    kind: TokenKind::Semicolon,
                    value: TokenValue::None,
                    line: self.line,
                })
            }
            '+' => {
                self.advance();
                Some(Token {
                    kind: TokenKind::Plus,
                    value: TokenValue::None,
                    line: self.line,
                })
            }
            '-' => {
                self.advance();
                if self.match_char('>') {
                    Some(Token {
                        kind: TokenKind::Arrow,
                        value: TokenValue::None,
                        line: self.line,
                    })
                } else {
                    Some(Token {
                        kind: TokenKind::Minus,
                        value: TokenValue::None,
                        line: self.line,
                    })
                }
            }
            '*' => {
                self.advance();
                Some(Token {
                    kind: TokenKind::Star,
                    value: TokenValue::None,
                    line: self.line,
                })
            }
            '/' => {
                self.advance();
                Some(Token {
                    kind: TokenKind::Slash,
                    value: TokenValue::None,
                    line: self.line,
                })
            }
            '^' => {
                self.advance();
                Some(Token {
                    kind: TokenKind::Caret,
                    value: TokenValue::None,
                    line: self.line,
                })
            }
            '.' => {
                self.advance();
                Some(Token {
                    kind: TokenKind::Dot,
                    value: TokenValue::None,
                    line: self.line,
                })
            }
            '?' => {
                self.advance();
                Some(Token {
                    kind: TokenKind::Question,
                    value: TokenValue::None,
                    line: self.line,
                })
            }
            '$' => {
                if self.peek() == Some('"') {
                    self.advance(); // consume '$'
                    self.make_interpolated_string()
                } else {
                    self.advance();
                    Some(Token {
                        kind: TokenKind::Dollar,
                        value: TokenValue::None,
                        line: self.line,
                    })
                }
            }

            // Multi-character tokens
            '=' => {
                self.advance();
                if self.match_char('=') {
                    Some(Token {
                        kind: TokenKind::EqualEqual,
                        value: TokenValue::None,
                        line: self.line,
                    })
                } else {
                    Some(Token {
                        kind: TokenKind::Equal,
                        value: TokenValue::None,
                        line: self.line,
                    })
                }
            }
            '!' => {
                self.advance();
                if self.match_char('=') {
                    Some(Token {
                        kind: TokenKind::BangEqual,
                        value: TokenValue::None,
                        line: self.line,
                    })
                } else {
                    Some(Token {
                        kind: TokenKind::Bang,
                        value: TokenValue::None,
                        line: self.line,
                    })
                }
            }
            '<' => {
                self.advance();
                if self.match_char('=') {
                    Some(Token {
                        kind: TokenKind::LessEqual,
                        value: TokenValue::None,
                        line: self.line,
                    })
                } else if self.match_char('-') {
                    Some(Token {
                        kind: TokenKind::LArrow,
                        value: TokenValue::None,
                        line: self.line,
                    })
                } else {
                    Some(Token {
                        kind: TokenKind::Less,
                        value: TokenValue::None,
                        line: self.line,
                    })
                }
            }
            '>' => {
                self.advance();
                if self.match_char('=') {
                    Some(Token {
                        kind: TokenKind::GreaterEqual,
                        value: TokenValue::None,
                        line: self.line,
                    })
                } else {
                    Some(Token {
                        kind: TokenKind::Greater,
                        value: TokenValue::None,
                        line: self.line,
                    })
                }
            }
            ':' => {
                self.advance();
                if self.match_char(':') {
                    Some(Token {
                        kind: TokenKind::DoubleColon,
                        value: TokenValue::None,
                        line: self.line,
                    })
                } else {
                    Some(Token {
                        kind: TokenKind::Colon,
                        value: TokenValue::None,
                        line: self.line,
                    })
                }
            }
            '&' => {
                self.advance();
                if self.match_char('&') {
                    Some(Token {
                        kind: TokenKind::And,
                        value: TokenValue::None,
                        line: self.line,
                    })
                } else {
                    Some(Token {
                        kind: TokenKind::Reflect,
                        value: TokenValue::None,
                        line: self.line,
                    })
                }
            }
            '|' => {
                self.advance();
                if self.match_char('|') {
                    Some(Token {
                        kind: TokenKind::Or,
                        value: TokenValue::None,
                        line: self.line,
                    })
                } else if self.match_char('>') {
                    Some(Token {
                        kind: TokenKind::Pipeline,
                        value: TokenValue::None,
                        line: self.line,
                    })
                } else {
                    Some(Token {
                        kind: TokenKind::Pipe,
                        value: TokenValue::None,
                        line: self.line,
                    })
                }
            }

            // String literals
            '"' => self.make_string('"'),
            '\'' => self.make_string('\''),

            // Numbers
            ch if ch.is_ascii_digit() => self.make_number(),

            // Standalone underscore (wildcard)
            '_' => {
                let next_char = self.peek();
                if next_char.is_none() || (!next_char.unwrap().is_ascii_alphanumeric() && next_char.unwrap() != '_') {
                    self.advance();
                    Some(Token {
                        kind: TokenKind::Underscore,
                        value: TokenValue::None,
                        line: self.line,
                    })
                } else {
                    self.make_identifier()
                }
            }

            // Identifiers and keywords
            ch if ch.is_ascii_alphabetic() => self.make_identifier(),

            _ => {
                self.advance();
                Some(Token {
                    kind: TokenKind::Error,
                    value: TokenValue::Error(format!("Unexpected character: {}", ch)),
                    line: self.line,
                })
            }
        }
    }
}

