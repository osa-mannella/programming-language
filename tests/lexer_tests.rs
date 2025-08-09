use mirrow::library::lexer::{Lexer, Token, TokenKind, TokenValue};

fn tokenize_all(source: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();

    while let Some(token) = lexer.next() {
        let is_eof = token.kind == TokenKind::Eof;
        tokens.push(token);
        if is_eof {
            break;
        }
    }

    tokens
}

#[test]
fn test_single_character_tokens() {
    let source = "(){}[],;+-*/.?$_^";
    let tokens = tokenize_all(source);

    let expected_kinds = vec![
        TokenKind::LParen,
        TokenKind::RParen,
        TokenKind::LBrace,
        TokenKind::RBrace,
        TokenKind::LBracket,
        TokenKind::RBracket,
        TokenKind::Comma,
        TokenKind::Semicolon,
        TokenKind::Plus,
        TokenKind::Minus,
        TokenKind::Star,
        TokenKind::Slash,
        TokenKind::Dot,
        TokenKind::Question,
        TokenKind::Dollar,
        TokenKind::Underscore,
        TokenKind::Caret,
    ];

    for (i, expected_kind) in expected_kinds.iter().enumerate() {
        assert_eq!(tokens[i].kind, *expected_kind, "Token {} mismatch", i);
    }
}

#[test]
fn test_multi_character_operators() {
    let source = "== != <= >= <- -> :: && || |>";
    let tokens = tokenize_all(source);

    let expected_kinds = vec![
        TokenKind::EqualEqual,
        TokenKind::BangEqual,
        TokenKind::LessEqual,
        TokenKind::GreaterEqual,
        TokenKind::LArrow,
        TokenKind::Arrow,
        TokenKind::DoubleColon,
        TokenKind::And,
        TokenKind::Or,
        TokenKind::Pipeline,
    ];

    for (i, expected_kind) in expected_kinds.iter().enumerate() {
        assert_eq!(tokens[i].kind, *expected_kind, "Token {} mismatch", i);
    }
}

#[test]
fn test_keywords() {
    let source = "let func if else true false match fn async await import enum";
    let tokens = tokenize_all(source);

    let expected_kinds = vec![
        TokenKind::Let,
        TokenKind::Func,
        TokenKind::If,
        TokenKind::Else,
        TokenKind::True,
        TokenKind::False,
        TokenKind::Match,
        TokenKind::Fn,
        TokenKind::Async,
        TokenKind::Await,
        TokenKind::Import,
        TokenKind::Enum,
    ];

    for (i, expected_kind) in expected_kinds.iter().enumerate() {
        assert_eq!(tokens[i].kind, *expected_kind, "Keyword {} mismatch", i);
    }
}

#[test]
fn test_identifiers() {
    let source = "hello world my_variable _private var123";
    let tokens = tokenize_all(source);

    let expected_values = vec!["hello", "world", "my_variable", "_private", "var123"];

    for (i, expected_value) in expected_values.iter().enumerate() {
        assert_eq!(tokens[i].kind, TokenKind::Identifier);
        if let TokenValue::Identifier(name) = &tokens[i].value {
            assert_eq!(name, expected_value, "Identifier {} value mismatch", i);
        } else {
            panic!("Expected identifier token value at position {}", i);
        }
    }
}

#[test]
fn test_numbers() {
    let source = "42 3.14 0 123.456";
    let tokens = tokenize_all(source);

    let expected_values = vec![42.0, 3.14, 0.0, 123.456];

    for (i, expected_value) in expected_values.iter().enumerate() {
        assert_eq!(tokens[i].kind, TokenKind::Number);
        if let TokenValue::Number(value) = tokens[i].value {
            assert!(
                (value - expected_value).abs() < f64::EPSILON,
                "Number {} value mismatch: expected {}, got {}",
                i,
                expected_value,
                value
            );
        } else {
            panic!("Expected number token value at position {}", i);
        }
    }
}

