#include "lexer.h"
#include "debug.h"
#include <ctype.h>
#include <stdio.h>
#include <string.h>

static Lexer lexer;

void lexer_init(const char *source) {
  lexer.start = source;
  lexer.current = source;
  lexer.line = 1;

  lexer_debug();
}

void lexer_debug() {
  Token t = lexer_next();

  while (t.type != TOKEN_EOF) {
    printf("Token: %s\n", token_type_name(t.type));
    t = lexer_next();
  }
}

static char advance() { return *lexer.current++; }

static char peek() { return *lexer.current; }

static char peek_next() { return lexer.current[1]; }

static void skip_whitespace() {
  for (;;) {
    char c = peek();
    if (c == ' ' || c == '\r' || c == '\t') {
      advance();
    } else if (c == '\n') {
      lexer.line++;
      advance();
    } else {
      break;
    }
  }
}

static int is_at_end() { return *lexer.current == '\0'; }

static Token make_token(TokenType type) {
  Token t;
  t.type = type;
  t.start = lexer.start;
  t.length = (int)(lexer.current - lexer.start);
  t.line = lexer.line;
  return t;
}

static Token error_token(const char *msg) {
  Token t;
  t.type = TOKEN_ERROR;
  t.start = msg;
  t.length = (int)strlen(msg);
  t.line = lexer.line;
  return t;
}

// Match and advance if the next character is expected
static int match(char expected) {
  if (is_at_end())
    return 0;
  if (*lexer.current != expected)
    return 0;
  lexer.current++;
  return 1;
}

static Token identifier() {
  while (isalnum(peek()) || peek() == '_')
    advance();

  int length = (int)(lexer.current - lexer.start);

  // Keyword checks
  if (length == 3 && strncmp(lexer.start, "let", 3) == 0)
    return make_token(TOKEN_LET);
  if (length == 5 && strncmp(lexer.start, "const", 5) == 0)
    return make_token(TOKEN_CONST);
  if (length == 4 && strncmp(lexer.start, "func", 4) == 0)
    return make_token(TOKEN_FUNC);
  if (length == 2 && strncmp(lexer.start, "if", 2) == 0)
    return make_token(TOKEN_IF);
  if (length == 4 && strncmp(lexer.start, "else", 4) == 0)
    return make_token(TOKEN_ELSE);
  if (length == 5 && strncmp(lexer.start, "while", 5) == 0)
    return make_token(TOKEN_WHILE);
  if (length == 6 && strncmp(lexer.start, "return", 6) == 0)
    return make_token(TOKEN_RETURN);
  if (length == 5 && strncmp(lexer.start, "break", 5) == 0)
    return make_token(TOKEN_BREAK);
  if (length == 8 && strncmp(lexer.start, "continue", 8) == 0)
    return make_token(TOKEN_CONTINUE);
  if (length == 3 && strncmp(lexer.start, "for", 3) == 0)
    return make_token(TOKEN_FOR);
  if (length == 4 && strncmp(lexer.start, "true", 4) == 0)
    return make_token(TOKEN_TRUE);
  if (length == 5 && strncmp(lexer.start, "false", 5) == 0)
    return make_token(TOKEN_FALSE);
  if (length == 4 && strncmp(lexer.start, "null", 4) == 0)
    return make_token(TOKEN_NULL);

  return make_token(TOKEN_IDENTIFIER);
}

static Token number() {
  while (isdigit(peek()))
    advance();

  // Fractional part
  if (peek() == '.' && isdigit(peek_next())) {
    advance(); // consume '.'
    while (isdigit(peek()))
      advance();
  }

  return make_token(TOKEN_NUMBER);
}

static Token string() {
  while (peek() != '"' && !is_at_end()) {
    if (peek() == '\n')
      lexer.line++;
    advance();
  }

  if (is_at_end())
    return error_token("Unterminated string.");

  advance(); // closing quote
  return make_token(TOKEN_STRING);
}

Token lexer_next() {
  skip_whitespace();

  lexer.start = lexer.current;
  if (is_at_end())
    return make_token(TOKEN_EOF);

  char c = advance();

  // Identifiers and keywords
  if (isalpha(c) || c == '_') {
    return identifier();
  }

  // Numbers
  if (isdigit(c)) {
    return number();
  }

  // Strings
  if (c == '"') {
    return string();
  }

  // Operators & single char tokens
  switch (c) {
  // Brackets
  case '(':
    return make_token(TOKEN_LPAREN);
  case ')':
    return make_token(TOKEN_RPAREN);
  case '{':
    return make_token(TOKEN_LBRACE);
  case '}':
    return make_token(TOKEN_RBRACE);

  // Operators, two-char first
  case '=':
    if (match('='))
      return make_token(TOKEN_EQUAL_EQUAL);
    return make_token(TOKEN_EQUAL);
  case '!':
    if (match('='))
      return make_token(TOKEN_BANG_EQUAL);
    return make_token(TOKEN_BANG);
  case '>':
    if (match('='))
      return make_token(TOKEN_GREATER_EQUAL);
    return make_token(TOKEN_GREATER);
  case '<':
    if (match('='))
      return make_token(TOKEN_LESS_EQUAL);
    return make_token(TOKEN_LESS);

  // Single-char operators and punctuation
  case '+':
    return make_token(TOKEN_PLUS);
  case '-':
    if (match('>'))
      return make_token(TOKEN_ARROW);
    return make_token(TOKEN_MINUS);
  case '*':
    return make_token(TOKEN_STAR);
  case '/':
    if (match('/')) {
      while (peek() != '\n' && !is_at_end())
        advance();
      return lexer_next();
    } else if (match('*')) {
      while (!(peek() == '*' && peek_next() == '/') && !is_at_end()) {
        if (peek() == '\n')
          lexer.line++;
        advance();
      }
      if (!is_at_end()) {
        advance();
        advance();
      }
      return lexer_next();
    }
    return make_token(TOKEN_SLASH);

  case ',':
    return make_token(TOKEN_COMMA);
  case ';':
    return make_token(TOKEN_SEMICOLON);
  case ':':
    return make_token(TOKEN_COLON);
  case '.':
    return make_token(TOKEN_DOT);
  case '[':
    return make_token(TOKEN_LBRACKET);
  case ']':
    return make_token(TOKEN_RBRACKET);
  case '&':
    if (match('&'))
      return make_token(TOKEN_AND);
    return error_token("Unexpected '&'.");
  case '|':
    if (match('|'))
      return make_token(TOKEN_OR);
    return error_token("Unexpected '|'.");

  case '?':
    return make_token(TOKEN_QUESTION);
  case '#':
    return make_token(TOKEN_REFLECT);
  }

  return error_token("Unexpected character.");
}