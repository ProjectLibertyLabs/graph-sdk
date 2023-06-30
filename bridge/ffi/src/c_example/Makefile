# Determine the operating system
UNAME := $(shell uname)

# Compiler options
CC := gcc
CFLAGS :=

PROFILE := release

# Linker options
LDFLAGS := -L../../../../target/$(PROFILE:dev=debug)

# Libraries to link in
LIBS := -ldsnp_graph_sdk_ffi -lsodium -lm

# Additional options for MacOS
ifeq ($(UNAME), Darwin)
    # MacOS compile options
    CFLAGS += -I /opt/homebrew/include -DMACOS_FLAG
    # MacOS linker options
    LDFLAGS += -L/opt/homebrew/lib
endif

# Additional options for Linux
ifeq ($(UNAME), Linux)
    # Linux compile options
    CFLAGS += -DLINUX_FLAG
    # Linux linker options
    LDFLAGS +=
endif

# Target executable
TARGET := main

# Source files
SRCS := main.c

# Object files
OBJS := $(SRCS:.c=.o)

.PHONY: all clean

all: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(OBJS) $(LDFLAGS) $(LIBS) -o $(TARGET)

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(OBJS) $(TARGET)
