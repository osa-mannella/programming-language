use crate::types::token::Token;

pub struct Lexer {
    input: String,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            current_char: None,
        };
        lexer.current_char = lexer.input.chars().nth(0);
        lexer
    }

    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.chars().nth(self.position);
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.position + 1)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_string(&mut self) -> String {
        let mut value = String::new();
        self.advance(); // skip opening quote

        while let Some(ch) = self.current_char {
            if ch == '"' {
                self.advance(); // skip closing quote
                break;
            }
            value.push(ch);
            self.advance();
        }

        value
    }

    fn read_number(&mut self) -> f64 {
        let mut value = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() || ch == '.' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        value.parse::<f64>().unwrap_or(0.0)
    }

    fn read_identifier(&mut self) -> String {
        let mut value = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        value
    }

    fn read_comment(&mut self) -> String {
        let mut comment = String::new();

        if self.current_char == Some('/') && self.peek() == Some('/') {
            // Single line comment
            self.advance(); // skip first /
            self.advance(); // skip second /

            while let Some(ch) = self.current_char {
                if ch == '\n' {
                    break;
                }
                comment.push(ch);
                self.advance();
            }
        } else if self.current_char == Some('/') && self.peek() == Some('*') {
            // Multi-line comment
            self.advance(); // skip /
            self.advance(); // skip *

            while let Some(ch) = self.current_char {
                if ch == '*' && self.peek() == Some('/') {
                    self.advance(); // skip *
                    self.advance(); // skip /
                    break;
                }
                comment.push(ch);
                self.advance();
            }
        }

        comment
    }

    pub fn next_token(&mut self) -> Token {
        loop {
            match self.current_char {
                None => return Token::Eof,

                Some(ch) if ch.is_whitespace() && ch != '\n' => {
                    self.skip_whitespace();
                    continue;
                }

                Some('\n') => {
                    self.advance();
                    return Token::Newline;
                }

                Some('"') => {
                    let string_value = self.read_string();
                    return Token::String(string_value);
                }

                Some(ch) if ch.is_ascii_digit() => {
                    let number = self.read_number();
                    return Token::Number(number);
                }

                Some(ch) if ch.is_alphabetic() || ch == '_' => {
                    let identifier = self.read_identifier();
                    return match identifier.as_str() {
                        "let" => {
                            if self.current_char == Some('!') {
                                self.advance();
                                Token::LetBang
                            } else {
                                Token::Let
                            }
                        }
                        "func" => Token::Func,
                        "fn" => Token::Fn,
                        "match" => Token::Match,
                        "import" => Token::Import,
                        "enum" => Token::Enum,
                        "if" => Token::If,
                        "else" => Token::Else,
                        "return" => Token::Return,
                        "async" => Token::Async,
                        "await" => Token::Await,
                        _ => Token::Identifier(identifier),
                    };
                }

                Some('/') if self.peek() == Some('/') || self.peek() == Some('*') => {
                    self.read_comment();
                    continue; // Skip comments entirely
                }

                Some(ch) => {
                    self.advance();
                    match ch {
                        '+' => return Token::Plus,
                        '-' => {
                            if self.current_char == Some('>') {
                                self.advance();
                                return Token::Arrow;
                            } else {
                                return Token::Minus;
                            }
                        }
                        '*' => return Token::Multiply,
                        '/' => return Token::Divide,
                        '%' => return Token::Modulo,
                        '=' => {
                            if self.current_char == Some('=') {
                                self.advance();
                                return Token::Equal;
                            } else if self.current_char == Some('>') {
                                self.advance();
                                return Token::FatArrow;
                            } else {
                                return Token::Assign;
                            }
                        }
                        '!' => {
                            if self.current_char == Some('=') {
                                self.advance();
                                return Token::NotEqual;
                            } else {
                                return Token::Not;
                            }
                        }
                        '<' => {
                            if self.current_char == Some('=') {
                                self.advance();
                                return Token::LessEqual;
                            } else if self.current_char == Some('-') {
                                self.advance();
                                return Token::Update;
                            } else {
                                return Token::Less;
                            }
                        }
                        '>' => {
                            if self.current_char == Some('=') {
                                self.advance();
                                return Token::GreaterEqual;
                            } else {
                                return Token::Greater;
                            }
                        }
                        '&' => {
                            if self.current_char == Some('&') {
                                self.advance();
                                return Token::And;
                            } else {
                                continue; // Skip single &
                            }
                        }
                        '|' => {
                            if self.current_char == Some('|') {
                                self.advance();
                                return Token::Or;
                            } else if self.current_char == Some('>') {
                                self.advance();
                                return Token::Pipeline;
                            } else {
                                continue; // Skip single |
                            }
                        }
                        ':' => {
                            if self.current_char == Some(':') {
                                self.advance();
                                return Token::DoubleColon;
                            } else {
                                continue; // Skip single :
                            }
                        }
                        '(' => return Token::LeftParen,
                        ')' => return Token::RightParen,
                        '{' => return Token::LeftBrace,
                        '}' => return Token::RightBrace,
                        '[' => return Token::LeftBracket,
                        ']' => return Token::RightBracket,
                        ',' => return Token::Comma,
                        '.' => return Token::Dot,
                        '#' => return Token::Hash,
                        _ => continue, // Skip unknown characters
                    }
                }
            }
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();
            let is_eof = matches!(token, Token::Eof);
            tokens.push(token);

            if is_eof {
                break;
            }
        }

        tokens
    }
}
