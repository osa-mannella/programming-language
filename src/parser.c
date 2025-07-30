#include "parser.h"
#include "lexer.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static ASTNode *parse_literal(Parser *parser, Token token);
static ASTNode *parse_grouping(Parser *parser, Token token);
static ASTNode *parse_binary(Parser *parser, ASTNode *left, Token token);

#define MAX_TOKEN_TYPE 64

static ParseRule parse_rules[MAX_TOKEN_TYPE];

ParseRule *get_rule(TokenType type) { return &parse_rules[type]; }

void parser_free_ast(ASTNode *node) {
  if (!node)
    return;
  switch (node->type) {
  case AST_BINARY:
    parser_free_ast(node->binary.left);
    parser_free_ast(node->binary.right);
    break;
  case AST_GROUPING:
    parser_free_ast(node->grouping.expression);
    break;
  default:
    break;
  }
  free(node);
}

static void parser_advance(Parser *parser) {
  parser->previous = parser->current;
  parser->current = lexer_next(parser->lexer);
}

ASTNode *parse_expression(Parser *parser, int precedence) {
  parser_advance(parser);
  ParseRule *prefix_rule = get_rule(parser->previous.type);
  if (!prefix_rule->nud) {
    printf("Parse error: Expected expression.\n");
    parser->had_error = 1;
    return NULL;
  }

  ASTNode *left = prefix_rule->nud(parser, parser->previous);

  while (precedence < get_rule(parser->current.type)->lbp) {
    parser_advance(parser);
    ParseRule *infix_rule = get_rule(parser->previous.type);
    if (!infix_rule->led)
      break;
    left = infix_rule->led(parser, left, parser->previous);
  }
  return left;
}

static ASTNode *parse_literal(Parser *parser, Token token) {
  ASTNode *node = malloc(sizeof(ASTNode));
  node->type = AST_LITERAL;
  node->literal.token = token;
  return node;
}

static ASTNode *parse_grouping(Parser *parser, Token token) {
  ASTNode *expr = parse_expression(parser, 0);
  if (parser->current.type != TOKEN_RPAREN) {
    printf("Parse error: Expected ')'.\n");
    parser->had_error = 1;
    parser_free_ast(expr);
    return NULL;
  }
  parser_advance(parser); // consume ')'
  ASTNode *node = malloc(sizeof(ASTNode));
  node->type = AST_GROUPING;
  node->grouping.expression = expr;
  return node;
}

static ASTNode *parse_binary(Parser *parser, ASTNode *left, Token token) {
  int precedence = get_rule(token.type)->lbp;
  ASTNode *right = parse_expression(parser, precedence);
  ASTNode *node = malloc(sizeof(ASTNode));
  node->type = AST_BINARY;
  node->binary.left = left;
  node->binary.op = token;
  node->binary.right = right;
  return node;
}

static ASTNode *nud_null(Parser *parser, Token token) {
  printf("Parse error: Unexpected token '%.*s'\n", token.length, token.start);
  parser->had_error = 1;
  return NULL;
}
static ASTNode *led_null(Parser *parser, ASTNode *left, Token token) {
  printf("Parse error: Unexpected infix operator '%.*s'\n", token.length,
         token.start);
  parser->had_error = 1;
  return NULL;
}

static void init_parse_rules() {
  for (int i = 0; i < MAX_TOKEN_TYPE; i++) {
    parse_rules[i].nud = nud_null;
    parse_rules[i].led = led_null;
    parse_rules[i].lbp = 0;
  }

  // Parentheses for grouping
  parse_rules[TOKEN_LPAREN].nud = parse_grouping;
  parse_rules[TOKEN_NUMBER].nud = parse_literal;

  // Binary operators
  parse_rules[TOKEN_PLUS].led = parse_binary;
  parse_rules[TOKEN_PLUS].lbp = 10;
  parse_rules[TOKEN_MINUS].led = parse_binary;
  parse_rules[TOKEN_MINUS].lbp = 10;
  parse_rules[TOKEN_STAR].led = parse_binary;
  parse_rules[TOKEN_STAR].lbp = 20;
  parse_rules[TOKEN_SLASH].led = parse_binary;
  parse_rules[TOKEN_SLASH].lbp = 20;
}

void parser_init(Parser *parser, Lexer *lexer) {
  parser->lexer = lexer;
  parser->had_error = 0;
  parser->panic_mode = 0;
  parser->current = lexer_next(lexer);
  parser->previous = parser->current; // doesn't matter at start
  init_parse_rules();
}

ASTNode *parse(Parser *parser) { return parse_expression(parser, 0); }

void parser_print_ast(ASTNode *node) {
  if (!node) {
    printf("NULL");
    return;
  }
  switch (node->type) {
  case AST_LITERAL:
    printf("%.*s", node->literal.token.length, node->literal.token.start);
    break;
  case AST_BINARY:
    printf("(");
    parser_print_ast(node->binary.left);
    printf(" %.*s ", node->binary.op.length, node->binary.op.start);
    parser_print_ast(node->binary.right);
    printf(")");
    break;
  case AST_GROUPING:
    printf("(");
    parser_print_ast(node->grouping.expression);
    printf(")");
    break;
  default:
    printf("<?>");
    break;
  }
}
