use crate::types::token::Token;

pub fn print_tokens(tokens: &[Token]) {
    println!("=== LEXED TOKENS ===");
    for (i, token) in tokens.iter().enumerate() {
        println!("{:3}: {:?}", i, token);
    }
    println!("===================");
}

pub fn print_token_summary(tokens: &[Token]) {
    let mut counts = std::collections::HashMap::new();

    for token in tokens {
        let token_type: &str = match token {
            Token::Identifier(_) => "Identifier",
            Token::String(_) => "String",
            Token::Number(_) => "Number",
            Token::Let => "Let",
            Token::LetBang => "LetBang",
            Token::Func => "Func",
            Token::Fn => "Fn",
            Token::Match => "Match",
            Token::Import => "Import",
            Token::Enum => "Enum",
            Token::If => "If",
            Token::Else => "Else",
            Token::Return => "Return",
            Token::Async => "Async",
            Token::Await => "Await",
            Token::Plus => "Plus",
            Token::Minus => "Minus",
            Token::Multiply => "Multiply",
            Token::Divide => "Divide",
            Token::Modulo => "Modulo",
            Token::Equal => "Equal",
            Token::NotEqual => "NotEqual",
            Token::Less => "Less",
            Token::Greater => "Greater",
            Token::LessEqual => "LessEqual",
            Token::GreaterEqual => "GreaterEqual",
            Token::Assign => "Assign",
            Token::And => "And",
            Token::Or => "Or",
            Token::Not => "Not",
            Token::Pipeline => "Pipeline",
            Token::Update => "Update",
            Token::DoubleColon => "DoubleColon",
            Token::LeftParen => "LeftParen",
            Token::RightParen => "RightParen",
            Token::LeftBrace => "LeftBrace",
            Token::RightBrace => "RightBrace",
            Token::LeftBracket => "LeftBracket",
            Token::RightBracket => "RightBracket",
            Token::Comma => "Comma",
            Token::Dot => "Dot",
            Token::Arrow => "Arrow",
            Token::FatArrow => "FatArrow",
            Token::Hash => "Hash",
            Token::Newline => "Newline",
            Token::Eof => "Eof",
        };
        *counts.entry(token_type).or_insert(0) += 1;
    }

    println!("=== TOKEN SUMMARY ===");
    for (token_type, count) in counts {
        println!("{}: {}", token_type, count);
    }
    println!("====================");
}
