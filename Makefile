# Chim 3.1 Compiler Makefile (Windows)
# 使用 nmake 或 mingw32-make

!IF "$(CC)" == ""
CC = gcc
!ENDIF

!IF "$(CFLAGS)" == ""
CFLAGS = -Wall -Wextra -Werror -std=c11 -O2
!ENDIF

!IF "$(OUTDIR)" == ""
OUTDIR = bin
!ENDIF

# 目录结构
SRC_DIR = src
INCLUDE_DIR = include
OUT_DIR = $(OUTDIR)

# 源文件
CHIM_SOURCES = ^
    $(SRC_DIR)\common.c ^
    $(SRC_DIR)\frontend\lexer.c ^
    $(SRC_DIR)\frontend\parser.c ^
    $(SRC_DIR)\middleend\ir.c ^
    $(SRC_DIR)\middleend\optimizer.c ^
    $(SRC_DIR)\backend\codegen.c ^
    $(SRC_DIR)\backend\c_codegen.c ^
    $(SRC_DIR)\backend\wasm_backend.c ^
    $(SRC_DIR)\toolchain\chim.c ^
    $(SRC_DIR)\toolchain\chimc.c

# 头文件
CHIM_HEADERS = ^
    $(INCLUDE_DIR)\chim\common.h ^
    $(INCLUDE_DIR)\chim\token.h ^
    $(INCLUDE_DIR)\chim\error.h ^
    $(INCLUDE_DIR)\chim\lexer.h ^
    $(INCLUDE_DIR)\chim\ast.h ^
    $(INCLUDE_DIR)\chim\parser.h ^
    $(INCLUDE_DIR)\chim\ir.h ^
    $(INCLUDE_DIR)\chim\optimizer.h ^
    $(INCLUDE_DIR)\chim\codegen.h ^
    $(INCLUDE_DIR)\chim\c_codegen.h ^
    $(INCLUDE_DIR)\chim\wasm_backend.h

# 对象文件
OBJECTS = $(CHIM_SOURCES:.c=.o)
OBJECTS := $(OBJECTS:$(SRC_DIR)=$(OUT_DIR)\$(SRC_DIR))

# 目标
TARGET = $(OUT_DIR)\chimc.exe

!IF "$(TARGET)" == "$(OUT_DIR)\chimc.exe"
TARGET = $(OUT_DIR)\chimc
!ENDIF

# 默认目标
all: $(OUT_DIR) $(TARGET)

# 创建输出目录
$(OUT_DIR):
    mkdir $(OUT_DIR)
    mkdir $(OUT_DIR)\frontend
    mkdir $(OUT_DIR)\middleend
    mkdir $(OUT_DIR)\backend
    mkdir $(OUT_DIR)\toolchain

# 编译目标
$(TARGET): $(OBJECTS)
    @echo 链接: $@
    $(CC) $(LDFLAGS) $(OBJECTS) -o $@
    @echo 生成: $@

# 编译源文件
{$(SRC_DIR)}.c{$(OUT_DIR)\$(SRC_DIR)}.o:
    @echo 编译: $<
    $(CC) $(CFLAGS) -I$(INCLUDE_DIR) -c $< -o $@

# 清理
clean:
    del /Q $(OUT_DIR)\*.o 2>NUL
    del /Q $(OUT_DIR)\frontend\*.o 2>NUL
    del /Q $(OUT_DIR)\middleend\*.o 2>NUL
    del /Q $(OUT_DIR)\backend\*.o 2>NUL
    del /Q $(OUT_DIR)\toolchain\*.o 2>NUL
    del /Q $(TARGET) 2>NUL
    @echo 清理完成

# 检查
check:
    @echo 检查依赖...
    @if not exist $(INCLUDE_DIR) echo 错误: include 目录不存在
    @if not exist $(SRC_DIR) echo 错误: src 目录不存在
    @echo 检查完成

# 帮助
help:
    @echo Chim 3.1 编译器 Makefile
    @echo.
    @echo 用法: nmake [目标] [选项]
    @echo.
    @echo 目标:
    @echo   all     - 默认构建 (编译器和工具)
    @echo   clean   - 清理构建产物
    @echo   check   - 检查依赖
    @echo   help    - 显示帮助
    @echo.
    @echo 选项:
    @echo   CC=<编译器>     - 指定 C 编译器
    @echo   CFLAGS=<标志>  - 指定编译标志
    @echo   OUTDIR=<目录>  - 指定输出目录
    @echo.
    @echo 示例:
    @echo   nmake all
    @echo   nmake clean CC=clang
    @echo   nmake all CFLAGS="-O3 -Wall"

.PHONY: all clean check help
