#include <stdio.h>
#include <stdlib.h>

#include "lexer.h"
#include "utils.h"

int main(int argc, char *argv[]) {
  printf("%s\n", argv[1]);
  if (argc < 2) {
    fprintf(stderr, "Usage: %s <file>\n", argv[0]);
    return 1;
  }
  char *source = read_file(argv[1]);
  lexer_init(source);
  free(source);
  return 0;
}