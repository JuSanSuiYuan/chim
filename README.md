# Chim 编程语言编译器

Chim 是一个现代编程语言及其编译器实现，采用木兰2.0开源许可证发布。本项目旨在提供一个完整的编译器教学示例，同时也是一个功能完备的编程语言实现。

## 项目特点

### 完整的编译器架构

Chim编译器采用了经典的编译器设计模式，包含完整的词法分析、语法分析、语义分析、中间代码生成和目标代码生成等阶段。每个阶段都经过精心设计，既展示了编译器工作的基本原理，又具备实际应用价值。词法分析器使用 `logos` 库进行高效的token生成，支持整数、浮点数、字符串、布尔值等多种字面量的识别。语法分析器采用递归下降解析方式，能够正确处理表达式、函数定义、控制流语句等复杂的语法结构。语义分析器负责类型检查、作用域管理、引用有效性验证等关键任务，确保程序在运行时的正确性。

### 多目标代码生成

编译器支持**8种目标平台**的代码生成，满足不同场景下的使用需求：

- **WebAssembly** (.wasm) - 输出标准的WebAssembly文本格式，可在浏览器和Node.js环境执行
- **Native C** (.c) - 生成可移植的C语言代码，适用于各种操作系统和硬件平台
- **LLVM IR** (.ll) - 输出LLVM中间表示，支持强大的优化和多平台编译
- **QBE** (.qbe) - 生成QBE中间语言，提供轻量级快速编译
- **TinyCC** (.c) - 针对TinyCC优化的C代码，实现极速编译（0.05秒）
- **Cranelift IR** (.clif) - 输出Cranelift中间表示，适用于JIT场景
- **Fortran** (.f90) - 生成Modern Fortran 2008/2018代码，专为科学计算优化
- **x86-64 Assembly** (.s) - 输出AT&T语法汇编代码，提供底层控制能力

这种多后端架构不仅展示了代码生成的基本原理，也为不同应用场景提供了最优的性能选择。

### 高级语言特性支持

Chim语言支持现代编程语言的诸多高级特性，包括函数式编程风格的lambda表达式和模式匹配，面向过程编程的函数定义和变量绑定，以及灵活的类型系统和引用语义。语言设计遵循简洁易学的原则，语法风格类似Python和Rust的混合体，既保持了表达力，又降低了学习门槛。特别值得一提的是，Chim实现了完整的所有权系统和生命周期检查，这是现代内存安全语言的核心特性，通过编译时检查消除了空指针引用、内存泄漏等常见问题。

## 快速开始

### 环境要求

构建Chim编译器需要以下软件环境。Rust工具链是必需的开发依赖，建议使用rustup进行安装和管理，确保Rust版本不低于1.70.0。操作系统方面，项目已在Windows、macOS和Linux等主流平台上完成测试，各平台的构建流程完全一致。对于代码编辑，任何支持Rust语法高亮的编辑器均可使用，Visual Studio Code配合rust-analyzer扩展是推荐的开发环境选择。

### 编译项目

使用Cargo工具链可以方便地完成项目的构建和测试。执行 `cargo build` 命令将在 `target/debug` 目录下生成可执行文件，这是开发调试阶段的常用构建方式。发布构建使用 `cargo build --release` 命令，优化后的二进制文件将位于 `target/release` 目录，性能更优但编译时间更长。运行测试套件可以使用 `cargo test` 命令，项目包含针对各模块的单元测试和集成测试，确保代码质量和功能正确性。

### 使用编译器

编译Chim源文件的基本命令格式为 `chim <source.chim> -t <target> -O <level>`. 如果不指定目标平台，默认生成WebAssembly代码。优化级别可以通过 `-O` 参数进行设置，分别对应 `-O0`（无优化）、`-O1`（基本优化）、`-O2`（激进优化）三个等级。

