# Chim 3.1 工具链完整指南

> "简洁、无歧义、易实现"
>
> Progressive Functional Programming Language Toolchain

---

## 概述

Chim 3.1 是一个完整的编程语言生态系统，提供从编译器到包管理器的全套工具链。本指南详细介绍如何使用和构建 Chim 3.1 工具链。

### 工具链架构

```
┌─────────────────────────────────────────────────────┐
│                    Chim 3.1 工具链                        │
├─────────────────────────────────────────────────────┤
│  用户界面层  │         统一 chim 命令 CLI                     │
│  核心命令    │  init build install add remove update list          │
│              │  audit fmt lint docs bench test                   │
├─────────────────────────────────────────────────────┤
│  编译器层    │    词法分析器    │    语法解析器    │    语义分析器      │
├─────────────────────────────────────────────────────┤
│  后端层      │     C            │     WASM        │     Native Code   │
│  编译引擎    │        C + TCC 编译引擎                         │
├─────────────────────────────────────────────────────┤
│  标准库层    │     基础库       │     数学库       │     字符串库       │
├─────────────────────────────────────────────────────┤
│  包管理      │   依赖解析       │    缓存管理      │    硬链接优化     │
│  构建系统    │      统一构建管道     │    增量编译    │    并行处理       │
└─────────────────────────────────────────────────────┘
```

---

## 快速开始

### 1. 安装工具链

```bash
# 使用 LLVM 或 TCC 编译
git clone https://gitee.com/iloverust/chim
cd chim

# Linux/macOS
chmod +x build.sh
./build.sh

# Windows (使用 MinGW + LLVM)
mingw32-make
# 或
make

# 添加到PATH
export PATH=$PATH:/path/to/chim/bin
```

### 2. 创建第一个项目

```bash
# 初始化项目
chim init my-project
cd my-project

# 安装依赖
chim install

# 构建项目
chim build

# 运行
chim run src/main.chim
```

### 3. 编写第一个程序

```chim
# src/main.chim
# Chim 3.1 渐进式函数式编程

fn fibonacci(n: int): int =
  match n:
    0 => 0
    1 => 1
    _ => fibonacci(n - 1) + fibonacci(n - 2)

let result = fibonacci(10)
println("Fibonacci(10) = " + str(result))

# 顶层代码直接执行
```

---

## 核心特性

### 统一命令架构

Chim 3.1 采用**统一命令架构**，所有功能都集成在单一的 `chim` 命令中：

#### 核心优势
- **单一入口点**: 所有功能通过 `chim` 子命令访问
- **一致体验**: 统一的CLI设计和用户体验
- **C引擎**: 基于C语言和TCC的编译引擎，支持多平台运行
- **无工具链分割**: 不需要区分编译器、包管理器、构建工具
- **pnpm风格包管理**: 零复制安装、全局缓存
- **cargo风格构建**: 声明式配置、命令式脚本

#### 完整命令映射
```bash
# 项目管理
chim init       # 项目初始化
chim build      # 项目构建
chim install    # 依赖安装
chim add        # 添加依赖
chim remove     # 移除依赖
chim update     # 更新依赖
chim list       # 列出依赖
chim audit      # 安全审核

# 编译运行
chim compile    # 编译文件
chim run        # 编译并运行
chim test       # 运行测试

# 开发工具
chim fmt        # 代码格式化
chim lint       # 代码检查
chim docs       # 文档生成
chim bench      # 性能测试

# 统一选项
--target c      # C代码目标
--target wasm   # WebAssembly目标
--debug         # 调试模式
--optimize      # 优化编译
```

---

## 核心工具详解

### 1. 统一 Chim 命令 (chim)

#### 完整命令列表

