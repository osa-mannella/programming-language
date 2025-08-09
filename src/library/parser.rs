use super::ast::{ASTNode, ASTProgram};
use super::lexer::{Lexer, Token, TokenKind, TokenValue};
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
    pub had_error: bool,
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
            rules: HashMap::new(),
        };
        parser.init_parse_rules();
        parser
    }

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

    fn handle_token_error(&mut self, token: &Token) {
        match &token.value {
            TokenValue::Error(msg) => {
                self.error(&msg.clone());
            }
            _ => {
                self.error(format!("{:?}", token.value).as_str());
            }
        }
    }

    pub fn parse_expression(&mut self, min_precedence: u8) -> ParseResult {
        self.advance();

        let nud = self
            .rules
            .get(&self.previous.kind)
            .and_then(|rule| rule.nud.as_ref())
            .cloned();

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
            let led = self
                .rules
                .get(&self.previous.kind)
                .and_then(|rule| rule.led.as_ref())
                .map(|arc| Arc::clone(arc));

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

    fn parse_literal(&mut self, token: Token) -> ParseResult {
        Some(match &token.kind {
            TokenKind::True => ASTNode::BoolLiteral { value: true },
            TokenKind::False => ASTNode::BoolLiteral { value: false },
            TokenKind::String => ASTNode::Literal {
                token: token.clone(),
            },
            TokenKind::InterpolatedString => self.parse_interpolated_string(token)?,
            _ => ASTNode::Literal { token },
        })
    }

    fn parse_interpolated_string(&mut self, initial_token: Token) -> ParseResult {
        let mut parts = Vec::new();

        if let TokenValue::String(s) = &initial_token.value {
            // Parse the interpolated string content
            let mut chars = s.chars().peekable();
            let mut current_str = String::new();

            while let Some(ch) = chars.next() {
                if ch == '$' && chars.peek() == Some(&'{') {
                    // Add current string part if not empty
                    if !current_str.is_empty() {
                        parts.push(ASTNode::Literal {
                            token: Token {
                                kind: TokenKind::String,
                                value: TokenValue::String(current_str.clone()),
                                line: initial_token.line,
                            },
                        });
                        current_str.clear();
                    }

                    chars.next(); // consume '{'

                    // Extract expression until '}'
                    let mut expr_str = String::new();
                    let mut brace_count = 1;

                    while let Some(ch) = chars.next() {
                        if ch == '{' {
                            brace_count += 1;
                            expr_str.push(ch);
                        } else if ch == '}' {
                            brace_count -= 1;
                            if brace_count == 0 {
                                break;
                            }
                            expr_str.push(ch);
                        } else {
                            expr_str.push(ch);
                        }
                    }

                    if brace_count != 0 {
                        self.error("Unclosed interpolation expression");
                        return None;
                    }

                    // Parse the expression string
                    if !expr_str.is_empty() {
                        let lexer = Lexer::new(&expr_str);
                        let mut parser = Parser::new(lexer);
                        if let Some(expr) = parser.parse_expression(0) {
                            parts.push(expr);
                        } else {
                            self.error("Invalid expression in string interpolation");
                            return None;
                        }
                    }
                } else {
                    current_str.push(ch);
                }
            }

            // Add remaining string part if not empty
            if !current_str.is_empty() {
                parts.push(ASTNode::Literal {
                    token: Token {
                        kind: TokenKind::String,
                        value: TokenValue::String(current_str),
                        line: initial_token.line,
                    },
                });
            }
        }

        Some(ASTNode::StringInterpolation { parts })
    }

    fn parse_grouping(&mut self, _token: Token) -> ParseResult {
        let expr = self.parse_expression(0)?;
        self.consume(TokenKind::RParen, "Expected ')'");
        Some(ASTNode::Grouping {
            expression: Box::new(expr),
        })
    }

    fn parse_variable(&mut self, token: Token, parse_enum_constructor: bool) -> ParseResult {
        if self.current.kind == TokenKind::DoubleColon && parse_enum_constructor {
            return self.parse_enum_constructor_with_leading(token);
        }
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
        let value = self.parse_expression(0)?;

        if self.current.kind != TokenKind::LBrace {
            self.error("Expected '{' after match value.");
            return None;
        }
        self.advance(); // consume '{'

        let mut arms = Vec::new();
        while self.current.kind != TokenKind::RBrace && self.current.kind != TokenKind::Eof {
            let mut patterns = Vec::new();
            let first_pattern = self.parse_pattern()?;
            patterns.push(first_pattern.clone());
            
            while self.current.kind == TokenKind::Pipe {
                // Check if the first pattern was a struct deconstruction
                if matches!(first_pattern, ASTNode::StructDeconstructPattern { .. }) {
                    self.error("Struct patterns cannot be combined with other patterns using OR operator.");
                    return None;
                }
                
                self.advance();
                let next_pattern = self.parse_pattern()?;
                
                // Check if any pattern in the OR chain is a struct deconstruction
                if matches!(next_pattern, ASTNode::StructDeconstructPattern { .. }) {
                    self.error("Struct patterns cannot be combined with other patterns using OR operator.");
                    return None;
                }
                
                patterns.push(next_pattern);
            }
            if patterns.is_empty() {
                self.error("Expected at least one pattern in match arm.");
                return None;
            }
            if self.current.kind != TokenKind::Arrow {
                self.error("Expected '->' after pattern in match arm.");
                return None;
            }
            self.advance(); // consume '->'

            let mut exprs = Vec::new();
            if self.current.kind == TokenKind::LBrace {
                exprs.extend(self.parse_block()?);
            } else {
                exprs.push(self.parse_expression(0)?);
            }

            if self.current.kind == TokenKind::Comma {
                self.advance();
            }

            arms.push(super::ast::MatchArm {
                patterns: patterns,
                expression: exprs,
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
                self.handle_token_error(&self.previous.clone());
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

    fn parse_async(&mut self) -> ParseResult {
        let kw = self.current.clone();
        match kw.kind {
            TokenKind::Func => {
                self.advance(); // consume `func`
                match self.parse_function_statement() {
                    Some(ASTNode::FunctionStatement { name, params, body }) => {
                        Some(ASTNode::AsyncFunctionStatement { name, params, body })
                    }
                    other => other,
                }
            }
            TokenKind::Fn => {
                self.advance(); // consume `fn`
                let fn_tok = kw;
                match self.parse_lambda_expression(fn_tok) {
                    Some(ASTNode::LambdaExpression { params, body }) => {
                        Some(ASTNode::AsyncLambdaExpression { params, body })
                    }
                    other => other,
                }
            }
            _ => {
                self.error("Expected 'func' or 'fn' after 'async'");
                None
            }
        }
    }

    fn parse_await_expression(&mut self, _tok: Token) -> ParseResult {
        let expr = self.parse_expression(0)?;
        Some(ASTNode::AwaitExpression {
            expression: Box::new(expr),
        })
    }

    fn parse_lambda_expression(&mut self, _token: Token) -> ParseResult {
        // we’ve just consumed the `fn`
        if self.current.kind != TokenKind::LParen {
            self.error("Expected '(' after 'fn'.");
            return None;
        }
        self.advance(); // consume '('

        // gather parameters
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

        // arrow
        if self.current.kind != TokenKind::Arrow {
            self.error("Expected '->' after lambda parameters.");
            return None;
        }
        self.advance(); // consume '->'

        // now decide whether this is a block or a single expression
        let mut body = Vec::new();
        if self.current.kind == TokenKind::LBrace {
            // block form: `{ … }`
            self.advance(); // consume '{'
            while self.current.kind != TokenKind::RBrace && self.current.kind != TokenKind::Eof {
                if let Some(stmt) = self.parse_expression_statement() {
                    body.push(stmt);
                } else {
                    break;
                }
            }
            if self.current.kind != TokenKind::RBrace {
                self.error("Expected '}' at end of lambda block.");
                return None;
            }
            self.advance(); // consume '}'
        } else {
            // single‐line form: just parse one expression and wrap it
            let expr = self.parse_expression(0)?;
            body.push(ASTNode::ExpressionStatement {
                expression: Box::new(expr),
            });
        }

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

    fn parse_array_append(&mut self, base: ASTNode) -> ParseResult {
        self.advance(); // consume '['

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

        self.consume(TokenKind::RBracket, "Expected ']' after array append.");

        Some(ASTNode::ArrayAppend {
            base: Box::new(base),
            elements,
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
            if let Some(stmt) = self.parse_expression(0) {
                nodes.push(stmt);
            } else {
                break;
            }
        }
        ASTProgram { nodes }
    }

    fn parse_import_statement(&mut self) -> ParseResult {
        // 'import' token is already consumed by the parser

        if self.current.kind != TokenKind::String {
            self.error("Parse error: Expected string literal after 'import'.");
            return None;
        }

        let path = self.current.clone();
        self.advance(); // consume the string literal

        Some(ASTNode::ImportStatement { path })
    }

    fn parse_property_access(&mut self, object: ASTNode, _token: Token) -> ParseResult {
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

    fn parse_index_access(&mut self, object: ASTNode, _token: Token) -> ParseResult {
        let index = self.parse_expression(0)?;
        self.consume(TokenKind::RBracket, "Expected ']' after index expression");

        Some(ASTNode::IndexAccess {
            object: Box::new(object),
            index: Box::new(index),
        })
    }

    fn parse_pattern(&mut self) -> Option<ASTNode> {
        // Wildcard pattern
        if self.current.kind == TokenKind::Underscore {
            self.advance();
            return Some(ASTNode::WildcardPattern);
        }

        // Struct deconstruction pattern
        if self.current.kind == TokenKind::LBrace {
            self.advance(); // skip '{'
            let mut field_names = Vec::new();
            
            // Ensure we have at least one field
            if self.current.kind == TokenKind::RBrace {
                self.error("Empty struct patterns are not allowed.");
                return None;
            }
            
            while self.current.kind != TokenKind::RBrace {
                if self.current.kind != TokenKind::Identifier {
                    self.error("Expected field name in struct pattern.");
                    return None;
                }
                field_names.push(self.current.clone());
                self.advance();

                if self.current.kind == TokenKind::Comma {
                    self.advance();
                } else if self.current.kind != TokenKind::RBrace {
                    self.error("Expected ',' or '}' in struct pattern.");
                    return None;
                }
            }
            self.advance(); // skip '}'
            return Some(ASTNode::StructDeconstructPattern { field_names });
        }

        // Enum destructor pattern
        if self.current.kind == TokenKind::Identifier {
            let name = self.current.clone();
            self.advance();
            if self.current.kind == TokenKind::LParen {
                let variable = self.parse_variable(name, false)?;
                return self.parse_call(variable);
            }
            if self.current.kind == TokenKind::DoubleColon {
                self.advance();

                let variant_name = self.current.clone();
                if variant_name.kind != TokenKind::Identifier {
                    self.error("Expected variant name in enum pattern.");
                    return None;
                }
                self.advance();

                if self.current.kind == TokenKind::LBrace {
                    let mut field_names = Vec::new();
                    self.advance(); // skip '{'
                    while self.current.kind != TokenKind::RBrace {
                        if self.current.kind != TokenKind::Identifier {
                            self.error("Expected field name in enum pattern.");
                            return None;
                        }
                        field_names.push(self.current.clone());
                        self.advance();

                        if self.current.kind == TokenKind::Comma {
                            self.advance();
                        } else if self.current.kind != TokenKind::RBrace {
                            self.error("Expected ',' or '}' in enum pattern.");
                            return None;
                        }
                    }
                    self.advance(); // skip '}'
                    return Some(ASTNode::EnumDeconstructPattern {
                        enum_name: name,
                        variant_name,
                        field_names,
                    });
                } else {
                    // Unit variant (no fields)
                    return Some(ASTNode::EnumDeconstructPattern {
                        enum_name: name,
                        variant_name,
                        field_names: Vec::new(),
                    });
                }
            }
            // Just an identifier (could be variable binding in pattern)
            return Some(ASTNode::Variable { name });
        }

        // Literal pattern (number, string, boolean, etc.)
        if matches!(
            self.current.kind,
            TokenKind::Number | TokenKind::String | TokenKind::True | TokenKind::False
        ) {
            let tok = self.current.clone();
            self.advance();
            return self.parse_literal(tok);
        }

        // Not a valid pattern
        self.error("Invalid pattern: expected literal, struct pattern, enum deconstructor, wildcard, or variable binding.");
        None
    }

    fn parse_if_expression(&mut self, _token: Token) -> ParseResult {
        let condition = self.parse_expression(0)?;
        let then_branch = self.parse_block()?; // Vec<ASTNode>

        let else_branch = if self.current.kind == TokenKind::Else {
            self.advance(); // consume 'else'
            if self.current.kind == TokenKind::If {
                // Parse an else if: wrap the next if in a block for uniformity
                let if_expr = self.parse_if_expression(self.current.clone())?;
                Some(vec![if_expr])
            } else {
                println!("{:?}", self.current);
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

    fn parse_enum_constructor_with_leading(&mut self, enum_name: Token) -> ParseResult {
        // At this point, self.current should be ColonColon
        self.advance(); // skip '::'

        // Parse variant name
        let variant_name = self.current.clone();
        if variant_name.kind != TokenKind::Identifier {
            self.error("Expected variant name in enum constructor.");
            return None;
        }
        self.advance();

        let mut field_names = Vec::new();
        let mut values = Vec::new();

        if self.current.kind == TokenKind::LBrace {
            self.advance();
            while self.current.kind != TokenKind::RBrace {
                // Parse field name
                if self.current.kind != TokenKind::Identifier {
                    self.error("Expected field name in enum constructor.");
                    return None;
                }
                field_names.push(self.current.clone());
                self.advance();

                if self.current.kind != TokenKind::Equal {
                    self.error("Expected '=' after field name in enum constructor.");
                    return None;
                }
                self.advance();

                if let Some(value) = self.parse_expression(0) {
                    values.push(value);
                } else {
                    self.error("Expected value in enum constructor.");
                    return None;
                }

                if self.current.kind == TokenKind::Comma {
                    self.advance();
                } else if self.current.kind != TokenKind::RBrace {
                    self.error("Expected ',' or '}' in enum constructor.");
                    return None;
                }
            }
            self.advance(); // skip '}'
        }

        Some(ASTNode::EnumConstructor {
            enum_name,
            variant_name,
            field_names,
            values,
        })
    }

    fn parse_enum_statement(&mut self) -> ParseResult {
        let name = self.current.clone();
        if name.kind != TokenKind::Identifier {
            self.error("Expected enum name after 'enum'.");
            return None;
        }
        self.advance();

        if self.current.kind != TokenKind::LBrace {
            self.error("Expected '{' after enum name.");
            return None;
        }
        self.advance();

        let mut variant_names = Vec::new();
        let mut field_names = Vec::new();
        let mut field_counts = Vec::new();

        while self.current.kind != TokenKind::RBrace {
            // Parse the variant name
            if self.current.kind != TokenKind::Identifier {
                self.error("Expected variant name in enum declaration.");
                return None;
            }
            variant_names.push(self.current.clone());
            self.advance();

            let mut fields = Vec::new();

            // Only allow struct-style fields (curly braces)
            if self.current.kind == TokenKind::LBrace {
                self.advance();
                while self.current.kind != TokenKind::RBrace {
                    if self.current.kind != TokenKind::Identifier {
                        self.error("Expected field name in struct variant.");
                        return None;
                    }
                    fields.push(self.current.clone());
                    self.advance();

                    if self.current.kind == TokenKind::Comma {
                        self.advance();
                    } else if self.current.kind != TokenKind::RBrace {
                        self.error("Expected ',' or '}' in struct variant.");
                        return None;
                    }
                }
                self.advance(); // consume '}'
            }

            field_counts.push(fields.len());
            field_names.push(fields);

            // Comma or end of enum
            if self.current.kind == TokenKind::Comma {
                self.advance();
            } else if self.current.kind != TokenKind::RBrace {
                self.error("Expected ',' or '}' in enum declaration.");
                return None;
            }
        }

        self.advance(); // consume closing '}'

        Some(ASTNode::EnumStatement {
            name,
            variant_names,
            field_names,
            field_counts,
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
            InterpolatedString,
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
                nud: Some(Arc::new(|s, t| s.parse_variable(t, true))),
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
                led: Some(Arc::new(|s, left, token| s.parse_index_access(left, token))),
                lbp: 40, // Same precedence as property access
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
        rules.insert(
            Async,
            ParseRule {
                nud: Some(Arc::new(|s, _| s.parse_async())),
                led: None,
                lbp: 0,
            },
        );

        rules.insert(
            Await,
            ParseRule {
                nud: Some(Arc::new(|s, t| s.parse_await_expression(t))),
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
            Caret,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, l, t| s.parse_binary(l, t))),
                lbp: 50, // High precedence for power operator 
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

        // Comparison operators
        rules.insert(
            EqualEqual,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, l, t| s.parse_binary(l, t))),
                lbp: 8,
            },
        );
        rules.insert(
            BangEqual,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, l, t| s.parse_binary(l, t))),
                lbp: 8,
            },
        );
        rules.insert(
            Less,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, l, t| s.parse_binary(l, t))),
                lbp: 9,
            },
        );
        rules.insert(
            LessEqual,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, l, t| s.parse_binary(l, t))),
                lbp: 9,
            },
        );
        rules.insert(
            Greater,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, l, t| s.parse_binary(l, t))),
                lbp: 9,
            },
        );
        rules.insert(
            GreaterEqual,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, l, t| s.parse_binary(l, t))),
                lbp: 9,
            },
        );

        // Logical operators
        rules.insert(
            And,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, l, t| s.parse_binary(l, t))),
                lbp: 6,
            },
        );
        rules.insert(
            Or,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, l, t| s.parse_binary(l, t))),
                lbp: 5,
            },
        );
        rules.insert(
            LArrow,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, l, _| {
                    if s.current.kind == TokenKind::LBracket {
                        s.parse_array_append(l)
                    } else {
                        s.parse_struct_update(l)
                    }
                })),
                lbp: 50,
            },
        );

        rules.insert(
            Pipeline,
            ParseRule {
                nud: None,
                led: Some(Arc::new(|s, l, t| s.parse_binary(l, t))),
                lbp: 30, // Pipeline has high precedence
            },
        );
        rules.insert(
            Enum,
            ParseRule {
                nud: Some(Arc::new(|s, _| s.parse_enum_statement())),
                led: None,
                lbp: 0,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::lexer::*;

    fn parse_source(source: &str) -> Result<ASTProgram, String> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        if parser.had_error {
            Err("Parser encountered errors".to_string())
        } else {
            Ok(program)
        }
    }

    fn parse_expression(source: &str) -> Result<ASTNode, String> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);

        // For statements like "if", we need to parse as a full program, not just an expression
        let program = parser.parse_program();

        if parser.had_error {
            Err("Parser encountered errors".to_string())
        } else if program.nodes.is_empty() {
            Err("No nodes parsed".to_string())
        } else {
            Ok(program.nodes[0].clone())
        }
    }

    #[test]
    fn test_literal_parsing() {
        // Numbers
        let result = parse_expression("42").unwrap();
        if let ASTNode::Literal { token } = result {
            assert_eq!(token.kind, TokenKind::Number);
            if let TokenValue::Number(val) = token.value {
                assert_eq!(val, 42.0);
            }
        } else {
            panic!("Expected literal node for number");
        }

        // Strings
        let result = parse_expression(r#""hello""#).unwrap();
        if let ASTNode::Literal { token } = result {
            assert_eq!(token.kind, TokenKind::String);
        } else {
            panic!("Expected literal node for string");
        }

        // Booleans
        let result = parse_expression("true").unwrap();
        if let ASTNode::BoolLiteral { value } = result {
            assert!(value);
        } else {
            panic!("Expected bool literal node");
        }

        let result = parse_expression("false").unwrap();
        if let ASTNode::BoolLiteral { value } = result {
            assert!(!value);
        } else {
            panic!("Expected bool literal node");
        }
    }

    #[test]
    fn test_variable_parsing() {
        let result = parse_expression("my_variable").unwrap();
        if let ASTNode::Variable { name } = result {
            if let TokenValue::Identifier(name_str) = name.value {
                assert_eq!(name_str, "my_variable");
            }
        } else {
            panic!("Expected variable node");
        }
    }

    #[test]
    fn test_binary_operations() {
        let result = parse_expression("1 + 2").unwrap();
        if let ASTNode::Binary { left, op, right } = result {
            assert_eq!(op.kind, TokenKind::Plus);
            // Verify left and right are literals
            matches!(left.as_ref(), ASTNode::Literal { .. });
            matches!(right.as_ref(), ASTNode::Literal { .. });
        } else {
            panic!("Expected binary operation node");
        }

        // Test precedence
        let result = parse_expression("1 + 2 * 3").unwrap();
        if let ASTNode::Binary { left, op, right } = result {
            assert_eq!(op.kind, TokenKind::Plus);
            matches!(left.as_ref(), ASTNode::Literal { .. });
            matches!(right.as_ref(), ASTNode::Binary { .. });
        } else {
            panic!("Expected binary operation with correct precedence");
        }
    }

    #[test]
    fn test_grouping() {
        let result = parse_expression("(1 + 2)").unwrap();
        if let ASTNode::Grouping { expression } = result {
            matches!(expression.as_ref(), ASTNode::Binary { .. });
        } else {
            panic!("Expected grouping node");
        }
    }

    #[test]
    fn test_function_calls() {
        let result = parse_expression("my_func()").unwrap();
        if let ASTNode::Call { callee, arguments } = result {
            matches!(callee.as_ref(), ASTNode::Variable { .. });
            assert_eq!(arguments.len(), 0);
        } else {
            panic!("Expected call node");
        }

        let result = parse_expression("my_func(1, 2, 3)").unwrap();
        if let ASTNode::Call { callee, arguments } = result {
            matches!(callee.as_ref(), ASTNode::Variable { .. });
            assert_eq!(arguments.len(), 3);
        } else {
            panic!("Expected call node with arguments");
        }
    }

    #[test]
    fn test_property_access() {
        let result = parse_expression("obj.property").unwrap();
        if let ASTNode::PropertyAccess { object, property } = result {
            matches!(object.as_ref(), ASTNode::Variable { .. });
            if let TokenValue::Identifier(prop_name) = property.value {
                assert_eq!(prop_name, "property");
            }
        } else {
            panic!("Expected property access node");
        }
    }

    #[test]
    fn test_list_literal() {
        let result = parse_expression("[]").unwrap();
        if let ASTNode::ListLiteral { elements } = result {
            assert_eq!(elements.len(), 0);
        } else {
            panic!("Expected empty list literal");
        }

        let result = parse_expression("[1, 2, 3]").unwrap();
        if let ASTNode::ListLiteral { elements } = result {
            assert_eq!(elements.len(), 3);
        } else {
            panic!("Expected list literal with elements");
        }
    }

    #[test]
    fn test_index_access() {
        let result = parse_expression("arr[0]").unwrap();
        if let ASTNode::IndexAccess { object, index } = result {
            matches!(object.as_ref(), ASTNode::Variable { .. });
            matches!(index.as_ref(), ASTNode::Literal { .. });
        } else {
            panic!("Expected index access node");
        }

        let result = parse_expression("list[i + 1]").unwrap();
        if let ASTNode::IndexAccess { object, index } = result {
            matches!(object.as_ref(), ASTNode::Variable { .. });
            matches!(index.as_ref(), ASTNode::Binary { .. });
        } else {
            panic!("Expected index access with binary expression");
        }
    }

    #[test]
    fn test_struct_literal() {
        let result = parse_expression("{}").unwrap();
        if let ASTNode::StructLiteral { keys, values } = result {
            assert_eq!(keys.len(), 0);
            assert_eq!(values.len(), 0);
        } else {
            panic!("Expected empty struct literal");
        }

        let result = parse_expression(r#"{ name = "John", age = 30 }"#).unwrap();
        if let ASTNode::StructLiteral { keys, values } = result {
            assert_eq!(keys.len(), 2);
            assert_eq!(values.len(), 2);

            if let TokenValue::Identifier(key1) = &keys[0].value {
                assert_eq!(key1, "name");
            }
            if let TokenValue::Identifier(key2) = &keys[1].value {
                assert_eq!(key2, "age");
            }
        } else {
            panic!("Expected struct literal with fields");
        }
    }

    #[test]
    fn test_let_statement() {
        let result = parse_source("let x = 42").unwrap();
        assert_eq!(result.nodes.len(), 1);

        if let ASTNode::LetStatement { name, initializer } = &result.nodes[0] {
            if let TokenValue::Identifier(var_name) = &name.value {
                assert_eq!(var_name, "x");
            }
            matches!(initializer.as_ref(), ASTNode::Literal { .. });
        } else {
            panic!("Expected let statement");
        }
    }

    #[test]
    fn test_let_bang_statement() {
        let result = parse_source("let! x = 42").unwrap();
        assert_eq!(result.nodes.len(), 1);

        if let ASTNode::LetBangStatement { name, initializer } = &result.nodes[0] {
            if let TokenValue::Identifier(var_name) = &name.value {
                assert_eq!(var_name, "x");
            }
            matches!(initializer.as_ref(), ASTNode::Literal { .. });
        } else {
            panic!("Expected let! statement");
        }
    }

    #[test]
    fn test_function_statement() {
        let result = parse_source("func test(x, y) { x + y }").unwrap();
        assert_eq!(result.nodes.len(), 1);

        if let ASTNode::FunctionStatement { name, params, body } = &result.nodes[0] {
            if let TokenValue::Identifier(func_name) = &name.value {
                assert_eq!(func_name, "test");
            }
            assert_eq!(params.len(), 2);
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected function statement");
        }
    }

    #[test]
    fn test_if_expression() {
        let result = parse_expression("if true { 42 }").unwrap();
        if let ASTNode::IfExpression {
            condition,
            then_branch,
            else_branch,
        } = result
        {
            matches!(condition.as_ref(), ASTNode::BoolLiteral { .. });
            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_none());
        } else {
            panic!("Expected if expression");
        }

        let result = parse_expression("if true { 42 } else { 0 }").unwrap();
        if let ASTNode::IfExpression {
            condition,
            then_branch,
            else_branch,
        } = result
        {
            matches!(condition.as_ref(), ASTNode::BoolLiteral { .. });
            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_some());
            assert_eq!(else_branch.unwrap().len(), 1);
        } else {
            panic!("Expected if-else expression");
        }
    }

    #[test]
    fn test_lambda_expression() {
        let result = parse_expression("fn() -> 42").unwrap();
        if let ASTNode::LambdaExpression { params, body } = result {
            assert_eq!(params.len(), 0);
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected lambda expression");
        }

        let result = parse_expression("fn(x, y) -> x + y").unwrap();
        if let ASTNode::LambdaExpression { params, body } = result {
            assert_eq!(params.len(), 2);
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected lambda expression with parameters");
        }

        let result = parse_expression("fn(x) -> { x * 2 }").unwrap();
        if let ASTNode::LambdaExpression { params, body } = result {
            assert_eq!(params.len(), 1);
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected lambda expression with block");
        }
    }

    #[test]
    fn test_async_function() {
        let result = parse_source("async func test() { await something() }").unwrap();
        assert_eq!(result.nodes.len(), 1);

        if let ASTNode::AsyncFunctionStatement { name, params, body } = &result.nodes[0] {
            if let TokenValue::Identifier(func_name) = &name.value {
                assert_eq!(func_name, "test");
            }
            assert_eq!(params.len(), 0);
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected async function statement");
        }
    }

    #[test]
    fn test_async_lambda() {
        let result = parse_expression("async fn() -> await something()").unwrap();
        if let ASTNode::AsyncLambdaExpression { params, body } = result {
            assert_eq!(params.len(), 0);
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected async lambda expression");
        }
    }

    #[test]
    fn test_await_expression() {
        let result = parse_expression("await my_async_func()").unwrap();
        if let ASTNode::AwaitExpression { expression } = result {
            matches!(expression.as_ref(), ASTNode::Call { .. });
        } else {
            panic!("Expected await expression");
        }
    }

    #[test]
    fn test_match_statement() {
        let result = parse_expression("match x { 1 -> \"one\", 2 -> \"two\" }").unwrap();
        if let ASTNode::MatchStatement { value, arms } = result {
            matches!(value.as_ref(), ASTNode::Variable { .. });
            assert_eq!(arms.len(), 2);
        } else {
            panic!("Expected match statement");
        }
    }

    #[test]
    fn test_enum_statement() {
        let result =
            parse_source("enum Shape { Circle { radius }, Rectangle { width, height } }").unwrap();
        assert_eq!(result.nodes.len(), 1);

        if let ASTNode::EnumStatement {
            name,
            variant_names,
            field_names,
            field_counts,
        } = &result.nodes[0]
        {
            if let TokenValue::Identifier(enum_name) = &name.value {
                assert_eq!(enum_name, "Shape");
            }
            assert_eq!(variant_names.len(), 2);
            assert_eq!(field_names.len(), 2);
            assert_eq!(field_counts.len(), 2);
            assert_eq!(field_counts[0], 1); // Circle has 1 field
            assert_eq!(field_counts[1], 2); // Rectangle has 2 fields
        } else {
            panic!("Expected enum statement");
        }
    }

    #[test]
    fn test_enum_constructor() {
        let result = parse_expression("Shape::Circle { radius = 5.0 }").unwrap();
        if let ASTNode::EnumConstructor {
            enum_name,
            variant_name,
            field_names,
            values,
        } = result
        {
            if let TokenValue::Identifier(enum_name_str) = enum_name.value {
                assert_eq!(enum_name_str, "Shape");
            }
            if let TokenValue::Identifier(variant_name_str) = variant_name.value {
                assert_eq!(variant_name_str, "Circle");
            }
            assert_eq!(field_names.len(), 1);
            assert_eq!(values.len(), 1);
        } else {
            panic!("Expected enum constructor");
        }
    }

    #[test]
    fn test_struct_update() {
        let result = parse_expression("person <- { age = 31 }").unwrap();
        if let ASTNode::StructUpdate { base, keys, values } = result {
            matches!(base.as_ref(), ASTNode::Variable { .. });
            assert_eq!(keys.len(), 1);
            assert_eq!(values.len(), 1);
        } else {
            panic!("Expected struct update");
        }
    }

    #[test]
    fn test_array_append() {
        let result = parse_expression("arr <- [4, 5, 6]").unwrap();
        if let ASTNode::ArrayAppend { base, elements } = result {
            matches!(base.as_ref(), ASTNode::Variable { .. });
            assert_eq!(elements.len(), 3);
        } else {
            panic!("Expected array append");
        }
    }

    #[test]
    fn test_pipeline() {
        let result = parse_expression("value |> transform |> process").unwrap();
        if let ASTNode::Pipeline { left, right } = result {
            if let ASTNode::Pipeline {
                left: inner_left,
                right: inner_right,
            } = left.as_ref()
            {
                matches!(inner_left.as_ref(), ASTNode::Variable { .. });
                matches!(inner_right.as_ref(), ASTNode::Variable { .. });
            } else {
                panic!("Expected nested pipeline");
            }
            matches!(right.as_ref(), ASTNode::Variable { .. });
        } else {
            panic!("Expected pipeline");
        }
    }

    #[test]
    fn test_import_statement() {
        let result = parse_source(r#"import "IO""#).unwrap();
        assert_eq!(result.nodes.len(), 1);

        if let ASTNode::ImportStatement { path } = &result.nodes[0] {
            if let TokenValue::String(path_str) = &path.value {
                assert_eq!(path_str, "IO");
            }
        } else {
            panic!("Expected import statement");
        }
    }

    #[test]
    fn test_power_operator() {
        let result = parse_expression("2 ^ 3").unwrap();
        if let ASTNode::Binary { left, op, right } = result {
            assert_eq!(op.kind, TokenKind::Caret);
            matches!(left.as_ref(), ASTNode::Literal { .. });
            matches!(right.as_ref(), ASTNode::Literal { .. });
        } else {
            panic!("Expected power operation");
        }
    }

    #[test]
    fn test_comparison_operators() {
        let operators = vec![
            ("==", TokenKind::EqualEqual),
            ("!=", TokenKind::BangEqual),
            ("<", TokenKind::Less),
            ("<=", TokenKind::LessEqual),
            (">", TokenKind::Greater),
            (">=", TokenKind::GreaterEqual),
        ];

        for (op_str, expected_kind) in operators {
            let source = format!("1 {} 2", op_str);
            let result = parse_expression(&source).unwrap();
            if let ASTNode::Binary { op, .. } = result {
                assert_eq!(op.kind, expected_kind);
            } else {
                panic!("Expected binary operation for {}", op_str);
            }
        }
    }

    #[test]
    fn test_logical_operators() {
        let result = parse_expression("true && false").unwrap();
        if let ASTNode::Binary { op, .. } = result {
            assert_eq!(op.kind, TokenKind::And);
        } else {
            panic!("Expected logical AND");
        }

        let result = parse_expression("true || false").unwrap();
        if let ASTNode::Binary { op, .. } = result {
            assert_eq!(op.kind, TokenKind::Or);
        } else {
            panic!("Expected logical OR");
        }
    }

    #[test]
    fn test_complex_expressions() {
        // Test nested function calls with complex arguments
        let result = parse_expression("func1(func2(x + y), z * 2)").unwrap();
        if let ASTNode::Call { callee, arguments } = result {
            matches!(callee.as_ref(), ASTNode::Variable { .. });
            assert_eq!(arguments.len(), 2);
            matches!(arguments[0], ASTNode::Call { .. });
            matches!(arguments[1], ASTNode::Binary { .. });
        } else {
            panic!("Expected complex nested call");
        }

        // Test method chaining
        let result = parse_expression("obj.method1().method2().property").unwrap();
        if let ASTNode::PropertyAccess { .. } = result {
            // This should parse correctly due to left-associativity
        } else {
            panic!("Expected method chaining");
        }
    }

    #[test]
    fn test_precedence_and_associativity() {
        // Test that multiplication has higher precedence than addition
        let result = parse_expression("1 + 2 * 3").unwrap();
        if let ASTNode::Binary { left, op, right } = result {
            assert_eq!(op.kind, TokenKind::Plus);
            matches!(left.as_ref(), ASTNode::Literal { .. });
            if let ASTNode::Binary { op: inner_op, .. } = right.as_ref() {
                assert_eq!(inner_op.kind, TokenKind::Star);
            } else {
                panic!("Expected multiplication to bind tighter");
            }
        } else {
            panic!("Expected addition at top level");
        }

        // Test power operator precedence (right-associative)
        let result = parse_expression("2 ^ 3 ^ 2").unwrap();
        if let ASTNode::Binary { left, op, right } = result {
            assert_eq!(op.kind, TokenKind::Caret);
            matches!(left.as_ref(), ASTNode::Literal { .. });
            matches!(right.as_ref(), ASTNode::Binary { .. });
        } else {
            panic!("Expected power operator");
        }
    }

    #[test]
    fn test_error_recovery() {
        // Test that parser handles various syntax errors gracefully
        assert!(parse_expression("(").is_err());
        assert!(parse_expression("func(").is_err());
        assert!(parse_expression("[1, 2,").is_err());
        assert!(parse_expression("{ name =").is_err());
        assert!(parse_source("let = 42").is_err());
    }

    #[test]
    fn test_multiline_constructs() {
        let source = r#"
func multiline(x, y) {
    if x > y {
        x - y
    } else {
        y - x
    }
}
        "#;

        let result = parse_source(source).unwrap();
        assert_eq!(result.nodes.len(), 1);

        if let ASTNode::FunctionStatement { body, .. } = &result.nodes[0] {
            assert_eq!(body.len(), 1);
            matches!(body[0], ASTNode::ExpressionStatement { .. });
        } else {
            panic!("Expected multiline function");
        }
    }

    #[test]
    fn test_empty_program() {
        let result = parse_source("").unwrap();
        assert_eq!(result.nodes.len(), 0);
    }

    #[test]
    fn test_whitespace_and_comments() {
        // Parser should handle extra whitespace gracefully
        let result = parse_source("  let   x   =   42   ").unwrap();
        assert_eq!(result.nodes.len(), 1);
        matches!(result.nodes[0], ASTNode::LetStatement { .. });
    }

    #[test]
    fn test_string_interpolation_simple() {
        let result = parse_expression(r#"$"Hello ${name}!""#).unwrap();
        if let ASTNode::StringInterpolation { parts } = result {
            assert_eq!(parts.len(), 3); // "Hello ", name, "!"
            // First part should be a string literal
            if let ASTNode::Literal { token } = &parts[0] {
                if let TokenValue::String(s) = &token.value {
                    assert_eq!(s, "Hello ");
                } else {
                    panic!("Expected string value in first part");
                }
            } else {
                panic!("Expected literal node in first part");
            }
            // Second part should be a variable
            matches!(parts[1], ASTNode::Variable { .. });
            // Third part should be a string literal
            if let ASTNode::Literal { token } = &parts[2] {
                if let TokenValue::String(s) = &token.value {
                    assert_eq!(s, "!");
                } else {
                    panic!("Expected string value in third part");
                }
            } else {
                panic!("Expected literal node in third part");
            }
        } else {
            panic!("Expected string interpolation node");
        }
    }

    #[test]
    fn test_string_interpolation_multiple_expressions() {
        let result = parse_expression(r#"$"Result: ${x + y} (${type})""#).unwrap();
        if let ASTNode::StringInterpolation { parts } = result {
            assert_eq!(parts.len(), 5); // "Result: ", x+y, " (", type, ")"
            // Check that we have the correct structure
            matches!(parts[0], ASTNode::Literal { .. });
            matches!(parts[1], ASTNode::Binary { .. }); // x + y
            matches!(parts[2], ASTNode::Literal { .. });
            matches!(parts[3], ASTNode::Variable { .. }); // type
            matches!(parts[4], ASTNode::Literal { .. });
        } else {
            panic!("Expected string interpolation node");
        }
    }
}
