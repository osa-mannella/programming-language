CC = gcc
CFLAGS = -Wall -Wextra -std=c11 -g

SRCS = src/main.c src/lexer.c
OBJS = $(SRCS:.c=.o)

mirrow: $(OBJS)
    $(CC) $(OBJS) -o mirrow

clean:
    rm -f $(OBJS) mirrow