```bash
# 项目管理
chim init [name]              # 初始化新项目
chim build [target]           # 构建项目
chim install                  # 安装依赖
chim add <package>           # 添加包
chim remove <package>        # 移除包
chim update                  # 更新依赖
chim list                    # 列出已安装包
chim audit                   # 安全审核

# 编译和运行
chim compile <file>          # 编译文件
chim run <file>             # 编译并运行
chim test                   # 运行测试

# 实用工具
chim fmt [files]            # 代码格式化
chim lint [files]           # 代码检查
chim docs                   # 文档生成
chim bench                  # 性能测试

# 全局选项
-v, --version               # 显示版本
-h, --help                 # 显示帮助
--verbose                   # 详细输出
--debug                     # 调试模式
--optimize                  # 优化编译
--target <target>          # 目标架构 (c/wasm)
--output <file>            # 输出文件
```

#### 使用示例

```bash
# 创建新项目
chim init my-project
cd my-project

# 安装依赖
chim install

# 添加依赖
chim add json-parser

# 构建项目
chim build

# 编译并运行
chim run src/main.chim

# 编译为WebAssembly
chim build --target wasm

# 调试编译
chim build --debug --verbose

# 优化编译
chim build --optimize -O2
```

#### 编译流程

```
源代码 (.chim)
     ↓
  词法分析器
     ↓
  Token流
     ↓
  语法解析器
     ↓
  AST抽象语法树
     ↓
  语义分析器
     ↓
  类型检查
     ↓
  中间表示 (IR)
     ↓
  优化器
     ↓
  代码生成器
     ↓
  目标代码 (C/WASM)
     ↓
  TCC/GCC 编译 → 可执行文件
```

### 2. 包管理功能

#### 包管理命令

```bash
# 项目初始化
chim init                    # 创建新项目
chim init --template lib    # 创建库项目

# 包管理
chim install                # 安装所有依赖
chim add <package>         # 添加包
chim add <package>@<ver>   # 添加指定版本
chim add --dev <package>  # 添加开发依赖
chim remove <package>      # 移除包
chim update                # 更新依赖
chim update <package>      # 更新指定包

# 查看和审核
chim list                  # 列出已安装包
chim list --tree          # 树形结构显示
chim audit                 # 安全审核
```

#### pnpm风格包管理

- **零复制安装**: 使用硬链接避免重复文件，节省磁盘空间70-90%
- **全局统一缓存**: 所有包版本存储在单一位置 ($HOME/.chim-cache)
- **高效依赖共享**: 通过符号链接实现node_modules/.pnpm风格的依赖结构
- **严格模式**: 严格的依赖树检查，确保版本一致性

#### cargo风格包管理

- **TOML配置**: 使用TOML格式的package.chim配置文件
- **语义化版本**: 严格遵循SemVer 2.0.0版本管理
- **工作区支持**: 完整的workspace支持
- **依赖锁文件**: 自动生成并维护chim-lock文件
- **构建脚本**: 支持build.rs风格的构建前处理脚本

#### 配置文件 (package.chim)

```toml
# Chim 3.1 Project Configuration
name = "my-chim-project"
version = "1.0.0"
description = "A Chim project"
main = "src/main.chim"
language = "chim"

[scripts]
build = "chim build"
test = "chim test"
dev = "chim run src/main.chim"
bench = "chim bench"

[dependencies]
json = "^1.5.0"
logging = "^2.1.0"
http = "^2.0.0"

[dev-dependencies]
testing = "^1.0.0"

[build]
entry_point = "src/main.chim"
output_dir = "build"
optimize = false
debug = true

[targets]
c = { enabled = true }
wasm = { enabled = true }

[engines]
chim = ">=3.1.0"

[package]
license = "Mulan-2.0"
author = "Your Name"
repository = "https://gitee.com/iloverust/chim"
```

#### 包版本规则

```
^1.2.3     # 兼容版本 (1.2.3 到 <2.0.0)
~1.2.3     # 补丁版本 (1.2.3 到 <1.3.0)
1.2.3      # 精确版本
latest      # 最新版本
*           # 任意版本
1.2.x      # 匹配次要版本
```

### 3. 构建系统 (chim build)

#### 构建命令

