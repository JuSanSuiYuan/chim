#!/bin/bash
# Chim 3.1 Compiler Build Script
# 支持 Linux/macOS 和 TCC/GCC/Clang

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 默认配置
CC="${CC:-gcc}"
CFLAGS="-Wall -Wextra -Werror -std=c11 -O2"
LDFLAGS=""
SRC_DIR="src"
INCLUDE_DIR="include"
OUT_DIR="bin"
TARGET="chimc"

# 平台检测
OS="$(uname -s)"
case "$OS" in
    Linux*)
        PLATFORM="linux"
        ;;
    Darwin*)
        PLATFORM="macos"
        ;;
    MINGW*|CYGWIN*|MSYS*)
        PLATFORM="windows"
        ;;
    *)
        PLATFORM="unknown"
        ;;
esac

# TCC 检测
if command -v tcc &> /dev/null; then
    TCC_VERSION=$(tcc -v 2>&1 | head -n1)
    echo -e "${GREEN}[信息]${NC} 找到 TCC: $TCC_VERSION"
    TCC_AVAILABLE=true
else
    echo -e "${YELLOW}[警告]${NC} 未找到 TCC，将使用 $CC 进行编译"
    TCC_AVAILABLE=false
fi

# GCC/Clang 检测
if command -v gcc &> /dev/null; then
    GCC_VERSION=$(gcc --version | head -n1)
    echo -e "${GREEN}[信息]${NC} 找到 GCC: $GCC_VERSION"
elif command -v clang &> /dev/null; then
    CLANG_VERSION=$(clang --version | head -n1)
    echo -e "${GREEN}[信息]${NC} 找到 Clang: $CLANG_VERSION"
fi

# 打印横幅
echo ""
echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           Chim 3.1 编译器构建系统                            ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# 解析参数
case "$1" in
    clean)
        echo -e "${YELLOW}[清理]${NC} 清理构建目录..."
        rm -rf "$OUT_DIR"
        rm -f "*.o" "*.a" "*.so"
        echo -e "${GREEN}[完成]${NC} 清理完成"
        exit 0
        ;;
    check)
        echo -e "${BLUE}[检查]${NC} 检查依赖..."
        if ! command -v make &> /dev/null; then
            echo -e "${RED}[错误]${NC} 需要 make 工具"
            exit 1
        fi
        if ! $TCC_AVAILABLE && ! command -v gcc &> /dev/null && ! command -v clang &> /dev/null; then
            echo -e "${RED}[错误]${NC} 需要 C 编译器 (gcc/clang)"
            exit 1
        fi
        echo -e "${GREEN}[完成]${NC} 所有依赖满足"
        exit 0
        ;;
    help|--help|-h)
        echo "用法: $0 [命令] [选项]"
        echo ""
        echo "命令:"
        echo "  clean       清理构建产物"
        echo "  check       检查依赖"
        echo "  help        显示此帮助"
        echo ""
        echo "选项:"
        echo "  CC=<编译器>     指定 C 编译器 (默认: gcc)"
        echo "  CFLAGS=<标志>   指定 C 编译标志"
        echo "  OUT_DIR=<目录>  指定输出目录 (默认: bin)"
        echo ""
        echo "示例:"
        echo "  $0              # 默认构建"
        echo "  $0 clean       # 清理并构建"
        echo "  CC=clang $0    # 使用 Clang 构建"
        exit 0
        ;;
esac

# 创建输出目录
mkdir -p "$OUT_DIR"

# 源文件列表
CHIM_SOURCES=(
    "$SRC_DIR/common.c"
    "$SRC_DIR/frontend/lexer.c"
    "$SRC_DIR/frontend/parser.c"
    "$SRC_DIR/middleend/ir.c"
    "$SRC_DIR/middleend/optimizer.c"
    "$SRC_DIR/backend/codegen.c"
    "$SRC_DIR/backend/c_codegen.c"
    "$SRC_DIR/backend/wasm_backend.c"
    "$SRC_DIR/toolchain/chim.c"
    "$SRC_DIR/toolchain/chimc.c"
)

