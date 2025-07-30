#include "lexer.h"
#include "debug.h"
#include <ctype.h>
#include <stdio.h>
#include <string.h>

static int is_at_end(Lexer *lexer);

void lexer_init(const char *source, Lexer *lexer) {
  lexer->start = source;
  lexer->current = source;
  lexer->line = 1;
}

void lexer_debug(Lexer *lexer) {
  Token t = lexer_next(lexer);
  while (t.type != TOKEN_EOF) {
    printf("Token: %s\n", token_type_name(t.type));
    t = lexer_next(lexer);
  }
}

static char advance(Lexer *lexer) {
  if (is_at_end(lexer))
    return '\0';
  return *lexer->current++;
}

static char peek(Lexer *lexer) {
  if (is_at_end(lexer))
    return '\0';
  return *lexer->current;
}

static char peek_next(Lexer *lexer) {
  if (is_at_end(lexer))
    return '\0';
  return lexer->current[1];
}

static void skip_whitespace(Lexer *lexer) {
  for (;;) {
    char c = peek(lexer);

    if (is_at_end(lexer)) {
      break;
    }

    if (c == ' ' || c == '\r' || c == '\t') {
      advance(lexer);
    } else if (c == '\n') {
      lexer->line++;
      advance(lexer);
    } else if (c < 32 && c != '\n' && c != '\r' && c != '\t') {
      advance(lexer);
    } else {
      break;
    }
  }
}

static int is_at_end(Lexer *lexer) { return *lexer->current == '\0'; }

static Token make_token(TokenType type, Lexer *lexer) {
  Token t;
  t.type = type;
  t.start = lexer->start;
  t.length = (int)(lexer->current - lexer->start);
  t.line = lexer->line;
  return t;
}

static Token error_token(const char *msg, Lexer *lexer) {
  Token t;
  t.type = TOKEN_ERROR;
  t.start = msg;
  t.length = (int)strlen(msg);
  t.line = lexer->line;
  return t;
}

// Match and advance if the next character is expected
static int match(char expected, Lexer *lexer) {
  if (is_at_end(lexer))
    return 0;
  if (*lexer->current != expected)
    return 0;
  lexer->current++;
  return 1;
}

static Token identifier(Lexer *lexer) {
  while (isalnum(peek(lexer)) || peek(lexer) == '_')
    advance(lexer);

  int length = (int)(lexer->current - lexer->start);

  // Keyword checks (only those supported)
  if (length == 3 && strncmp(lexer->start, "let", 3) == 0)
    return make_token(TOKEN_LET, lexer);
  if (length == 4 && strncmp(lexer->start, "func", 4) == 0)
    return make_token(TOKEN_FUNC, lexer);
  if (length == 2 && strncmp(lexer->start, "if", 2) == 0)
    return make_token(TOKEN_IF, lexer);
  if (length == 4 && strncmp(lexer->start, "else", 4) == 0)
    return make_token(TOKEN_ELSE, lexer);
  if (length == 4 && strncmp(lexer->start, "true", 4) == 0)
    return make_token(TOKEN_TRUE, lexer);
  if (length == 5 && strncmp(lexer->start, "false", 5) == 0)
    return make_token(TOKEN_FALSE, lexer);
  if (length == 5 && strncmp(lexer->start, "match", 5) == 0)
    return make_token(TOKEN_MATCH, lexer);
  if (length == 2 && strncmp(lexer->start, "fn", 2) == 0)
    return make_token(TOKEN_FN, lexer);
  if (length == 5 && strncmp(lexer->start, "async", 5) == 0)
    return make_token(TOKEN_ASYNC, lexer);
  if (length == 5 && strncmp(lexer->start, "await", 5) == 0)
    return make_token(TOKEN_AWAIT, lexer);
  if (length == 5 && strncmp(lexer->start, "throw", 5) == 0)
    return make_token(TOKEN_THROW, lexer);
  if (length == 3 && strncmp(lexer->start, "try", 3) == 0)
    return make_token(TOKEN_TRY, lexer);
  if (length == 5 && strncmp(lexer->start, "catch", 5) == 0)
    return make_token(TOKEN_CATCH, lexer);
  if (length == 6 && strncmp(lexer->start, "import", 6) == 0)
    return make_token(TOKEN_IMPORT, lexer);

  return make_token(TOKEN_IDENTIFIER, lexer);
}