#[test]
fn test_strings() {
    let source = r#""hello" "world with spaces" "escaped \"quote\"" "newline\n""#;
    let tokens = tokenize_all(source);

    let expected_values = vec![
        "hello",
        "world with spaces",
        "escaped \"quote\"",
        "newline\n",
    ];

    for (i, expected_value) in expected_values.iter().enumerate() {
        assert_eq!(tokens[i].kind, TokenKind::String);
        if let TokenValue::String(value) = &tokens[i].value {
            assert_eq!(value, expected_value, "String {} value mismatch", i);
        } else {
            panic!("Expected string token value at position {}", i);
        }
    }
}

#[test]
fn test_complex_expression() {
    let source = "let result = my_func(x, y) + 42 * 3.14";
    let tokens = tokenize_all(source);

    let expected_kinds = vec![
        TokenKind::Let,
        TokenKind::Identifier,
        TokenKind::Equal,
        TokenKind::Identifier,
        TokenKind::LParen,
        TokenKind::Identifier,
        TokenKind::Comma,
        TokenKind::Identifier,
        TokenKind::RParen,
        TokenKind::Plus,
        TokenKind::Number,
        TokenKind::Star,
        TokenKind::Number,
    ];

    for (i, expected_kind) in expected_kinds.iter().enumerate() {
        assert_eq!(
            tokens[i].kind, *expected_kind,
            "Complex expression token {} mismatch",
            i
        );
    }
}

#[test]
fn test_array_append_syntax() {
    let source = "arr <- [1, 2, 3]";
    let tokens = tokenize_all(source);

    let expected_kinds = vec![
        TokenKind::Identifier,
        TokenKind::LArrow,
        TokenKind::LBracket,
        TokenKind::Number,
        TokenKind::Comma,
        TokenKind::Number,
        TokenKind::Comma,
        TokenKind::Number,
        TokenKind::RBracket,
    ];

    for (i, expected_kind) in expected_kinds.iter().enumerate() {
        assert_eq!(
            tokens[i].kind, *expected_kind,
            "Array append token {} mismatch",
            i
        );
    }
}

#[test]
fn test_struct_syntax() {
    let source = "Person::Programmer { name = \"John\", age = 30 }";
    let tokens = tokenize_all(source);

    let expected_kinds = vec![
        TokenKind::Identifier,
        TokenKind::DoubleColon,
        TokenKind::Identifier,
        TokenKind::LBrace,
        TokenKind::Identifier,
        TokenKind::Equal,
        TokenKind::String,
        TokenKind::Comma,
        TokenKind::Identifier,
        TokenKind::Equal,
        TokenKind::Number,
        TokenKind::RBrace,
    ];

    for (i, expected_kind) in expected_kinds.iter().enumerate() {
        assert_eq!(
            tokens[i].kind, *expected_kind,
            "Struct syntax token {} mismatch",
            i
        );
    }
}

#[test]
fn test_match_expression() {
    let source = "match value { Some(x) -> x, None -> 0 }";
    let tokens = tokenize_all(source);

    let expected_kinds = vec![
        TokenKind::Match,
        TokenKind::Identifier,
        TokenKind::LBrace,
        TokenKind::Identifier,
        TokenKind::LParen,
        TokenKind::Identifier,
        TokenKind::RParen,
        TokenKind::Arrow,
        TokenKind::Identifier,
        TokenKind::Comma,
        TokenKind::Identifier,
        TokenKind::Arrow,
        TokenKind::Number,
        TokenKind::RBrace,
    ];

    for (i, expected_kind) in expected_kinds.iter().enumerate() {
        assert_eq!(
            tokens[i].kind, *expected_kind,
            "Match expression token {} mismatch",
            i
        );
    }
}