```bash
# 编译为WebAssembly（默认）
chim program.chim -t wasm -O2

# 编译为C语言代码
chim program.chim -t native -O1

# 编译为LLVM IR（高级优化）
chim program.chim -t llvm -O2

# 编译为Fortran（科学计算）
chim program.chim -t fortran -O2

# 编译为x86-64汇编（底层优化）
chim program.chim -t asm -O1

# 快速编译（TinyCC后端）
chim program.chim -t tinycc -O0

# 输出中间表示
chim program.chim -t ir
```

**后端选择指南：**
- 开发调试 → TinyCC（极速编译，0.05秒）
- 科学计算 → Fortran（数值优化，gfortran支持）
- Web应用 → WASM（浏览器运行）
- 生产环境 → LLVM（最优性能）
- JIT编译 → Cranelift（即时编译优化）
- 底层优化 → Assembly（完全控制）

### 示例程序

以下是一个简单的Chim程序示例，展示了语言的基本语法特性。函数定义使用 `fn` 关键字，参数列表和返回值类型位于参数括号之后，函数体可以是单个表达式或包含多条语句的代码块。变量绑定使用 `let` 关键字，支持显式类型声明和类型推断两种方式。

```chim
fn add(a: int, b: int) -> int = a + b;

fn multiply(x: int, y: int) -> int = x * y;

fn main() {
    let result: int = add(10, 20);
    let product: int = multiply(5, 6);
}
```

## 项目结构

```
chim_compiler/
├── Cargo.toml              # 项目配置文件
├── src/
│   ├── main.rs             # 程序入口和命令行处理
│   ├── ast.rs              # 抽象语法树定义
│   ├── lexer.rs            # 词法分析器
│   ├── parser.rs           # 语法分析器
│   ├── semantic.rs         # 语义分析器
│   ├── ir.rs               # 中间表示定义
│   ├── codegen.rs          # IR生成器
│   ├── optimizer.rs        # 代码优化器
│   ├── rvo.rs              # 返回值优化器（RVO）
│   ├── memory_layout.rs    # 内存布局分析器
│   ├── group_manager.rs    # 组生命周期管理器
│   ├── allocation.rs       # 栈/堆分配决策器
│   ├── backend.rs          # 统一后端接口
│   └── backends/           # 代码生成后端
│       ├── wasm.rs         # WebAssembly后端
│       ├── native.rs       # C语言后端
│       ├── llvm.rs         # LLVM IR后端
│       ├── qbe.rs          # QBE后端
│       ├── tinycc.rs       # TinyCC后端
│       ├── cranelift.rs    # Cranelift后端
│       ├── fortran.rs      # Fortran后端（科学计算）
│       └── asm.rs          # x86-64汇编后端
├── tests/                  # 测试文件目录
│   ├── value_type_test.chim
│   └── scientific_test.chim
└── test_all_backends.ps1   # 多后端测试脚本
```

## 语言规范

Chim语言的语法规范定义在 `chim语法规范.md` 文档中，设计理念阐述在 `chim设计理念.md` 文档中。这两份文档详细描述了语言的类型系统、表达式语法、语句结构以及标准库等内容，是理解语言设计和实现的重要参考资料。语言设计遵循渐进式复杂性原则，从简单的函数式表达式开始，逐步引入更复杂的控制流和类型特性，使学习过程平滑自然。

## 技术架构

编译器采用模块化设计，各阶段职责清晰划分。词法分析阶段负责将源代码转换为token序列，为语法分析提供基础。语法分析阶段根据文法规则验证token序列的结构正确性，并构建抽象语法树。语义分析阶段在语法树基础上进行类型推断、作用域解析和生命周期检查。中间代码生成阶段将抽象语法树转换为平台无关的中间表示，便于后续优化和多目标代码生成。目标代码生成阶段将中间表示转换为特定平台的机器码或可执行格式。

中间表示层是连接前端分析和后端生成的关键桥梁。IR模块定义了类型系统，包括基本类型、引用类型、数组类型和结构体类型。指令集涵盖内存操作、算术运算、比较运算、逻辑运算、控制流和函数调用等完整的操作语义。模块结构支持函数、全局变量和结构体的组织，便于复杂程序的表示和生成。