# 头文件列表
CHIM_HEADERS=(
    "$INCLUDE_DIR/chim/common.h"
    "$INCLUDE_DIR/chim/token.h"
    "$INCLUDE_DIR/chim/error.h"
    "$INCLUDE_DIR/chim/lexer.h"
    "$INCLUDE_DIR/chim/ast.h"
    "$INCLUDE_DIR/chim/parser.h"
    "$INCLUDE_DIR/chim/ir.h"
    "$INCLUDE_DIR/chim/optimizer.h"
    "$INCLUDE_DIR/chim/codegen.h"
    "$INCLUDE_DIR/chim/c_codegen.h"
    "$INCLUDE_DIR/chim/wasm_backend.h"
)

# 检查源文件
echo -e "${BLUE}[扫描]${NC} 源文件..."
for src in "${CHIM_SOURCES[@]}"; do
    if [ ! -f "$src" ]; then
        echo -e "${RED}[错误]${NC} 源文件不存在: $src"
        exit 1
    fi
done
echo -e "${GREEN}[完成]${NC} 找到 ${#CHIM_SOURCES[@]} 个源文件"

# 检查头文件
echo -e "${BLUE}[扫描]${NC} 头文件..."
for header in "${CHIM_HEADERS[@]}"; do
    if [ ! -f "$header" ]; then
        echo -e "${RED}[错误]${NC} 头文件不存在: $header"
        exit 1
    fi
done
echo -e "${GREEN}[完成]${NC} 找到 ${#CHIM_HEADERS[@]} 个头文件"

# 构建对象文件
echo ""
echo -e "${BLUE}[编译]${NC} 编译源文件..."

# 编译每个源文件
COMPILED=0
for src in "${CHIM_SOURCES[@]}"; do
    obj="${src%.c}.o"
    obj_dir=$(dirname "$obj")
    
    # 确保子目录存在
    mkdir -p "$OUT_DIR/$obj_dir"
    
    # 输出对象文件路径
    out_obj="$OUT_DIR/${src%.c}.o"
    
    # 编译
    if [ "$out_obj" -nt "$src" ]; then
        echo -e "${YELLOW}[跳过]${NC} $src (已是最新)"
        continue
    fi
    
    echo -e "  ${BLUE}->${NC} $src"
    
    $CC $CFLAGS -I"$INCLUDE_DIR" -c "$src" -o "$out_obj"
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}[错误]${NC} 编译失败: $src"
        exit 1
    fi
    
    COMPILED=$((COMPILED + 1))
done

echo -e "${GREEN}[完成]${NC} 编译了 $COMPILED 个文件"

# 收集对象文件
OBJECTS=$(find "$OUT_DIR" -name "*.o" -type f | tr '\n' ' ')

# 链接
echo ""
echo -e "${BLUE}[链接]${NC} 链接目标文件..."

$CC $LDFLAGS $OBJECTS -o "$OUT_DIR/$TARGET"

if [ $? -ne 0 ]; then
    echo -e "${RED}[错误]${NC} 链接失败"
    exit 1
fi

echo -e "${GREEN}[完成]${NC} 生成: $OUT_DIR/$TARGET"

# 检查输出
if [ -f "$OUT_DIR/$TARGET" ]; then
    SIZE=$(du -h "$OUT_DIR/$TARGET" | cut -f1)
    echo -e "${GREEN}[信息]${NC} 输出大小: $SIZE"
fi

# 测试编译
echo ""
echo -e "${BLUE}[测试]${NC} 运行自检..."

if [ -f "$OUT_DIR/$TARGET" ]; then
    "$OUT_DIR/$TARGET" --version
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}[通过]${NC} 自检通过"
    else
        echo -e "${YELLOW}[警告]${NC} 自检失败，编译器可能有问题"
    fi
fi

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                      构建完成!                                ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "用法: $OUT_DIR/$TARGET <输入文件> -o <输出文件>"
echo ""
