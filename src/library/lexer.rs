#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Identifier,
    Number,
    String,

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
    Power,

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
        let c = self.current;
        self.current = self.chars.next();
        c
    }

    fn peek(&self) -> Option<char> {
        self.current
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.clone().next()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            match c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                c if c < ' ' && c != '\n' && c != '\r' && c != '\t' => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn lex_identifier(&mut self, first: char) -> Token {
        let mut text = String::new();
        text.push(first);
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                text.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        let (kind, value) = match text.as_str() {
            "let" => (TokenKind::Let, TokenValue::None),
            "func" => (TokenKind::Func, TokenValue::None),
            "if" => (TokenKind::If, TokenValue::None),
            "match" => (TokenKind::Match, TokenValue::None),
            "async" => (TokenKind::Async, TokenValue::None),
            "await" => (TokenKind::Await, TokenValue::None),
            "import" => (TokenKind::Import, TokenValue::None),
            "enum" => (TokenKind::Enum, TokenValue::None),
            "fn" => (TokenKind::Fn, TokenValue::None),
            _ => (TokenKind::Identifier, TokenValue::Identifier(text)),
        };

        Token {
            kind,
            value,
            line: self.line,
        }
    }

    fn lex_number(&mut self, first: char) -> Token {
        let mut text = String::new();
        text.push(first);

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                text.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        if self.peek() == Some('.')
            && self
                .peek_next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            text.push(self.advance().unwrap());
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    text.push(self.advance().unwrap());
                } else {
                    break;
                }
            }
        }

        let value: f64 = text.parse().unwrap_or(0.0);
        Token {
            kind: TokenKind::Number,
            value: TokenValue::Number(value),
            line: self.line,
        }
    }

    fn lex_string(&mut self) -> Token {
        let mut text = String::new();
        while let Some(c) = self.current {
            if c == '"' {
                break;
            }
            if c == '\n' {
                self.line += 1;
            }
            text.push(c);
            self.advance();
        }

        if self.current.is_none() {
            return Token {
                kind: TokenKind::Error,
                value: TokenValue::Error("Unterminated string.".to_string()),
                line: self.line,
            };
        }

        self.advance(); // consume closing quote
        Token {
            kind: TokenKind::String,
            value: TokenValue::String(text),
            line: self.line,
        }
    }
    fn make_error(&self, msg: &str) -> Token {
        Token {
            kind: TokenKind::Error,
            value: TokenValue::Error(msg.to_string()),
            line: self.line,
        }
    }
    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let c = self.advance();
        match c {
            None => Some(Token {
                kind: TokenKind::Eof,
                value: TokenValue::None,
                line: self.line,
            }),

            Some(c) if c.is_alphabetic() || c == '_' => Some(self.lex_identifier(c)),
            Some(c) if c.is_ascii_digit() => Some(self.lex_number(c)),
            Some('"') => Some(self.lex_string()),

            Some('(') => Some(Token {
                kind: TokenKind::LParen,
                value: TokenValue::None,
                line: self.line,
            }),
            Some(')') => Some(Token {
                kind: TokenKind::RParen,
                value: TokenValue::None,
                line: self.line,
            }),
            Some('{') => Some(Token {
                kind: TokenKind::LBrace,
                value: TokenValue::None,
                line: self.line,
            }),
            Some('}') => Some(Token {
                kind: TokenKind::RBrace,
                value: TokenValue::None,
                line: self.line,
            }),
            Some('=') => {
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
            Some('!') => {
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
            Some('>') => {
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
            Some('<') => {
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
            Some('-') => {
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
            Some('+') => Some(Token {
                kind: TokenKind::Plus,
                value: TokenValue::None,
                line: self.line,
            }),
            Some('*') => Some(Token {
                kind: TokenKind::Star,
                value: TokenValue::None,
                line: self.line,
            }),
            Some('/') => {
                if self.match_char('/') {
                    while let Some(ch) = self.peek() {
                        if ch == '\n' {
                            break;
                        }
                        self.advance();
                    }
                    self.next()
                } else if self.match_char('*') {
                    while let Some(ch) = self.peek() {
                        if ch == '*' && self.peek_next() == Some('/') {
                            self.advance();
                            self.advance();
                            break;
                        }
                        if ch == '\n' {
                            self.line += 1;
                        }
                        self.advance();
                    }
                    self.next()
                } else {
                    Some(Token {
                        kind: TokenKind::Slash,
                        value: TokenValue::None,
                        line: self.line,
                    })
                }
            }
            Some(',') => Some(Token {
                kind: TokenKind::Comma,
                value: TokenValue::None,
                line: self.line,
            }),
            Some(';') => Some(Token {
                kind: TokenKind::Semicolon,
                value: TokenValue::None,
                line: self.line,
            }),
            Some(':') => {
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
            Some('.') => Some(Token {
                kind: TokenKind::Dot,
                value: TokenValue::None,
                line: self.line,
            }),
            Some('[') => Some(Token {
                kind: TokenKind::LBracket,
                value: TokenValue::None,
                line: self.line,
            }),
            Some(']') => Some(Token {
                kind: TokenKind::RBracket,
                value: TokenValue::None,
                line: self.line,
            }),
            Some('&') => {
                if self.match_char('&') {
                    Some(Token {
                        kind: TokenKind::And,
                        value: TokenValue::None,
                        line: self.line,
                    })
                } else {
                    Some(self.make_error("Unexpected '&'"))
                }
            }
            Some('|') => {
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
            Some('?') => Some(Token {
                kind: TokenKind::Question,
                value: TokenValue::None,
                line: self.line,
            }),
            Some('#') => Some(Token {
                kind: TokenKind::Reflect,
                value: TokenValue::None,
                line: self.line,
            }),
            Some('_') => Some(Token {
                kind: TokenKind::Underscore,
                value: TokenValue::None,
                line: self.line,
            }),
            Some('^') => Some(Token {
                kind: TokenKind::Power,
                value: TokenValue::None,
                line: self.line,
            }),
            Some('$') => Some(Token {
                kind: TokenKind::Dollar,
                value: TokenValue::None,
                line: self.line,
            }),
            _ => Some(self.make_error("Unexpected character.")),
        }
    }
}
