#ifndef PARSER_H
#define PARSER_H

#include "lexer.h"

typedef enum
{
  AST_LITERAL,
  AST_BINARY,
  AST_UNARY,
  AST_VARIABLE,
  AST_GROUPING,
  AST_ASSIGNMENT,
  AST_CALL,
  AST_ERROR,
  AST_LET_STATEMENT,
  AST_EXPRESSION_STATEMENT,
  AST_FUNCTION_STATEMENT,
  AST_LAMBDA_EXPRESSION
} ASTNodeType;

typedef struct ASTNode ASTNode;

struct ASTNode
{
  ASTNodeType type;

  union
  {
    // Literal number, string, boolean
    struct
    {
      Token token;
    } literal;

    // Unary operators: -expr, !expr
    struct
    {
      Token op;
      ASTNode *right;
    } unary;

    // Binary operators: expr + expr
    struct
    {
      ASTNode *left;
      Token op;
      ASTNode *right;
    } binary;

    // Variable reference
    struct
    {
      Token name;
    } variable;

    // Grouping: (expression)
    struct
    {
      ASTNode *expression;
    } grouping;

    // Assignment: name = expression
    struct
    {
      Token name;
      ASTNode *value;
    } assignment;

    // Function calls: func(expr, expr...)
    struct
    {
      ASTNode *callee;
      ASTNode **arguments;
      int arg_count;
    } call;

    // Let statement: let name = expression
    struct
    {
      Token name;
      ASTNode *initializer;
    } let_statement;

    // Expression statement: expression
    struct
    {
      ASTNode *expression;
    } expression_statement;

    struct
    {
      Token name;
      Token *params;
      int param_count;
      ASTNode **body;
      int body_count;
    } function_statement;

    struct
    {
      Token *params;
      int param_count;
      ASTNode **body;
      int body_count;
    } lambda;
  };
};

typedef struct
{
  ASTNode **nodes;
  int count;
  int capacity;
} ASTProgram;

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
void parser_print_ast(ASTProgram *node);
static ASTNode *parse_statement(Parser *parser);
static ASTNode *parse_let_statement(Parser *parser);
void parser_free_ast(ASTProgram *node);
static ASTNode *parse_expression_statement(Parser *parser);
static ASTNode *parse_function_statement(Parser *parser);
static void parse_block(Parser *parser, ASTNode ***body_nodes, int *body_count);
static void parse_parameter_list(Parser *parser, Token **params, int *param_count);
static void parser_print_ast_node(ASTNode *node);
static void free_node(ASTNode *node);
static void print_token(const Token *token);

// Lookup table for tokens → parse rules
static ParseRule *get_rule(TokenType type);

#endif
