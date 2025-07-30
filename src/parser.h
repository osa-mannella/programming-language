#ifndef PARSER_H
#define PARSER_H

#include "lexer.h"
#include "ast.h"

typedef struct
{
  Lexer *lexer;
  Token current;
  Token previous;
  int had_error;
  int panic_mode;
} Parser;

// Pratt parser function pointer types
typedef ASTNode *(*NudFn)(Parser *, Token);            // Null Denotation
typedef ASTNode *(*LedFn)(Parser *, ASTNode *, Token); // Left Denotation

typedef struct
{
  NudFn nud;
  LedFn led;
  int lbp; // Left Binding Power (precedence)
} ParseRule;

// Parser API
void parser_init(Parser *parser, Lexer *lexer);
static ASTNode *parse_expression(Parser *parser, int precedence);
ASTProgram parse(Parser *parser); // entry point, returns root AST
static ASTNode *parse_statement(Parser *parser);
static ASTNode *parse_let_statement(Parser *parser);
static ASTNode *parse_expression_statement(Parser *parser);
static ASTNode *parse_function_statement(Parser *parser);
static ASTNode *parse_match_statement(Parser *parser);
static void parse_block(Parser *parser, ASTNode ***body_nodes, int *body_count);
static void parse_parameter_list(Parser *parser, Token **params, int *param_count);

// Lookup table for tokens → parse rules
static ParseRule *get_rule(TokenType type);

#endif