```bash
# 默认构建
chim build

# 指定目标
chim build --target c           # C代码（默认）
chim build --target wasm        # WebAssembly

# 调试构建
chim build --debug

# 优化构建
chim build --optimize -O2

# 清理后构建
chim build --clean

# 重新构建
chim build --force

# 并行构建
chim build --jobs 4
```

#### 构建系统架构

**类似 Zig Build 的设计哲学：**

1. **声明式配置**: package.chim 中声明构建配置
2. **命令式脚本**: build.chim 中编写自定义构建逻辑
3. **统一命令**: 单一 `chim build` 命令处理所有构建

#### 声明式配置

```toml
[build]
compiler = "chim"
entry_point = "src/main.chim"
output_dir = "build"
optimize = true
debug = false

[targets]
js = { backend = "c", arch = "universal" }
wasm = { backend = "wasm", arch = "wasm32" }
wasm64 = { backend = "wasm", arch = "wasm64" }
native = { backend = "c", arch = "x86_64" }

[build_flags]
warnings = true
optimization = "O2"
```

#### 命令式构建脚本 (build.chim)

```chim
# build.chim
# 自定义构建逻辑

fn custom_build(): void =
  # 1. 生成资源文件
  generate_assets()

  # 2. 预处理代码
  preprocess_code()

  # 3. 编译为多个目标
  compile_target("c")
  compile_target("wasm")

  # 4. 运行测试

  # 5. 打包发布
  package_release()

fn generate_assets(): void =
  # 生成配置文件
  let asset_files = ["config.json", "schema.json"]
  for file in asset_files:
    println("Generating: " + file)

fn compile_target(target: string): void =
  println("Compiling for target: " + target)
  let backend = match target:
    "c" => "c"
    "wasm" => "wasm"
    _ => "c"

  # 使用统一的chim命令
  let chim_command = "chim build --target " + target
  exec(chim_command)

# 执行构建
custom_build()
```

### 4. C 编译引擎

#### C 代码生成器

Chim 3.1 将代码编译为可读的 C 代码，然后使用 TCC 或 GCC 编译为最终可执行文件：

```bash
# 编译流程
chim build

# 内部流程：
# 1. chim -> C 代码 (build/main.c)
# 2. tcc/gcc -> 可执行文件 (build/main)
```

#### TCC 集成

```bash
# 安装 TCC
# Ubuntu/Debian:
sudo apt-get install tcc

# macOS:
brew install tcc

# 使用 TCC 快速编译
chim build

# 手动指定编译器
chim build --cc gcc
```

---

## 开发工具

### chim test - 运行测试

```bash
# 运行所有测试
chim test

# 运行指定测试
chim test tests/unit

# 详细输出
chim test --verbose

# 覆盖率
chim test --coverage
```

### chim fmt - 代码格式化

```bash
# 格式化所有文件
chim fmt

# 检查格式化
chim fmt --check

# 格式化指定文件
chim fmt src/main.chim lib/utils.chim
```

### chim lint - 代码检查

```bash
# 检查所有文件
chim lint

# 详细输出
chim lint --verbose

# 修复自动修复的问题
chim lint --fix
```

### chim docs - 文档生成

```bash
# 生成文档
chim docs

# 指定输出目录
chim docs --output docs/

# 详细输出
chim docs --verbose
```

### chim bench - 性能测试

```bash
# 运行基准测试
chim bench

# 指定测试文件
chim bench benchmarks/

# 详细输出
chim bench --verbose
```

### chim audit - 安全审核

```bash
# 审核依赖
chim audit

# 详细输出
chim audit --verbose

# 修复安全问题
chim audit --fix
```

---

## 项目结构

### 标准目录结构

