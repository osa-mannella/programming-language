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

static void free_node(ASTNode *node)
{
  if (!node)
    return;

  switch (node->type)
  {
  case AST_BINARY:
    free_node(node->binary.left);
    free_node(node->binary.right);
    break;

  case AST_GROUPING:
    free_node(node->grouping.expression);
    break;

  case AST_LET_STATEMENT:
    free_node(node->let_statement.initializer);
    break;

  case AST_EXPRESSION_STATEMENT:
    free_node(node->expression_statement.expression);
    break;

  case AST_FUNCTION_STATEMENT:
    for (int i = 0; i < node->function_statement.body_count; i++)
    {
      free_node(node->function_statement.body[i]);
    }
    free(node->function_statement.body);
    free(node->function_statement.params);
    break;

  case AST_LAMBDA_EXPRESSION:
    for (int i = 0; i < node->lambda.body_count; i++)
    {
      free_node(node->lambda.body[i]);
    }
    free(node->lambda.body);
    free(node->lambda.params);
    break;

  case AST_VARIABLE:
  case AST_LITERAL:
    // no additional heap memory
    break;

  default:
    break;
  }

  free(node);
}

void parser_free_ast(ASTProgram *program)
{
  if (!program || !program->nodes)
    return;

  for (int i = 0; i < program->count; i++)
  {
    free_node(program->nodes[i]);
  }
  free(program->nodes);
  program->nodes = NULL;
  program->count = 0;
  program->capacity = 0;
}

static void parser_advance(Parser *parser)
{
  parser->previous = parser->current;
  parser->current = lexer_next(parser->lexer);
}

static ASTNode *parse_expression(Parser *parser, int precedence)
{
  parser_advance(parser);
  ParseRule *prefix_rule = get_rule(parser->previous.type);
  if (!prefix_rule->nud)
  {
    printf("Parse error: Expected expression.\n");
    parser->had_error = 1;
    return NULL;
  }

  ASTNode *left = prefix_rule->nud(parser, parser->previous);

  while (precedence < get_rule(parser->current.type)->lbp &&
         parser->current.type != TOKEN_EOF)
  {
    parser_advance(parser);
    ParseRule *infix_rule = get_rule(parser->previous.type);
    if (!infix_rule->led)
      break;
    left = infix_rule->led(parser, left, parser->previous);
  }
  return left;
}

static ASTNode *parse_literal(Parser *parser, Token token)
{
  ASTNode *node = malloc(sizeof(ASTNode));
  node->type = AST_LITERAL;
  node->literal.token = token;
  return node;
}

static ASTNode *parse_statement(Parser *parser)
{
  if (parser->current.type == TOKEN_LET)
  {
    return parse_let_statement(parser);
  }
  if (parser->current.type == TOKEN_FUNC)
  {
    return parse_function_statement(parser);
  }
  return parse_expression_statement(parser);
}

static ASTNode *parse_expression_statement(Parser *parser)
{
  ASTNode *expr = parse_expression(parser, 0);
  ASTNode *node = malloc(sizeof(ASTNode));
  node->type = AST_EXPRESSION_STATEMENT;
  node->expression_statement.expression = expr;
  return node;
}

