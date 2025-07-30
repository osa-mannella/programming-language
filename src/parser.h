#ifndef PARSER_H
#define PARSER_H

#include "lexer.h"

typedef enum {
    AST_LITERAL,
    AST_BINARY,
    AST_UNARY,
    AST_VARIABLE,
    AST_GROUPING,
    AST_ASSIGNMENT,
    AST_CALL,
    AST_ERROR
} ASTNodeType;

typedef struct ASTNode ASTNode;

struct ASTNode {
    ASTNodeType type;

    union {
        // Literal number, string, boolean
        struct {
            Token token;
        } literal;

        // Unary operators: -expr, !expr
        struct {
            Token op;
            ASTNode* right;
        } unary;

        // Binary operators: expr + expr
        struct {
            ASTNode* left;
            Token op;
            ASTNode* right;
        } binary;

        // Variable reference
        struct {
            Token name;
        } variable;

        // Grouping: (expression)
        struct {
            ASTNode* expression;
        } grouping;

        // Assignment: name = expression
        struct {
            Token name;
            ASTNode* value;
        } assignment;

        // Function calls: func(expr, expr...)
        struct {
            ASTNode* callee;
            ASTNode** arguments;
            int arg_count;
        } call;
    };
};

typedef struct {
    Lexer* lexer;
    Token current;
    Token previous;
    int had_error;
    int panic_mode;
} Parser;

// Pratt parser function pointer types
typedef ASTNode* (*NudFn)(Parser*, Token);           // Null Denotation
typedef ASTNode* (*LedFn)(Parser*, ASTNode*, Token); // Left Denotation

typedef struct {
    NudFn nud;
    LedFn led;
    int lbp; // Left Binding Power (precedence)
} ParseRule;

// Parser API
void parser_init(Parser* parser, Lexer* lexer);
ASTNode* parse_expression(Parser* parser, int precedence);
ASTNode* parse(Parser* parser); // entry point, returns root AST
void parser_print_ast(ASTNode* node);
void parser_free_ast(ASTNode* node);

// Lookup table for tokens → parse rules
ParseRule* get_rule(TokenType type);

#endif