```
my-project/
├── src/                   # 源代码目录
│   ├── main.chim         # 入口文件
│   ├── lib/              # 库代码
│   │   ├── math.chim
│   │   ├── string.chim
│   │   └── utils/
│   └── modules/           # 子模块
│       ├── core/
│       └── utils/
├── test/                  # 测试文件
│   ├── unit/
│   │   ├── test_math.chim
│   │   └── test_string.chim
│   └── integration/
├── examples/              # 示例程序
│   └── basic/
├── build/                 # 构建产物
├── docs/                  # 文档
├── benchmarks/            # 基准测试
├── package.chim          # 项目配置
├── build.chim            # 构建脚本（可选）
├── Makefile              # Makefile（可选）
└── README.md
```

### 工作区结构

```toml
# workspace.chim
[workspace]
members = [
  "packages/core",
  "packages/utils",
  "packages/web"
]
packages = {
  core = "packages/core"
  utils = "packages/utils"
  web = "packages/web"
}
```

---

## 高级特性

### 1. 自举编译

```bash
# 第一阶段: 使用C编译器
./build.sh

# 第二阶段: 使用Chim编译器编译自己
chim build --bootstrap

# 验证自举
diff build/chim build/chim_bootstrap
```

### 2. 多目标编译

```bash
# C 后端（默认）
chim build --target c

# WebAssembly 后端
chim build --target wasm

# 同时编译多个目标
chim build --target c --target wasm
```

### 3. 优化器

支持多种优化级别：

| 级别 | 说明 | 包含优化 |
|------|------|---------|
| -O0 | 无优化 | 仅解析和代码生成 |
| -O1 | 基础优化 | 常量折叠、死代码消除 |
| -O2 | 标准优化 | + 公共子表达式消除、循环优化 |
| -O3 | 激进优化 | + 函数内联、矢量化 |

```bash
# 使用优化级别
chim build --optimize -O2
```

### 4. 中间表示

```bash
# 生成 IR 用于调试
chim build --emit-ir

# 打印 IR 到 stdout
chim build --print-ir
```

### 5. 抽象语法树

```bash
# 生成 AST (JSON格式)
chim build --emit-ast

# 打印 AST 到 stdout
chim build --print-ast
```

---

## 错误处理和调试

### 1. 编译错误

```bash
# 详细错误信息
chim build --verbose

# 显示错误上下文
chim build --show-context

# 类型错误详细说明
chim build --detailed-types
```

### 2. 运行时错误

```chim
# 使用Result类型处理错误
fn safe_divide(a: float, b: float): Result[float, string] =
  match b:
    0.0 => Error("Division by zero")
    _ => Success(a / b)

# 使用match处理错误
let result = safe_divide(10.0, 2.0)
match result:
  Success(value) => println("Result: " + str(value))
  Error(message) => println("Error: " + message)
```

### 3. 调试技巧

```bash
# 生成调试信息
chim build --debug

# 使用 GDB/LLDB
gdb ./build/main

# 在 GDB 中设置断点
(gdb) break main
(gdb) run
(gdb) backtrace
```

---

## 最佳实践

### 1. 代码组织

```chim
# 函数: 蛇形命名
fn calculate_sum(numbers: [int]): int = 0

# 类型: 驼峰命名
type ResultValue = Success(value: string) | Error(message: string)

# 常量: 大写蛇形
const MAX_SIZE = 1000

# 模块: 小写驼峰
module json_parser
module math_utils
```

### 2. 错误处理

```chim
# 优先使用Result类型
fn risky_operation(): Result[Value, Error] = ...

# 提供默认值
fn safe_operation(default: Value): Value =
  match risky_operation():
    Success(v) => v
    Error(_) => default

# 使用Option类型
fn find_item(items: [int], target: int): Option[int] =
  match items:
    [] => None
    [x, ...rest] =>
      if x == target:
        Some(x)
      else:
        find_item(rest, target)
```

### 3. 性能考虑

```chim
# 使用不可变数据结构
let immutable_data = original_data.map(transform)

# 使用尾递归优化
fn factorial(n: int, acc: int): int =
  match n:
    0 => acc
    _ => factorial(n - 1, n * acc)

# 避免不必要的复制
fn process_large_data(data: Data): void =
  # 直接操作引用
  transform_in_place(data)
```

