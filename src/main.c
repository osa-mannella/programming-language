#include <stdio.h>
#include <stdlib.h>

#include "lexer.h"
#include "parser.h"
#include "utils.h"

int main(int argc, char *argv[]) {
  if (argc < 2) {
    fprintf(stderr, "Usage: %s <file>\n", argv[0]);
    return 1;
  }
  char *source = read_file(argv[1]);
  Lexer lexer;
  lexer_init(source, &lexer);
  Parser parser;
  parser_init(&parser, &lexer);

  ASTNode *ast = parse(&parser);
  parser_print_ast(ast);
  parser_free_ast(ast);
  free(source);
  return 0;
}