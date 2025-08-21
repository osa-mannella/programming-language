use crate::types::{ast::*, token::Token};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

const MIN_PREC_DEFAULT: u8 = 1;

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
        match self.current() {
            Token::Let | Token::LetBang => self.let_statement(),
            Token::Func => self.func_statement(),
            _ => {
                if matches!(self.current(), Token::Eof | Token::Match | Token::Enum) {
                    return Stmt::Expr(Expr::Identifier("unimplemented".to_string()));
                }
                Stmt::Expr(self.expression(MIN_PREC_DEFAULT))
            }
        }
    }

    fn let_statement(&mut self) -> Stmt {
        self.advance();
        let name = match self.advance() {
            Token::Identifier(n) => n,
            _ => panic!("Expected identifier"),
        };
        self.expect(Token::Assign);
        let value = self.expression(MIN_PREC_DEFAULT);
        Stmt::Let { name, value }
    }

    fn func_statement(&mut self) -> Stmt {
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
        Stmt::Func { name, params, body }
    }

    fn expression(&mut self, min_prec: u8) -> Expr {
        let mut left = self.nud();
        while self.precedence() >= min_prec {
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
                let expr = self.expression(MIN_PREC_DEFAULT);
                self.expect(Token::RightParen);
                expr
            }
            _ => panic!("Unexpected token in nud"),
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
                let right = self.expression(self.precedence() + 1);
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
                    args.push(self.expression(MIN_PREC_DEFAULT));
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
                let right = self.expression(self.precedence() + 1);
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

    fn precedence(&self) -> u8 {
        match self.current() {
            Token::Pipeline => 1,
            Token::Equal
            | Token::NotEqual
            | Token::Less
            | Token::Greater
            | Token::LessEqual
            | Token::GreaterEqual => 2,
            Token::Plus | Token::Minus => 3,
            Token::Multiply | Token::Divide => 4,
            Token::LeftParen => 5,
            _ => 0,
        }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
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
}
