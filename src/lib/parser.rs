use super::ast::{ASTNode, ASTProgram};
use super::lexer::{Lexer, Token, TokenKind};
use std::{collections::HashMap, sync::Arc};

type ParseResult = Option<ASTNode>;

type NudFn<'a> = Arc<dyn Fn(&mut Parser<'a>, Token) -> ParseResult + 'a>;
type LedFn<'a> = Arc<dyn Fn(&mut Parser<'a>, ASTNode, Token) -> ParseResult + 'a>;

struct ParseRule<'a> {
    nud: Option<NudFn<'a>>,
    led: Option<LedFn<'a>>,
    lbp: u8,
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
    rules: HashMap<TokenKind, ParseRule<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let first = lexer.next().unwrap_or_else(|| Token::eof());
        let mut parser = Parser {
            lexer,
            current: first.clone(),
            previous: first,
            had_error: false,
            panic_mode: false,
            rules: HashMap::new(),
        };
        parser.init_parse_rules();
        parser
    }

    // Utility: advance the parser to the next token
    fn advance(&mut self) {
        self.previous = self.current.clone();
        self.current = self.lexer.next().unwrap_or_else(|| Token::eof());
    }

    fn consume(&mut self, kind: TokenKind, message: &str) {
        if self.current.kind == kind {
            self.advance();
        } else {
            self.error(message);
        }
    }

    fn error(&mut self, msg: &str) {
        self.had_error = true;
        eprintln!("Parse error (line {}): {}", self.current.line, msg);
    }

    fn parse_expression(&mut self, min_precedence: u8) -> ParseResult {
        self.advance();
        let nud = self
            .rules
            .get(&self.previous.kind)
            .and_then(|rule| rule.nud.as_ref())
            .cloned(); // <-- get an owned copy (Box) of the closure

        let mut left = if let Some(nud) = nud {
            nud(self, self.previous.clone())
        } else {
            self.error("Expected expression");
            return None;
        };

        while min_precedence < self.get_rule(&self.current.kind).lbp
            && self.current.kind != TokenKind::Eof
        {
            self.advance();
            // Extract and clone the closure before calling it
            let led = self
                .rules
                .get(&self.previous.kind)
                .and_then(|rule| rule.led.as_ref())
                .map(|arc| Arc::clone(arc)); // <-- explicit clone of Arc

            if let Some(led) = led {
                if let Some(l) = left {
                    left = led(self, l, self.previous.clone());
                }
            } else {
                break;
            }
        }

        left
    }

    // --- Example rules ---

    fn parse_literal(&mut self, token: Token) -> ParseResult {
        Some(match &token.kind {
            TokenKind::True => ASTNode::BoolLiteral { value: true },
            TokenKind::False => ASTNode::BoolLiteral { value: false },
            TokenKind::String => ASTNode::Literal {
                token: token.clone(),
            },
            _ => ASTNode::Literal { token },
        })
    }

    fn parse_grouping(&mut self, _token: Token) -> ParseResult {
        let expr = self.parse_expression(0)?;
        self.consume(TokenKind::RParen, "Expected ')'");
        Some(ASTNode::Grouping {
            expression: Box::new(expr),
        })
    }

    fn parse_variable(&mut self, token: Token) -> ParseResult {
        // TODO: parse enum constructor (like C)
        Some(ASTNode::Variable { name: token })
    }

    fn parse_binary(&mut self, left: ASTNode, token: Token) -> ParseResult {
        let precedence = self.get_rule(&token.kind).lbp;
        let right = self.parse_expression(precedence)?;

        if token.kind == TokenKind::Pipeline {
            Some(ASTNode::Pipeline {
                left: Box::new(left),
                right: Box::new(right),
            })
        } else {
            Some(ASTNode::Binary {
                left: Box::new(left),
                op: token,
                right: Box::new(right),
            })
        }
    }

    fn parse_expression_statement(&mut self) -> ParseResult {
        let expr = self.parse_expression(0)?;
        Some(ASTNode::ExpressionStatement {
            expression: Box::new(expr),
        })
    }

    fn parse_match_statement(&mut self) -> ParseResult {
        self.advance(); // consume 'match'
        let value = self.parse_expression(0)?;

        if self.current.kind != TokenKind::LBrace {
            self.error("Expected '{' after match value.");
            return None;
        }
        self.advance(); // consume '{'

        let mut arms = Vec::new();
        while self.current.kind != TokenKind::RBrace && self.current.kind != TokenKind::Eof {
            let pattern = self.parse_expression(0)?;
            if self.current.kind != TokenKind::Arrow {
                self.error("Expected '->' after pattern in match arm.");
                return None;
            }
            self.advance(); // consume '->'

            let expr = self.parse_expression(0)?;

            if self.current.kind == TokenKind::Comma {
                self.advance();
            }

            arms.push(super::ast::MatchArm {
                pattern: Box::new(pattern),
                expression: Box::new(expr),
            });
        }
        if self.current.kind != TokenKind::RBrace {
            self.error("Expected '}' after match arms.");
            return None;
        }
        self.advance(); // consume '}'
        Some(ASTNode::MatchStatement {
            value: Box::new(value),
            arms,
        })
    }

    fn parse_call(&mut self, callee: ASTNode) -> ParseResult {
        let mut arguments = Vec::new();

        if self.current.kind != TokenKind::RParen {
            loop {
                if arguments.len() >= 255 {
                    self.error("Too many arguments in function call.");
                    return None;
                }
                let arg = self.parse_expression(0)?;
                arguments.push(arg);

                if self.current.kind == TokenKind::Comma {
                    self.advance();
                }
                if self.current.kind == TokenKind::RParen || self.current.kind == TokenKind::Eof {
                    break;
                }
            }
        }
        self.consume(TokenKind::RParen, "Expected ')' after arguments.");

        Some(ASTNode::Call {
            callee: Box::new(callee),
            arguments,
        })
    }

    fn parse_block(&mut self) -> Option<Vec<ASTNode>> {
        if self.current.kind != TokenKind::LBrace {
            self.error("Expected '{' at start of block.");
            return None;
        }
        self.advance(); // consume '{'

        let mut nodes = Vec::new();
        while self.current.kind != TokenKind::RBrace && self.current.kind != TokenKind::Eof {
            if let Some(stmt) = self.parse_expression_statement() {
                nodes.push(stmt);
            } else {
                break;
            }
        }
        if self.current.kind != TokenKind::RBrace {
            self.error("Expected '}' at end of block.");
            return None;
        }
        self.advance(); // consume '}'
        Some(nodes)
    }

    fn parse_function_statement(&mut self) -> ParseResult {
        self.advance(); // consume 'func'
        let name = self.current.clone();
        if name.kind != TokenKind::Identifier {
            self.error("Expected function name after 'func'.");
            return None;
        }
        self.advance();

        if self.current.kind != TokenKind::LParen {
            self.error("Expected '(' after function name.");
            return None;
        }
        self.advance();

        // parse parameter list
        let mut params = Vec::new();
        while self.current.kind != TokenKind::RParen {
            if self.current.kind != TokenKind::Identifier {
                self.error("Expected parameter name.");
                return None;
            }
            params.push(self.current.clone());
            self.advance();
            if self.current.kind == TokenKind::Comma {
                self.advance();
            } else if self.current.kind != TokenKind::RParen {
                self.error("Expected ',' or ')'.");
                return None;
            }
        }
        self.advance(); // consume ')'

        // parse the body using the helper
        let body = self.parse_block()?;

        Some(ASTNode::FunctionStatement { name, params, body })
    }

    fn parse_lambda_expression(&mut self, _token: Token) -> ParseResult {
        if self.current.kind != TokenKind::LParen {
            self.error("Expected '(' after 'fn'.");
            return None;
        }
        self.advance();

        let mut params = Vec::new();
        while self.current.kind != TokenKind::RParen {
            if self.current.kind != TokenKind::Identifier {
                self.error("Expected parameter name.");
                return None;
            }
            params.push(self.current.clone());
            self.advance();
            if self.current.kind == TokenKind::Comma {
                self.advance();
            } else if self.current.kind != TokenKind::RParen {
                self.error("Expected ',' or ')'.");
                return None;
            }
        }
        self.advance(); // consume ')'

        if self.current.kind != TokenKind::Arrow {
            self.error("Expected '->' after lambda parameters.");
            return None;
        }
        self.advance(); // consume '->'

        if self.current.kind != TokenKind::LBrace {
            self.error("Expected '{' after '->' in lambda.");
            return None;
        }
        self.advance();

        let mut body = Vec::new();
        while self.current.kind != TokenKind::RBrace && self.current.kind != TokenKind::Eof {
            if let Some(stmt) = self.parse_expression_statement() {
                body.push(stmt);
            } else {
                break;
            }
        }
        if self.current.kind != TokenKind::RBrace {
            self.error("Expected '}' at end of block.");
            return None;
        }
        self.advance(); // consume '}'
        Some(ASTNode::LambdaExpression { params, body })
    }

    fn parse_list_literal(&mut self, _token: Token) -> ParseResult {
        let mut elements = Vec::new();

        if self.current.kind != TokenKind::RBracket {
            loop {
                let element = self.parse_expression(0)?;
                elements.push(element);

                if self.current.kind == TokenKind::Comma {
                    self.advance();
                }
                if self.current.kind == TokenKind::RBracket || self.current.kind == TokenKind::Eof {
                    break;
                }
            }
        }
        self.consume(TokenKind::RBracket, "Expected ']' after list literal.");
        Some(ASTNode::ListLiteral { elements })
    }

    fn parse_struct_literal(&mut self, _token: Token) -> ParseResult {
        let mut keys = Vec::new();
        let mut values = Vec::new();

        if self.current.kind != TokenKind::RBrace {
            loop {
                if self.current.kind != TokenKind::Identifier {
                    self.error("Expected property name in struct literal.");
                    return None;
                }
                let key = self.current.clone();
                self.advance();

                self.consume(TokenKind::Equal, "Expected '=' after property name.");
                let value = self.parse_expression(0)?;
                keys.push(key);
                values.push(value);

                if self.current.kind == TokenKind::Comma {
                    self.advance();
                }
                if self.current.kind == TokenKind::RBrace || self.current.kind == TokenKind::Eof {
                    break;
                }
            }
        }
        self.consume(TokenKind::RBrace, "Expected '}' after struct literal.");
        Some(ASTNode::StructLiteral { keys, values })
    }

    fn parse_struct_update(&mut self, base: ASTNode) -> ParseResult {
        self.advance(); // consume '{'

        let mut keys = Vec::new();
        let mut values = Vec::new();

        if self.current.kind != TokenKind::RBrace {
            loop {
                if self.current.kind != TokenKind::Identifier {
                    self.error("Expected property name in struct update.");
                    return None;
                }
                let key = self.current.clone();
                self.advance();

                self.consume(TokenKind::Equal, "Expected '=' after property name.");
                let value = self.parse_expression(0)?;
                keys.push(key);
                values.push(value);

                if self.current.kind == TokenKind::Comma {
                    self.advance();
                }
                if self.current.kind == TokenKind::RBrace || self.current.kind == TokenKind::Eof {
                    break;
                }
            }
        }

        self.consume(TokenKind::RBrace, "Expected '}' after struct update.");

        Some(ASTNode::StructUpdate {
            base: Box::new(base),
            keys,
            values,
        })
    }

    pub fn parse_let_statement(&mut self) -> ParseResult {
        //self.advance(); // consume 'let'

        // Check for let! form
        let is_bang = if self.current.kind == TokenKind::Bang {
            self.advance();
            true
        } else {
            false
        };

        let name_token = self.current.clone();
        if !matches!(self.current.kind, TokenKind::Identifier) {
            self.error("Expected variable name after 'let'.");
            return None;
        }
        self.advance();

        if self.current.kind != TokenKind::Equal {
            self.error("Expected '=' after variable name.");
            return None;
        }
        self.advance();

        let initializer = self.parse_expression(0)?;
        Some(if is_bang {
            ASTNode::LetBangStatement {
                name: name_token,
                initializer: Box::new(initializer),
            }
        } else {
            ASTNode::LetStatement {
                name: name_token,
                initializer: Box::new(initializer),
            }
        })
    }

    pub fn parse_program(&mut self) -> ASTProgram {
        let mut nodes = Vec::new();
        while self.current.kind != TokenKind::Eof && !self.had_error {
            if let Some(stmt) = self.parse_expression_statement() {
                nodes.push(stmt);
            } else {
                break;
            }
        }
        ASTProgram { nodes }
    }

    fn parse_import_statement(&mut self) -> ParseResult {
        self.advance(); // consume 'import'

        if self.current.kind != TokenKind::String {
            self.error("Parse error: Expected string literal after 'import'.");
            return None;
        }

        let path = self.current.clone();
        self.advance(); // consume the string literal

        Some(ASTNode::ImportStatement { path })
    }

    fn parse_property_access(&mut self, object: ASTNode, _token: Token) -> ParseResult {
        // After '.', expect an identifier (the property name)
        if self.current.kind != TokenKind::Identifier {
            self.error("Expected property name after '.'");
            return None;
        }
        let property = self.current.clone();
        self.advance();

        Some(ASTNode::PropertyAccess {
            object: Box::new(object),
            property,
        })
    }

    fn parse_if_expression(&mut self, _token: Token) -> ParseResult {
        let condition = self.parse_expression(0)?;

        self.consume(TokenKind::LBrace, "Expected '{' after 'if' condition.");
        let then_branch = self.parse_block()?; // Vec<ASTNode>

        let else_branch = if self.current.kind == TokenKind::Else {
            self.advance(); // consume 'else'
            if self.current.kind == TokenKind::If {
                // Parse an else if: wrap the next if in a block for uniformity
                let if_expr = self.parse_if_expression(self.current.clone())?;
                Some(vec![if_expr])
            } else {
                self.consume(TokenKind::LBrace, "Expected '{' after 'else'.");
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Some(ASTNode::IfExpression {
            condition: Box::new(condition),
            then_branch,
            else_branch,
        })
    }

    // --- Parse rule helpers ---

    fn get_rule(&self, kind: &TokenKind) -> &ParseRule<'a> {
        self.rules
            .get(kind)
            .unwrap_or_else(|| self.rules.get(&TokenKind::Eof).unwrap())
    }

    fn init_parse_rules(&mut self) {
        use TokenKind::*;
        let mut rules: HashMap<TokenKind, ParseRule<'a>> = HashMap::new();

        // --- Primary expressions ---
        rules.insert(
            Number,
            ParseRule {
                nud: Some(Arc::new(|s, t| s.parse_literal(t))),
                led: None,
                lbp: 0,
            },
        );
        rules.insert(
            True,
            ParseRule {
                nud: Some(Arc::new(|s, t| s.parse_literal(t))),
                led: None,
                lbp: 0,
            },
        );
        rules.insert(
            String,
            ParseRule {
                nud: Some(Arc::new(|s, t| s.parse_literal(t))),
                led: None,
                lbp: 0,
            },
        );
        rules.insert(
            False,
            ParseRule {
                nud: Some(Arc::new(|s, t| s.parse_literal(t))),
                led: None,
                lbp: 0,
            },
        );
        rules.insert(
            If,
            ParseRule {
                nud: Some(Arc::new(|s, t| s.parse_if_expression(t))),
                led: None,
                lbp: 0,
            },
        );

        rules.insert(
            Identifier,
            ParseRule {
                nud: Some(Arc::new(|s, t| s.parse_variable(t))),
                led: None,
                lbp: 0,
            },
        );
        rules.insert(
            LParen,
            ParseRule {
                nud: Some(Arc::new(|s, t| s.parse_grouping(t))),
                led: Some(Arc::new(|s, left, _| s.parse_call(left))),
                lbp: 30, // Function call has high precedence
            },
        );
        rules.insert(
            LBracket,
            ParseRule {
                nud: Some(Arc::new(|s, t| s.parse_list_literal(t))),
                led: None,
                lbp: 0,
            },
        );
        rules.insert(
            LBrace,
            ParseRule {
                nud: Some(Arc::new(|s, t| s.parse_struct_literal(t))),
                led: None,
                lbp: 0,
            },
        );

        // --- Statements and keywords ---
        rules.insert(
            Let,
            ParseRule {
                nud: Some(Arc::new(|s, _| s.parse_let_statement())),
                led: None,
                lbp: 0,
            },
        );
        rules.insert(
            Func,
            ParseRule {
                nud: Some(Arc::new(|s, _| s.parse_function_statement())),
                led: None,
                lbp: 0,
            },
        );
        rules.insert(
            Fn,
            ParseRule {
                nud: Some(Arc::new(|s, t| s.parse_lambda_expression(t))),
                led: None,
                lbp: 0,
            },
        );
        rules.insert(
            Import,
            ParseRule {
                nud: Some(Arc::new(|s, _| s.parse_import_statement())),
                led: None,
                lbp: 0,
            },
        );
        rules.insert(
            Match,
            ParseRule {
                nud: Some(Arc::new(|s, _| s.parse_match_statement())),
                led: None,
                lbp: 0,
            },
        );
        // Enum parsing can be added similarly if desired

        // --- Binary/infix operators ---
        rules.insert(
            Plus,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, l, t| s.parse_binary(l, t))),
                lbp: 10,
            },
        );
        rules.insert(
            Minus,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, l, t| s.parse_binary(l, t))),
                lbp: 10,
            },
        );
        rules.insert(
            Star,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, l, t| s.parse_binary(l, t))),
                lbp: 20,
            },
        );
        rules.insert(
            Slash,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, l, t| s.parse_binary(l, t))),
                lbp: 20,
            },
        );
        rules.insert(
            LArrow,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, l, _| s.parse_struct_update(l))),
                lbp: 50,
            },
        );

        rules.insert(
            Dot,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, left, token| {
                    s.parse_property_access(left, token)
                })),
                lbp: 40, // Give dot higher precedence than +, -, etc
            },
        );

        rules.insert(
            Eof,
            ParseRule {
                nud: Some(Arc::new(|_, _| None)),
                led: None,
                lbp: 0,
            },
        );

        // Attach to parser
        self.rules = rules;
    }
}