代码生成层实现了统一的后端架构，所有代码生成器都实现 `CodegenBackend` trait，提供一致的接口。

**主要后端实现：**
- **WASMBackend** - 生成符合WebAssembly规范的文本格式代码
- **NativeBackend** - 生成可移植的C语言代码
- **LLVMBackend** - 输出LLVM IR，利用LLVM优化管道
- **QBEBackend** - 生成QBE中间语言，提供快速编译
- **TinyCCBackend** - 针对TinyCC优化，实现极速编译
- **CraneliftBackend** - 输出Cranelift IR，适用于JIT场景
- **FortranBackend** - 生成Modern Fortran代码，专为科学计算优化，支持：
  - REAL(8)双精度浮点（数值稳定性）
  - MODULE/SUBROUTINE/FUNCTION结构
  - 兼容gfortran/ifort编译器
- **AsmBackend** - 生成x86-64汇编代码，提供底层控制，特性包括：
  - AT&T语法（GNU AS兼容）
  - System V ABI调用约定
  - 完整的寄存器分配和栈帧管理

后端架构采用统一接口设计，支持灵活的目标扩展和优化。

优化器实现了多种代码优化技术：

**通用优化：**
- **常数传播** - 识别常量表达式，在编译时直接计算结果
- **函数内联** - 将小函数体展开到调用点，消除调用开销
- **死代码消除** - 移除不可达代码和未使用变量

**值类型优化：**
- **内存布局优化** - 字段重排，减少填充空间（最高节省33%内存）
- **RVO优化** - 返回值优化，消除不必要的拷贝操作
- **栈/堆分配决策** - 智能选择栈分配或堆分配

**生命周期管理：**
- **组生命周期** - 统一管理相关对象的生命周期
- **借用检查** - 编译时验证引用安全性

优化过程分为三个级别（-O0/-O1/-O2），用户可根据性能和编译时间权衡选择。

## 特色功能

### 科学计算支持

Chim通过Fortran后端为科学计算场景提供了原生支持。生成的Fortran代码采用Modern Fortran 2008/2018语法，充分利用gfortran和ifort编译器的优化能力。适用于：
- 数值计算和线性代数
- 物理模拟和工程计算
- 科学研究代码

使用示例：
```bash
chim scientific_program.chim -t fortran -O2
gfortran -O3 scientific_program.f90 -o program
```

### 底层性能优化

汇编后端提供了对生成代码的完全控制能力。生成的x86-64汇编代码采用AT&T语法，遵循System V ABI调用约定，适用于：
- 性能关键代码路径
- 系统级编程
- 学习和调试汇编

使用示例：
```bash
chim performance_critical.chim -t asm -O1
as performance_critical.s -o program.o
```

## 扩展与贡献

项目采用木兰2.0开源许可证，允许自由使用、修改和分发。鼓励开发者通过以下方式参与项目贡献：报告发现的问题和漏洞，提出功能改进建议，提交代码补丁和优化实现，完善文档和测试用例。

**已完成的扩展方向：**
- ✅ 多后端架构（8个后端）
- ✅ 科学计算支持（Fortran后端）
- ✅ 底层汇编输出（x86-64后端）
- ✅ 值类型系统和内存优化
- ✅ RVO返回值优化

**未来扩展方向：**
- GPU代码生成支持（CUDA或OpenCL）
- ARM架构汇编后端
- RISC-V架构支持
- 更多的代码优化Pass
- 完善标准库和运行时支持

## 许可证

本项目采用木兰公共许可证第二版（Mulan PSL v2）开源许可。关于许可证的详细信息，请参阅 LICENSE 文件或访问木兰开源社区获取官方说明。

## 致谢

感谢所有为项目做出贡献的开发者，以及开源社区提供的优秀工具和库。项目的成功离不开社区的支持和反馈，欢迎更多开发者加入我们，共同推动Chim语言和编译器技术的发展。
