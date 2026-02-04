# Chim
Chim 是一个现代化的编程语言，旨在提供简洁、高效的编程体验。

## 项目结构

```
├── docs/            # 文档目录
│   ├── chim3.1语法规范.md    # Chim 3.1 语法规范
│   └── chim工具链.md         # Chim 工具链文档
├── examples/        # 示例代码目录
│   └── examples.md  # 示例说明
├── include/chim/    # 头文件目录
│   ├── ast.h        # 抽象语法树
│   ├── c_codegen.h  # C 代码生成
│   ├── codegen.h    # 代码生成
│   ├── common.h     # 通用定义
│   ├── error.h      # 错误处理
│   ├── ir.h         # 中间表示
│   ├── lexer.h      # 词法分析器
│   ├── optimizer.h  # 优化器
│   ├── parser.h     # 语法分析器
│   ├── token.h      # 词法单元
│   └── wasm_backend.h # WebAssembly 后端
├── src/             # 源代码目录
│   ├── backend/     # 后端代码
│   │   ├── c_codegen.c   # C 代码生成实现
│   │   └── wasm_backend.c # WebAssembly 后端实现
│   ├── frontend/    # 前端代码
│   │   └── parser.c # 语法分析器实现
│   ├── middleend/   # 中端代码
│   │   └── optimizer.c # 优化器实现
│   ├── toolchain/   # 工具链代码
│   │   └── chim.c   # 主入口
│   └── common.c     # 通用实现
├── Makefile         # 构建文件
├── build.sh         # 构建脚本
└── chim.png         # 项目 Logo
```

## 构建方法

### 使用 Makefile

```bash
make
```

### 使用 build.sh 脚本

```bash
./build.sh
```

## 文档

- [Chim 3.1 语法规范](docs/chim3.1语法规范.md)
- [Chim 工具链文档](docs/chim工具链.md)

## 示例

查看 [示例目录](examples/) 了解如何使用 Chim。

## 许可证

本项目采用 [木兰宽松许可证, 第2版](https://license.coscl.org.cn/MulanPSL2/)

