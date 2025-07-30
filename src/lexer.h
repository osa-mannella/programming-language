#ifndef LEXER_H
#define LEXER_H

typedef enum {
  TOKEN_IDENTIFIER,
  TOKEN_NUMBER,
  TOKEN_STRING,
  TOKEN_LET,
  TOKEN_FUNC,
  TOKEN_EQUAL,
  TOKEN_LPAREN,
  TOKEN_RPAREN,
  TOKEN_EOF,
  TOKEN_ERROR,
  TOKEN_EQUAL_EQUAL,
  TOKEN_BANG_EQUAL,
  TOKEN_GREATER_EQUAL,
  TOKEN_LESS_EQUAL,
  TOKEN_GREATER,
  TOKEN_LESS,
  TOKEN_PLUS,
  TOKEN_MINUS,
  TOKEN_STAR,
  TOKEN_SLASH,
  TOKEN_COMMA,
  TOKEN_SEMICOLON,
  TOKEN_COLON,
  TOKEN_BANG,
  TOKEN_LBRACE,
  TOKEN_RBRACE,
  TOKEN_LBRACKET,
  TOKEN_RBRACKET,
  TOKEN_DOT,
  TOKEN_AND,
  TOKEN_OR,
  TOKEN_ARROW,
  TOKEN_QUESTION,
  TOKEN_REFLECT,
  TOKEN_IF,
  TOKEN_ELSE,
  TOKEN_TRUE,
  TOKEN_FALSE,
  TOKEN_PIPE,
  TOKEN_UNDERSCORE,
  TOKEN_MATCH,
  TOKEN_FN,
  TOKEN_DOLLAR,
  TOKEN_ASYNC,
  TOKEN_AWAIT,
  TOKEN_THROW,
  TOKEN_TRY,
  TOKEN_CATCH,
  TOKEN_IMPORT,

} TokenType;

typedef struct {
  TokenType type;
  const char *start;
  int length;
  int line;
} Token;

typedef struct {
  const char *start;
  const char *current;
  int line;
} Lexer;

void lexer_init(const char *source, Lexer *lexer);
Token lexer_next(Lexer *lexer);
void lexer_debug(Lexer *lexer);

#endif
