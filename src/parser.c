#include "parser.h"
#include "lexer.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static ASTNode *parse_literal(Parser *parser, Token token);
static ASTNode *parse_grouping(Parser *parser, Token token);
static ASTNode *parse_binary(Parser *parser, ASTNode *left, Token token);
static ASTNode *parse_variable(Parser *parser, Token token);

#define MAX_TOKEN_TYPE 64
#define INITIAL_CAPACITY 8

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

static ASTNode *parse_expression(Parser *parser, int precedence) {
  parser_advance(parser);
  ParseRule *prefix_rule = get_rule(parser->previous.type);
  if (!prefix_rule->nud) {
    printf("Parse error: Expected expression.\n");
    parser->had_error = 1;
    return NULL;
  }

  ASTNode *left = prefix_rule->nud(parser, parser->previous);

  while (precedence < get_rule(parser->current.type)->lbp &&
         parser->current.type != TOKEN_EOF) {
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

static ASTNode *parse_statement(Parser *parser) {
  if (parser->current.type == TOKEN_LET) {
    return parse_let_statement(parser);
  }
  // You can add more statement types here (if, match, etc.)
  return parse_expression_statement(parser);
}

static ASTNode *parse_expression_statement(Parser *parser) {
  ASTNode *expr = parse_expression(parser, 0);
  ASTNode *node = malloc(sizeof(ASTNode));
  node->type = AST_EXPRESSION_STATEMENT;
  node->expression_statement.expression = expr;
  return node;
}

static ASTNode *parse_let_statement(Parser *parser) {
  parser_advance(parser); // consume 'let'
  Token name = parser->current;
  if (parser->current.type != TOKEN_IDENTIFIER) {
    printf("Parse error: Expected variable name after 'let'.\n");
    parser->had_error = 1;
    return NULL;
  }
  parser_advance(parser); // consume identifier

  if (parser->current.type != TOKEN_EQUAL) {
    printf("Parse error: Expected '=' after variable name.\n");
    parser->had_error = 1;
    return NULL;
  }
  parser_advance(parser); // consume '='

  ASTNode *initializer = parse_expression(parser, 0);

  ASTNode *node = malloc(sizeof(ASTNode));
  node->type = AST_LET_STATEMENT;
  node->let_statement.name = name;
  node->let_statement.initializer = initializer;
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

static ASTNode *parse_variable(Parser *parser, Token token) {
  ASTNode *node = malloc(sizeof(ASTNode));
  node->type = AST_VARIABLE;
  node->variable.name = token;
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
  if (token.type == TOKEN_EOF) {
    return NULL;
  } else {
    printf("Parse error: Unexpected token '%.*s'\n", token.length, token.start);
    parser->had_error = 1;
    return NULL;
  }
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
  parse_rules[TOKEN_IDENTIFIER].nud = parse_variable;

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
  parser->previous = parser->current;
  init_parse_rules();
}

ASTProgram parse(Parser *parser) {
  ASTProgram program;
  program.nodes = malloc(sizeof(ASTNode *) * INITIAL_CAPACITY);
  program.count = 0;
  program.capacity = INITIAL_CAPACITY;

  while (parser->current.type != TOKEN_EOF && !parser->had_error) {
    ASTNode *node = parse_statement(parser);
    if (!node)
      break;

    if (program.count >= program.capacity) {
      program.capacity *= 2;
      program.nodes =
          realloc(program.nodes, sizeof(ASTNode *) * program.capacity);
    }
    program.nodes[program.count++] = node;
  }

  return program;
}

void parser_print_ast(ASTNode *node) {
  if (!node) {
    printf("NULL\n");
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
  case AST_VARIABLE:
    printf("%.*s", node->variable.name.length, node->variable.name.start);
    break;
  case AST_LET_STATEMENT:
    printf("let %.*s = ", node->let_statement.name.length,
           node->let_statement.name.start);
    parser_print_ast(node->let_statement.initializer);
    break;
  case AST_EXPRESSION_STATEMENT:
    parser_print_ast(node->expression_statement.expression);
    break;
  default:
    printf("<?>");
    break;
  }
}