---

## 常见问题

### 1. 包管理问题

**Q: 依赖冲突**
A: 使用`chim update`更新依赖版本，或手动修改package.chim

**Q: 缓存问题**
A: 运行`chim cache clean`清理缓存

**Q: 安装失败**
A: 检查网络连接和注册表可用性

### 2. 编译问题

**Q: 编译时出现"未定义的符号"错误**
A: 检查是否正确链接了所需的库和模块

**Q: 类型推断失败**
A: 添加明确的类型注解帮助编译器推断

**Q: 模式匹配不完整**
A: 使用通配符`_`匹配所有剩余情况

### 3. 运行时问题

**Q: 栈溢出**
A: 检查递归调用，确保使用尾递归优化

**Q: 内存泄漏**
A: 确保所有资源都有对应的释放操作

**Q: 性能问题**
A: 使用`chim bench`分析性能瓶颈

---

## 工具链扩展

### 1. 开发自定义插件

```c
// plugins/custom_lint.c
#include <stdio.h>
#include "chim_plugin_api.h"

void register_custom_lint(PluginRegistry* registry) {
    register_lint_rule(registry, "custom_rule", custom_lint);
}

LintResult custom_lint(const char* source) {
    // 自定义lint规则实现
    return create_lint_result();
}
```

### 2. IDE集成

```json
// .vscode/settings.json
{
    "chim.languageServer": {
        "enabled": true,
        "path": "./bin/chim-lsp"
    },
    "chim.compiler": "./bin/chim",
    "chim.format": "./bin/chim fmt",
    "chim.lint": "./bin/chim lint"
}
```

---

## 生态系统

### 核心包

```bash
# 官方包
chim add chim/std      # 标准库
chim add chim/cli      # 命令行工具
chim add chim/web      # Web开发
chim add chim/database # 数据库接口

# 第三方包
chim add json-parser   # JSON解析
chim add http-client   # HTTP客户端
chim add crypto-lib    # 加密库
```

### IDE支持

- **VS Code**: 完整的Chim扩展
- **Vim/Neovim**: Chim语法高亮和LSP
- **Emacs**: 主模式支持
- **IntelliJ**: 插件开发中

---
## 贡献指南

### 1. 提交PR

1. Fork项目
2. 创建特性分支
3. 编写测试
4. 提交代码
5. 创建Pull Request

### 2. 开发环境

```bash
# 克隆仓库
git clone https://gitee.com/iloverust/chim
cd chim

# 构建编译器
chmod +x build.sh
./build.sh

# 运行测试
make test

# 代码检查
make lint
```

### 3. 代码规范

- 遵循Chim 3.1语法规范
- 添加充分的注释
- 编写单元测试
- 保持向后兼容

---

## 许可证

Chim 3.1 采用 **木兰宽松许可证 2.0 (Mulan Permissive Software License v2, MPSLv2)**。

详见 [LICENSE](LICENSE) 文件。

**木兰2.0许可证特点：**
- 宽松的商业使用许可
- 无传染性，可与其他许可证混合使用
- 支持专利授权
- 适合开源项目和商业项目

---

## 社区和支持

- **官网**: https://gitee.com/iloverust/chim
- **Gitee**: https://gitee.com/iloverust/chim


---

## 结语

Chim 3.1 工具链代表了现代编程语言工具链的最新发展。通过简洁无歧义的语法、C+TCC编译引擎、pnpm+cargo风格的包管理和构建系统，Chim 3.1 为开发者提供了强大而优雅的编程体验。

**"简洁、无歧义、易实现"** - 这就是Chim 3.1的核心价值。

无论您是编程语言研究者、系统开发者，还是追求代码优雅的工程师，Chim 3.1 都能为您提供令人愉悦的开发体验。

---

*本文档持续更新中。如有问题或建议，请访问我们的社区或提交Issue。*
