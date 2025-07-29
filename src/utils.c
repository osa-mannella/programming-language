#include "utils.h"
#include <stdio.h>
#include <stdlib.h>

char *read_file(const char *path) {
  FILE *file = fopen(path, "rb");
  if (!file) {
    fprintf(stderr, "Could not open file \"%s\".\n", path);
    exit(1);
  }

  fseek(file, 0L, SEEK_END);
  size_t file_size = ftell(file);
  rewind(file);

  char *buffer = (char *)malloc(file_size + 1);
  if (!buffer) {
    fprintf(stderr, "Out of memory reading \"%s\".\n", path);
    fclose(file);
    exit(1);
  }

  size_t bytes_read = fread(buffer, 1, file_size, file);
  if (bytes_read < file_size) {
    fprintf(stderr, "Could not read file \"%s\".\n", path);
    fclose(file);
    free(buffer);
    exit(1);
  }

  buffer[bytes_read] = '\0';
  fclose(file);
  return buffer;
}
