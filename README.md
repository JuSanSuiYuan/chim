# Chim 编程语言

**中文名：启语** | 强类型静态编译语言

---

## 📖 目录

- [语言特性](#-语言特性)
- [核心语法](#-核心语法)
- [编译器架构](#-编译器架构)
- [工具链](#-工具链)
- [快速开始](#-快速开始)
- [构建指南](#-构建指南)
- [测试与验证](#-测试与验证)
- [当前实现状态](#-当前实现状态)

---

## ✨ 语言特性

### 核心理念
- **强类型静态编译**：编译期类型检查，生成高性能原生代码
- **缩进式语法**：Python/Nim 风格，使用冒号 `:` + 缩进，无花括号
- **中英双轨关键字**：`匹配`/`match`、`案例`/`case`、`返回`/`return` 等双语支持
- **多方言支持**：`safe`/`system`/`compat` 满足不同安全级别需求

### 独特设计

#### 1. Swift 风格模式匹配
- **元组模式**：`案例 (0, 0) ->` 精确匹配坐标
- **范围匹配**：`案例 90..100 ->` 支持半开/闭区间
- **Guard 条件**：`案例 x 当 x > 0 ->` 附加条件判断
- **模式变量绑定**：自动解构并绑定变量

#### 2. 底层编程能力
- **原始指针**：`&variable` 取地址，`ptr.*` 解引用（支持 Zig/C 两种风格）
- **内联汇编**：`asm volatile ("code")` 直接嵌入汇编代码
- **端口 I/O**：`inb()/outb()` 等硬件访问指令
- **十六进制字面量**：`0x3F8` 等底层开发必备

#### 3. 并发与通道
- **通道类型**：`chan<T>` 用于协程间通信
- **方向运算符**：`<-` 避免与赋值符号冲突
- **发送/接收**：`ch <- value` 和 `value := <- ch`

#### 4. 组生命周期系统
- **组块管理**：`组 名称:` 自动管理资源生命周期
- **Arena 分配器**：批量分配，统一释放
- **快照与句柄**：跨组传递的安全机制

---

## 🔧 工具链

### 1. 包管理器 `chim mod`

**特性**
- **TOML 风格配置**：`package.chim` 清晰易读
- **硬链接仓库**：类似 pnpm，全局存储，项目硬链接，节省空间
- **依赖锁定**：`chim.lock` 确保可重现构建

**常用命令**
```bash
chim mod init                    # 初始化项目
chim mod add <name>@<version>    # 添加依赖
chim mod remove <name>           # 移除依赖
chim mod install                 # 安装依赖
chim mod update                  # 更新依赖
chim mod link                    # 本地包链接
chim mod publish                 # 发布包
```

**目录结构**
```
项目/
├── package.chim      # 项目清单
├── chim.lock         # 依赖锁定
└── vendor/           # 依赖目录（硬链接）
```

### 2. 编译器 `chim cl`

**构建命令**
```bash
chim cl build                    # 构建项目
chim cl run                      # 构建并运行
chim cl test                     # 运行测试
```

**常用参数**
```bash
--dialect <safe|system|compat>   # 选择方言
--out <path>                     # 输出路径
--release                        # 发布模式
```

### 3. 格式化工具 `chim fmt`

**功能**
- 统一缩进风格
- 规范化构造/调用语法
- 关键字风格统一
- 默认分支规范化（`默认 ->` → `_ ->`）

**使用方法**
```bash
chim fmt .                       # 格式化当前项目
chim fmt --check .               # 检查格式
chim fmt --fix .                 # 自动修复
```

---

## 🏗️ 编译器架构

### 前端设计

#### Python 实现（主版本）
- **完整功能**：词法分析、语法分析、语义分析
- **开发友好**：易于调试和扩展
- **性能**：1.56ms/次编译

#### Rust 优化前端
- **高性能词法分析**：2.03x 性能提升
- **C FFI 导出**：可被其他语言调用
- **性能**：0.77ms/次编译（含调试输出）

### 后端支持

#### 1. Python 后端
- **快速原型**：生成 Python 代码，即时执行
- **调试便利**：代码可读性高
- **适用场景**：快速开发、脚本编写

#### 2. Zig 后端
- **原生编译**：Chim → Zig → 可执行文件
- **高性能**：接近 C 语言性能
- **完整运行时**：
  - **大小**：10.5KB 轻量级
  - **Channel 通信**：create/send/recv/close/destroy
  - **Arena 分配器**：批量内存管理
  - **动态数组**：push/pop/get/set
  - **字符串操作**：concat/substring/equals
  - **数学函数**：abs/sqrt/pow/三角函数等

### 性能对比

| 组件 | 性能 | 提升 |
|------|------|------|
| Python 前端 | 1.56ms/次 | 基准 |
| Rust 前端 | 0.77ms/次 | **2.03x** |
| Zig 后端 | 原生代码 | **接近 C** |

---

## 🔨 构建指南

### Windows 平台

#### 步骤 1：Rust 前端（可选，性能优化）

```bash
cd Chim/compiler-rs
cargo build --release --target i686-pc-windows-msvc
```

**输出**：`chim_frontend.dll`（32位，匹配 Python 环境）

#### 步骤 2：Zig 运行时库

```bash
cd Chim/runtime-zig/src
zig build-lib lib.zig -dynamic -lc -target x86-windows-msvc
```

**输出**：`lib.dll`（Chim 运行时支持库）

#### 步骤 3：编译示例

**方式 A：Python 后端（快速开发）**
```bash
python Chim/compiler/main.py Chim/examples/hello.chim
python Chim/build/hello.py
```

**方式 B：Zig 后端（原生编译）**
```bash
cd Chim/tests
python test_zig_full_pipeline.py
```

**完整流程**：Chim 源码 → Zig 代码 → 编译 → 可执行文件

---

## 🚀 快速开始

### 示例 1：Hello World

**代码**：`hello.chim`
```chim
system

fn add(a: 整数, b: 整数) -> 整数:
    返回 a + b

fn 主():
    输出("计算结果:")
    令 x := 10
    令 y := 20
    令 result := add(x, y)
    输出("x + y =", result)
```

**运行**
```bash
python compiler/main.py examples/hello.chim
python build/hello.py
```

### 示例 2：模式匹配

#### 元组模式
```chim
匹配 点:
    案例 (0, 0) -> 输出("原点")
    案例 (x, 0) -> 输出("在X轴")
    案例 (0, y) -> 输出("在Y轴")
    _ -> 输出("其他位置")
```

#### 范围匹配
```chim
匹配 分数:
    案例 90..100 -> 输出("优秀")
    案例 80..90 -> 输出("良好")
    案例 60..80 -> 输出("及格")
    _ -> 输出("不及格")
```

#### Guard 条件
```chim
匹配 值:
    案例 x 当 x > 0 -> 输出("正数")
    案例 x 当 x < 0 -> 输出("负数")
    _ -> 输出("零")
```

### 示例 3：底层编程

#### 指针操作
```chim
system

fn 测试指针():
    设 x := 42
    设 px := &x        # 取地址
    设 y := px.*       # 解引用（Zig 风格）
    设 z := *px        # 解引用（C 风格）
    返回 y
```

#### 内联汇编
```chim
fn 禁用中断():
    asm volatile ("cli")  # 关闭中断
    返回
```

#### 端口 I/O
```chim
fn 读端口(port: 整数) -> 整数:
    设 value := inb(port)  # 读取端口
    outb(port, value)      # 写入端口
    返回 value
```

### 示例 4：组生命周期

```chim
system

fn 主():
    组 临时数据:
        令 buffer := arena()        # Arena 分配器
        令 data := 数组(1, 2, 3, 4, 5)
        
        # 快照：只读副本
        令 snapshot := 快照 data
        
        # 句柄：轻量引用
        令 handle := 句柄 data
        
        输出("组内处理:", data)
    # 组结束时自动释放所有资源
    输出("组外完成")
```

### 示例 5：并发与通道

```chim
fn 通道示例():
    令 ch := chan<整数>(16)  # 创建容量为16的通道
    
    ch <- 1                  # 发送数据
    ch <- 2
    
    设 x := <- ch            # 接收数据
    设 y := <- ch
    
    输出("接收到:", x, y)
```

---

## 🧪 测试与验证

### 运行测试套件

```bash
cd Chim/tests

# 基础编译流程测试
python test_simple.py

# Zig 后端完整流水线
python test_zig_full_pipeline.py

# Zig 运行时 FFI 测试
python test_zig_runtime_ffi.py

# 组生命周期测试
python test_simple_group.chim

# 性能对比测试
python benchmark_frontend.py

# 指针、汇编、端口 I/O 测试
python build/test_final_all.py
```

### 测试覆盖

- ✅ **31 个测试文件**
- ✅ **全部核心功能覆盖**
- ✅ **端到端编译流程验证**

---

## 📋 核心语法

### 包配置示例

**package.chim**（TOML 风格）
```toml
[package]
name = "hello-chim"
version = "0.1.0"
dialect = "system"

[dependencies]
chim-std = "0.1.*"
```

### 代码风格约定

- **命名规范**
  - 类型名：首字母大写（`Person`、`整数`）
  - 函数名：小写（`add`、`计算`）
  - 变量名：小写（`value`、`结果`）

- **语法规范**
  - 构造：使用命名参数 `Person(name: "张三", age: 25)`
  - 调用：使用位置参数 `add(10, 20)`
  - 可变性：显式书写 `令`（不可变）或 `设`（可变）
  - 行尾：禁止使用分号

- **格式化**
  - 使用 `chim fmt` 保持风格一致
  - Match 语句：`匹配 值:` 而非 `匹配 值 |`
  - 默认分支：统一为 `_ ->`

### 方言系统

#### `safe` 方言（默认）
- 禁止不安全操作
- 内存安全保证
- 适用：应用层开发

#### `system` 方言
- 允许指针操作
- 支持内联汇编
- 允许端口 I/O
- 适用：系统编程、驱动开发

#### `compat` 方言
- 兼容 if/else/while 语法
- 平滑迁移
- 适用：代码迁移期

#### `dynamic` 方言（预览）
- 动态类型
- 多重分派
- 适用：快速原型、REPL

#### `DSL/toml` 方言（预览）
- TOML 配置文件
- 用于 `package.chim`
- 交由 DSL 解析器处理

---

## ✅ 当前实现状态

### 编译器前端
- ✅ **完整实现**：词法、语法、语义分析
- ✅ **Rust 优化**：2x 性能提升
- ✅ **C FFI 导出**：跨语言调用

### 编译器后端
- ✅ **Python 后端**：快速原型开发
- ✅ **Zig 后端**：原生高性能编译
- ✅ **端到端流程**：Chim → Zig → 可执行文件

### 运行时库
- ✅ **Channel 通信**：create/send/recv/close/destroy
- ✅ **Arena 分配器**：批量内存管理
- ✅ **动态数组**：push/pop/get/set
- ✅ **字符串操作**：concat/substring/equals
- ✅ **数学函数**：abs/sqrt/pow/三角函数

### 语言特性
- ✅ **组生命周期**：group/snapshot/handle/arena
- ✅ **模式匹配**：元组/范围/guard 条件
- ✅ **底层编程**：指针/内联汇编/端口 I/O
- ✅ **并发通道**：chan/发送/接收
- ✅ **十六进制字面量**：0x...

### 测试覆盖
- ✅ **31 个测试文件**
- ✅ **全部核心功能**
- ✅ **性能基准测试**

---

## 📚 适用场景

### 系统编程
- ✅ 操作系统内核开发
- ✅ 设备驱动开发
- ✅ 嵌入式系统
- ✅ 底层硬件控制

### 应用开发
- ✅ 高性能计算
- ✅ 网络服务
- ✅ 命令行工具
- ✅ 快速原型

---

## 📄 许可证

木兰宽松许可证 2.0 (MulanPSL-2.0)

---

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！