static Token number(Lexer *lexer) {
  while (isdigit(peek(lexer)))
    advance(lexer);

  // Fractional part
  if (peek(lexer) == '.' && isdigit(peek_next(lexer))) {
    advance(lexer); // consume '.'
    while (isdigit(peek(lexer)))
      advance(lexer);
  }

  return make_token(TOKEN_NUMBER, lexer);
}

static Token string(Lexer *lexer) {
  while (peek(lexer) != '"' && !is_at_end(lexer)) {
    if (peek(lexer) == '\n')
      lexer->line++;
    advance(lexer);
  }

  if (is_at_end(lexer))
    return error_token("Unterminated string.", lexer);

  advance(lexer); // closing quote
  return make_token(TOKEN_STRING, lexer);
}

Token lexer_next(Lexer *lexer) {
  skip_whitespace(lexer);

  lexer->start = lexer->current;
  if (is_at_end(lexer))
    return make_token(TOKEN_EOF, lexer);

  char c = advance(lexer);

  // Identifiers and keywords
  if (isalpha(c) || c == '_') {
    return identifier(lexer);
  }

  // Numbers
  if (isdigit(c)) {
    return number(lexer);
  }

  // Strings
  if (c == '"') {
    return string(lexer);
  }

  // Operators & single char tokens
  switch (c) {
  // Brackets
  case '(':
    return make_token(TOKEN_LPAREN, lexer);
  case ')':
    return make_token(TOKEN_RPAREN, lexer);
  case '{':
    return make_token(TOKEN_LBRACE, lexer);
  case '}':
    return make_token(TOKEN_RBRACE, lexer);

  // Operators, two-char first
  case '=':
    if (match('=', lexer))
      return make_token(TOKEN_EQUAL_EQUAL, lexer);
    return make_token(TOKEN_EQUAL, lexer);
  case '!':
    if (match('=', lexer))
      return make_token(TOKEN_BANG_EQUAL, lexer);
    return make_token(TOKEN_BANG, lexer);
  case '>':
    if (match('=', lexer))
      return make_token(TOKEN_GREATER_EQUAL, lexer);
    return make_token(TOKEN_GREATER, lexer);
  case '<':
    if (match('=', lexer))
      return make_token(TOKEN_LESS_EQUAL, lexer);
    return make_token(TOKEN_LESS, lexer);

  // Single-char operators and punctuation
  case '+':
    return make_token(TOKEN_PLUS, lexer);
  case '-':
    if (match('>', lexer))
      return make_token(TOKEN_ARROW, lexer);
    return make_token(TOKEN_MINUS, lexer);
  case '*':
    return make_token(TOKEN_STAR, lexer);
  case '/':
    if (match('/', lexer)) {
      while (peek(lexer) != '\n' && !is_at_end(lexer))
        advance(lexer);
      return lexer_next(lexer);
    } else if (match('*', lexer)) {
      while (!(peek(lexer) == '*' && peek_next(lexer) == '/') &&
             !is_at_end(lexer)) {
        if (peek(lexer) == '\n')
          lexer->line++;
        advance(lexer);
      }
      if (!is_at_end(lexer)) {
        advance(lexer);
        advance(lexer);
      }
      return lexer_next(lexer);
    }
    return make_token(TOKEN_SLASH, lexer);

  case ',':
    return make_token(TOKEN_COMMA, lexer);
  case ';':
    return make_token(TOKEN_SEMICOLON, lexer);
  case ':':
    return make_token(TOKEN_COLON, lexer);
  case '.':
    return make_token(TOKEN_DOT, lexer);
  case '[':
    return make_token(TOKEN_LBRACKET, lexer);
  case ']':
    return make_token(TOKEN_RBRACKET, lexer);
  case '&':
    if (match('&', lexer))
      return make_token(TOKEN_AND, lexer);
    return error_token("Unexpected '&'.", lexer);
  case '|':
    if (match('|', lexer))
      return make_token(TOKEN_OR, lexer);
    return make_token(TOKEN_PIPE, lexer);

  case '?':
    return make_token(TOKEN_QUESTION, lexer);
  case '#':
    return make_token(TOKEN_REFLECT, lexer);
  case '_':
    return make_token(TOKEN_UNDERSCORE, lexer);
  case '$':
    return make_token(TOKEN_DOLLAR, lexer);
  }

  return error_token("Unexpected character.", lexer);
}