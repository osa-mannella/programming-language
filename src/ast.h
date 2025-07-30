#ifndef AST_H
#define AST_H

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
  AST_LAMBDA_EXPRESSION,
  AST_MATCH_STATEMENT,
} ASTNodeType;

typedef struct ASTNode ASTNode;

typedef struct
{
  ASTNode *pattern;
  ASTNode *expression;
} MatchArm;

struct ASTNode
{
  ASTNodeType type;
  union
  {
    struct
    {
      Token token;
    } literal;
    struct
    {
      Token op;
      ASTNode *right;
    } unary;
    struct
    {
      ASTNode *left;
      Token op;
      ASTNode *right;
    } binary;
    struct
    {
      Token name;
    } variable;
    struct
    {
      ASTNode *expression;
    } grouping;
    struct
    {
      Token name;
      ASTNode *value;
    } assignment;
    struct
    {
      ASTNode *callee;
      ASTNode **arguments;
      int arg_count;
    } call;
    struct
    {
      Token name;
      ASTNode *initializer;
    } let_statement;
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

    struct
    {
      ASTNode *value;
      MatchArm *arms;
      int arm_count;
    } match_statement;
  };
};

typedef struct
{
  ASTNode **nodes;
  int count;
  int capacity;
} ASTProgram;

void free_node(ASTNode *node);
void parser_free_ast(ASTProgram *program);
void parser_print_ast(ASTProgram *program);
void parser_print_ast_node(ASTNode *node);
static void print_token(const Token *token);

#endif