static ASTNode *parse_let_statement(Parser *parser)
{
  parser_advance(parser); // consume 'let'
  Token name = parser->current;
  if (parser->current.type != TOKEN_IDENTIFIER)
  {
    printf("Parse error: Expected variable name after 'let'.\n");
    parser->had_error = 1;
    return NULL;
  }
  parser_advance(parser); // consume identifier

  if (parser->current.type != TOKEN_EQUAL)
  {
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

static ASTNode *parse_grouping(Parser *parser, Token token)
{
  ASTNode *expr = parse_expression(parser, 0);
  if (parser->current.type != TOKEN_RPAREN)
  {
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

static ASTNode *parse_variable(Parser *parser, Token token)
{
  ASTNode *node = malloc(sizeof(ASTNode));
  node->type = AST_VARIABLE;
  node->variable.name = token;
  return node;
}

static ASTNode *parse_binary(Parser *parser, ASTNode *left, Token token)
{
  int precedence = get_rule(token.type)->lbp;
  ASTNode *right = parse_expression(parser, precedence);
  ASTNode *node = malloc(sizeof(ASTNode));
  node->type = AST_BINARY;
  node->binary.left = left;
  node->binary.op = token;
  node->binary.right = right;
  return node;
}

static ASTNode *nud_null(Parser *parser, Token token)
{
  if (token.type == TOKEN_EOF)
  {
    return NULL;
  }
  else
  {
    printf("Parse error: Unexpected token '%.*s'\n", token.length, token.start);
    parser->had_error = 1;
    return NULL;
  }
}
static ASTNode *led_null(Parser *parser, ASTNode *left, Token token)
{
  printf("Parse error: Unexpected infix operator '%.*s'\n", token.length,
         token.start);
  parser->had_error = 1;
  return NULL;
}

static void parse_parameter_list(Parser *parser, Token **params, int *param_count)
{
  *params = NULL;
  *param_count = 0;
  int capacity = 0;

  while (parser->current.type != TOKEN_RPAREN)
  {
    if (parser->current.type != TOKEN_IDENTIFIER)
    {
      printf("Parse error: Expected parameter name.\n");
      parser->had_error = 1;
      free(*params);
      *params = NULL;
      *param_count = 0;
      return;
    }
    if (*param_count >= capacity)
    {
      capacity = capacity == 0 ? 4 : capacity * 2;
      *params = realloc(*params, sizeof(Token) * capacity);
    }
    (*params)[(*param_count)++] = parser->current;
    parser_advance(parser);

    if (parser->current.type == TOKEN_COMMA)
    {
      parser_advance(parser);
    }
    else if (parser->current.type != TOKEN_RPAREN)
    {
      printf("Parse error: Expected ',' or ')'.\n");
      parser->had_error = 1;
      free(*params);
      *params = NULL;
      *param_count = 0;
      return;
    }
  }
  parser_advance(parser); // consume ')'
}

static void parse_block(Parser *parser, ASTNode ***body_nodes, int *body_count)
{
  *body_nodes = NULL;
  *body_count = 0;
  int capacity = 0;

  while (parser->current.type != TOKEN_RBRACE && parser->current.type != TOKEN_EOF)
  {
    ASTNode *stmt = parse_statement(parser);
    if (!stmt)
    {
      free(*body_nodes);
      *body_nodes = NULL;
      *body_count = 0;
      return;
    }
    if (*body_count >= capacity)
    {
      capacity = capacity == 0 ? 4 : capacity * 2;
      *body_nodes = realloc(*body_nodes, sizeof(ASTNode *) * capacity);
    }
    (*body_nodes)[(*body_count)++] = stmt;
  }

  if (parser->current.type != TOKEN_RBRACE)
  {
    printf("Parse error: Expected '}' at end of block.\n");
    parser->had_error = 1;
    free(*body_nodes);
    *body_nodes = NULL;
    *body_count = 0;
    return;
  }
  parser_advance(parser); // consume '}'
}

static ASTNode *parse_lambda_expression(Parser *parser, Token token)
{
  if (parser->current.type != TOKEN_LPAREN)
  {
    printf("Parse error: Expected '(' after 'fn'.\n");
    parser->had_error = 1;
    return NULL;
  }
  parser_advance(parser);

  Token *params;
  int param_count;
  parse_parameter_list(parser, &params, &param_count);
  if (parser->had_error)
    return NULL;

  if (parser->current.type != TOKEN_ARROW)
  {
    printf("Parse error: Expected '->' after lambda parameters.\n");
    parser->had_error = 1;
    free(params);
    return NULL;
  }
  parser_advance(parser);

  if (parser->current.type != TOKEN_LBRACE)
  {
    printf("Parse error: Expected '{' after '->' in lambda.\n");
    parser->had_error = 1;
    free(params);
    return NULL;
  }
  parser_advance(parser);

  ASTNode **body_nodes;
  int body_count;
  parse_block(parser, &body_nodes, &body_count);
  if (parser->had_error)
  {
    free(params);
    return NULL;
  }

  ASTNode *node = malloc(sizeof(ASTNode));
  node->type = AST_LAMBDA_EXPRESSION;
  node->lambda.params = params;
  node->lambda.param_count = param_count;
  node->lambda.body = body_nodes;
  node->lambda.body_count = body_count;
  return node;
}

static ASTNode *parse_function_statement(Parser *parser)
{
  parser_advance(parser); // consume 'func'

  if (parser->current.type != TOKEN_IDENTIFIER)
  {
    printf("Parse error: Expected function name after 'func'.\n");
    parser->had_error = 1;
    return NULL;
  }
  Token name = parser->current;
  parser_advance(parser);

  if (parser->current.type != TOKEN_LPAREN)
  {
    printf("Parse error: Expected '(' after function name.\n");
    parser->had_error = 1;
    return NULL;
  }
  parser_advance(parser);

  Token *params;
  int param_count;
  parse_parameter_list(parser, &params, &param_count);
  if (parser->had_error)
    return NULL;

  if (parser->current.type != TOKEN_LBRACE)
  {
    printf("Parse error: Expected '{' for function body.\n");
    parser->had_error = 1;
    free(params);
    return NULL;
  }
  parser_advance(parser);

  ASTNode **body_nodes;
  int body_count;
  parse_block(parser, &body_nodes, &body_count);
  if (parser->had_error)
  {
    free(params);
    return NULL;
  }

  ASTNode *node = malloc(sizeof(ASTNode));
  node->type = AST_FUNCTION_STATEMENT;
  node->function_statement.name = name;
  node->function_statement.params = params;
  node->function_statement.param_count = param_count;
  node->function_statement.body = body_nodes;
  node->function_statement.body_count = body_count;
  return node;
}

static void init_parse_rules()
{
  for (int i = 0; i < MAX_TOKEN_TYPE; i++)
  {
    parse_rules[i].nud = nud_null;
    parse_rules[i].led = led_null;
    parse_rules[i].lbp = 0;
  }

  // Parentheses for grouping
  parse_rules[TOKEN_LPAREN].nud = parse_grouping;
  parse_rules[TOKEN_NUMBER].nud = parse_literal;
  parse_rules[TOKEN_IDENTIFIER].nud = parse_variable;
  parse_rules[TOKEN_FN].nud = parse_lambda_expression;

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

void parser_init(Parser *parser, Lexer *lexer)
{
  parser->lexer = lexer;
  parser->had_error = 0;
  parser->panic_mode = 0;
  parser->current = lexer_next(lexer);
  parser->previous = parser->current;
  init_parse_rules();
}

ASTProgram parse(Parser *parser)
{
  ASTProgram program;
  program.nodes = malloc(sizeof(ASTNode *) * INITIAL_CAPACITY);
  program.count = 0;
  program.capacity = INITIAL_CAPACITY;

  while (parser->current.type != TOKEN_EOF && !parser->had_error)
  {
    ASTNode *node = parse_statement(parser);
    if (!node)
      break;

    if (program.count >= program.capacity)
    {
      program.capacity *= 2;
      program.nodes =
          realloc(program.nodes, sizeof(ASTNode *) * program.capacity);
    }
    program.nodes[program.count++] = node;
  }

  return program;
}

static void print_token(const Token *token)
{
  if (token)
  {
    printf("%.*s", token->length, token->start);
  }
}

void parser_print_ast(ASTProgram *program)
{
  for (int i = 0; i < program->count; i++)
  {
    parser_print_ast_node(program->nodes[i]);
    printf("\n");
  }
}

static void parser_print_ast_node(ASTNode *node)
{
  if (!node)
  {
    printf("NULL");
    return;
  }
  switch (node->type)
  {
  case AST_LITERAL:
    print_token(&node->literal.token);
    break;
  case AST_UNARY:
    print_token(&node->unary.op);
    printf("(");
    parser_print_ast_node(node->unary.right);
    printf(")");
    break;
  case AST_BINARY:
    printf("(");
    parser_print_ast_node(node->binary.left);
    printf(" ");
    print_token(&node->binary.op);
    printf(" ");
    parser_print_ast_node(node->binary.right);
    printf(")");
    break;
  case AST_VARIABLE:
    print_token(&node->variable.name);
    break;
  case AST_GROUPING:
    printf("(");
    parser_print_ast_node(node->grouping.expression);
    printf(")");
    break;
  case AST_ASSIGNMENT:
    print_token(&node->assignment.name);
    printf(" = ");
    parser_print_ast_node(node->assignment.value);
    break;
  case AST_CALL:
    parser_print_ast_node(node->call.callee);
    printf("(");
    for (int i = 0; i < node->call.arg_count; i++)
    {
      parser_print_ast_node(node->call.arguments[i]);
      if (i < node->call.arg_count - 1)
        printf(", ");
    }
    printf(")");
    break;
  case AST_ERROR:
    printf("<error>");
    break;
  case AST_LET_STATEMENT:
    printf("let ");
    print_token(&node->let_statement.name);
    printf(" = ");
    parser_print_ast_node(node->let_statement.initializer);
    break;
  case AST_EXPRESSION_STATEMENT:
    parser_print_ast_node(node->expression_statement.expression);
    break;
  case AST_FUNCTION_STATEMENT:
    printf("func ");
    print_token(&node->function_statement.name);
    printf("(");
    for (int i = 0; i < node->function_statement.param_count; i++)
    {
      print_token(&node->function_statement.params[i]);
      if (i < node->function_statement.param_count - 1)
        printf(", ");
    }
    printf(") ");
    printf("{ ");
    for (int i = 0; i < node->function_statement.body_count; i++)
    {
      parser_print_ast_node(node->function_statement.body[i]);
      if (i < node->function_statement.body_count - 1)
        printf("; ");
    }
    printf(" }");
    break;
  case AST_LAMBDA_EXPRESSION:
    printf("fn(");
    for (int i = 0; i < node->lambda.param_count; i++)
    {
      print_token(&node->lambda.params[i]);
      if (i < node->lambda.param_count - 1)
        printf(", ");
    }
    printf(") -> { ");
    for (int i = 0; i < node->lambda.body_count; i++)
    {
      parser_print_ast_node(node->lambda.body[i]);
      if (i < node->lambda.body_count - 1)
        printf("; ");
    }
    printf(" }");
    break;
  }
}