#[test]
fn test_async_await_syntax() {
    let source = "async func test() { await some_async_call() }";
    let tokens = tokenize_all(source);

    let expected_kinds = vec![
        TokenKind::Async,
        TokenKind::Func,
        TokenKind::Identifier,
        TokenKind::LParen,
        TokenKind::RParen,
        TokenKind::LBrace,
        TokenKind::Await,
        TokenKind::Identifier,
        TokenKind::LParen,
        TokenKind::RParen,
        TokenKind::RBrace,
    ];

    for (i, expected_kind) in expected_kinds.iter().enumerate() {
        assert_eq!(
            tokens[i].kind, *expected_kind,
            "Async/await token {} mismatch",
            i
        );
    }
}

#[test]
fn test_pipeline_operator() {
    let source = "value |> transform |> filter";
    let tokens = tokenize_all(source);

    let expected_kinds = vec![
        TokenKind::Identifier,
        TokenKind::Pipeline,
        TokenKind::Identifier,
        TokenKind::Pipeline,
        TokenKind::Identifier,
    ];

    for (i, expected_kind) in expected_kinds.iter().enumerate() {
        assert_eq!(
            tokens[i].kind, *expected_kind,
            "Pipeline token {} mismatch",
            i
        );
    }
}

#[test]
fn test_line_numbers() {
    let source = "let\nx\n=\n42";
    let tokens = tokenize_all(source);

    assert_eq!(tokens[0].line, 1); // let
    assert_eq!(tokens[1].line, 2); // x  
    assert_eq!(tokens[2].line, 3); // =
    assert_eq!(tokens[3].line, 4); // 42
}

#[test]
fn test_whitespace_handling() {
    let source = "  let   x    =    42   ";
    let tokens = tokenize_all(source);

    let expected_kinds = vec![
        TokenKind::Let,
        TokenKind::Identifier,
        TokenKind::Equal,
        TokenKind::Number,
    ];

    for (i, expected_kind) in expected_kinds.iter().enumerate() {
        assert_eq!(
            tokens[i].kind, *expected_kind,
            "Whitespace handling token {} mismatch",
            i
        );
    }
}

#[test]
fn test_error_handling() {
    let source = "@#%";
    let tokens = tokenize_all(source);

    // Should produce Error tokens for unexpected characters
    for token in &tokens[..3] {
        // Skip EOF
        assert_eq!(token.kind, TokenKind::Error);
    }
}

#[test]
fn test_unterminated_string() {
    let source = r#""unterminated string"#;
    let tokens = tokenize_all(source);

    assert_eq!(tokens[0].kind, TokenKind::Error);
    if let TokenValue::Error(msg) = &tokens[0].value {
        assert!(msg.contains("Unterminated string"));
    }
}

#[test]
fn test_empty_input() {
    let source = "";
    let tokens = tokenize_all(source);

    assert_eq!(tokens.len(), 0);
}

#[test]
fn test_whitespace_only() {
    let source = "   \n  \t  \n  ";
    let tokens = tokenize_all(source);

    assert_eq!(tokens.len(), 0);
}

#[test]
fn test_interpolated_strings() {
    let source = r#"$"Hello ${name}!""#;
    let tokens = tokenize_all(source);

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::InterpolatedString);
    if let TokenValue::String(s) = &tokens[0].value {
        assert_eq!(s, "Hello ${name}!");
    } else {
        panic!("Expected string value in interpolated string token");
    }
}

#[test]
fn test_interpolated_strings_multiple_expressions() {
    let source = r#"$"User ${user} has ${count} items""#;
    let tokens = tokenize_all(source);

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::InterpolatedString);
    if let TokenValue::String(s) = &tokens[0].value {
        assert_eq!(s, "User ${user} has ${count} items");
    } else {
        panic!("Expected string value in interpolated string token");
    }
}

#[test]
fn test_caret_power_operator() {
    let source = "2 ^ 3";
    let tokens = tokenize_all(source);

    let expected_kinds = vec![TokenKind::Number, TokenKind::Caret, TokenKind::Number];

    for (i, expected_kind) in expected_kinds.iter().enumerate() {
        assert_eq!(
            tokens[i].kind, *expected_kind,
            "Power operator token {} mismatch",
            i
        );
    }
}
