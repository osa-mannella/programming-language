#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Identifier(String),
    String(String),
    Number(f64),

    // Keywords
    Let,
    LetBang,
    Func,
    Fn,
    Match,
    Import,
    Enum,
    If,
    Else,
    Return,
    Async,
    Await,

    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Assign,
    And,
    Or,
    Not,
    Pipeline,    // |>
    Update,      // <-
    DoubleColon, // ::

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Arrow,    // ->
    FatArrow, // =>
    Hash,     // #

    // Misc
    Newline,
    Eof,
}
