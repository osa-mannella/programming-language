use crate::types::{ast::*, constants::Precedence, token::Token};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Program {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            self.skip_newlines();
            if !self.is_at_end() {
                statements.push(self.statement());
            }
        }
        Program { statements }
    }

    fn statement(&mut self) -> Stmt {
        let line = self.current_line();
        match self.current() {
            Token::Let | Token::LetBang => self.let_statement(line),
            Token::Func => self.func_statement(line),
            _ => Stmt::Expr(self.expression(Precedence::Pipeline.as_u8()), line),
        }
    }

    fn let_statement(&mut self, line: usize) -> Stmt {
        self.advance();
        let name = match self.advance() {
            Token::Identifier(n) => n,
            _ => panic!("Expected identifier"),
        };
        self.expect(Token::Assign);
        let value = self.expression(Precedence::Pipeline.as_u8());
        Stmt::Let { name, value, line }
    }

    fn func_statement(&mut self, line: usize) -> Stmt {
        self.advance();
        let name = match self.advance() {
            Token::Identifier(n) => n,
            _ => panic!("Expected identifier"),
        };
        self.expect(Token::LeftParen);
        let mut params = Vec::new();
        while !matches!(self.current(), Token::RightParen) {
            if let Token::Identifier(p) = self.advance() {
                params.push(p);
            }
            if matches!(self.current(), Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RightParen);
        self.expect(Token::LeftBrace);
        let mut body = Vec::new();
        while !matches!(self.current(), Token::RightBrace) {
            self.skip_newlines();
            if !matches!(self.current(), Token::RightBrace) {
                body.push(self.statement());
            }
        }
        self.expect(Token::RightBrace);
        Stmt::Func {
            name,
            params,
            body,
            line,
        }
    }

    fn expression(&mut self, min_prec: u8) -> Expr {
        let mut left = self.nud();
        while self.precedence(false) >= min_prec {
            left = self.led(left);
        }
        left
    }

    fn nud(&mut self) -> Expr {
        match self.advance() {
            Token::Identifier(s) => Expr::Identifier(s),
            Token::Number(n) => Expr::Number(n),
            Token::String(s) => Expr::String(s),
            Token::LeftParen => {
                let expr = self.expression(Precedence::Pipeline.as_u8());
                self.expect(Token::RightParen);
                expr
            }
            Token::Minus => {
                let right = self.expression(Precedence::Unary.as_u8());
                Expr::Unary {
                    op: UnaryOp::Neg,
                    right: Box::new(right),
                }
            }
            Token::Not => {
                let right = self.expression(Precedence::Unary.as_u8());
                Expr::Unary {
                    op: UnaryOp::Not,
                    right: Box::new(right),
                }
            }
            Token::LeftBracket => {
                let mut elements = Vec::new();

                // Handle empty array
                if matches!(self.current(), Token::RightBracket) {
                    self.advance();
                    return Expr::Array { elements };
                }

                // Parse array elements [expr, expr, ...]
                loop {
                    elements.push(self.expression(Precedence::Pipeline.as_u8()));

                    match self.current() {
                        Token::Comma => {
                            self.advance();
                            // Allow trailing comma [1, 2, 3,]
                            if matches!(self.current(), Token::RightBracket) {
                                break;
                            }
                        }
                        Token::RightBracket => break,
                        _ => panic!("Expected ',' or ']' in array literal"),
                    }
                }

                self.expect(Token::RightBracket);
                Expr::Array { elements }
            }
            Token::True => Expr::Boolean(true),
            Token::False => Expr::Boolean(false),
            t => {
                panic!("Unexpected token in nud: {:?}", t);
            }
        }
    }

    fn led(&mut self, left: Expr) -> Expr {
        match self.current() {
            Token::Plus
            | Token::Minus
            | Token::Multiply
            | Token::Divide
            | Token::Equal
            | Token::NotEqual
            | Token::Less
            | Token::Greater
            | Token::LessEqual
            | Token::GreaterEqual => {
                let op = self.binary_op();
                self.advance();
                let right = self.expression(self.precedence(true) + 1);
                Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                }
            }
            Token::LeftParen => {
                self.advance();
                let mut args = Vec::new();
                while !matches!(self.current(), Token::RightParen) {
                    args.push(self.expression(Precedence::Pipeline.as_u8()));
                    if matches!(self.current(), Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RightParen);
                Expr::Call {
                    func: Box::new(left),
                    args,
                }
            }
            Token::Pipeline => {
                self.advance();
                let right = self.expression(self.precedence(true) + 1);
                Expr::Pipeline {
                    left: Box::new(left),
                    right: Box::new(right),
                }
            }
            _ => left,
        }
    }

    fn binary_op(&self) -> BinaryOp {
        match self.current() {
            Token::Plus => BinaryOp::Add,
            Token::Minus => BinaryOp::Sub,
            Token::Multiply => BinaryOp::Mul,
            Token::Divide => BinaryOp::Div,
            Token::Equal => BinaryOp::Eq,
            Token::NotEqual => BinaryOp::Ne,
            Token::Less => BinaryOp::Lt,
            Token::Greater => BinaryOp::Gt,
            Token::LessEqual => BinaryOp::Le,
            Token::GreaterEqual => BinaryOp::Ge,
            _ => panic!("Not a binary operator"),
        }
    }

    fn precedence(&self, right_parse: bool) -> u8 {
        match self.current() {
            Token::Pipeline => Precedence::Pipeline.as_u8(),
            Token::Equal
            | Token::NotEqual
            | Token::Less
            | Token::Greater
            | Token::LessEqual
            | Token::GreaterEqual => Precedence::Comparison.as_u8(),
            Token::Plus | Token::Minus => Precedence::Term.as_u8(),
            Token::Multiply | Token::Divide => Precedence::Factor.as_u8(),
            Token::LeftParen => Precedence::Unary.as_u8(),
            Token::String(_)
            | Token::Number(_)
            | Token::Identifier(_)
            | Token::True
            | Token::False => {
                if right_parse {
                    return Precedence::Lowest.as_u8();
                } else {
                    panic!("Invalid hanging literal: {:?}", self.current());
                }
            }
            _ => Precedence::Lowest.as_u8(),
        }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1)
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, expected: Token) {
        if std::mem::discriminant(self.current()) != std::mem::discriminant(&expected) {
            panic!("Expected {:?}, found {:?}", expected, self.current());
        }
        self.advance();
    }

    fn skip_newlines(&mut self) {
        while matches!(self.current(), Token::Newline) {
            self.advance();
        }
    }

    fn is_at_end(&mut self) -> bool {
        self.skip_newlines();
        matches!(self.current(), Token::Eof)
    }

    fn current_line(&self) -> usize {
        let mut line = 1;
        for t in self.tokens.iter().take(self.pos) {
            if matches!(t, Token::Newline) {
                line += 1;
            }
        }
        line
    }
}
