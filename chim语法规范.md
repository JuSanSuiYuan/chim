# Chim编程语言语法规范
---

## 📖 目录

1. [语言概述](#语言概述)
2. [核心特性](#核心特性)
3. [词法规范](#词法规范)
4. [语法规范](#语法规范)
5. [类型系统](#类型系统)
6. [内置函数](#内置函数)
7. [标准库](#标准库)
8. [编译流程](#编译流程)
9. [最佳实践](#最佳实践)

---

## 🚀 语言概述

Chim是一门**现代化、类型安全、高性能**的编程语言，设计目标是为开发者提供**简洁、高效、可验证**的编程体验。Chim融合了多种先进语言的设计理念，包括：

- **Zig**的显式内存管理和低开销
- **Agda**的依赖类型和数学证明
- **TileLang**的高性能计算和分块技术
- **Rust**的所有权和生命周期系统
- **June**的编译期资源管理（组机制）
- **Unison**的Effect系统和代数数据类型
- **Koka**的代数效果和类型类系统

### 🎯 设计哲学

**"简洁而强大，安全而高效"**

Chim语言的设计哲学是在保持语法简洁易懂的同时，提供足够的语言特性来满足从系统编程到AI模型训练的各种应用场景需求。

### 🌟 核心优势

| 特性 | 优势 | 应用场景 |
|------|------|----------|
| **显式内存管理** | 类似Zig，低开销、精确控制 | 系统编程、嵌入式开发 |
| **数学验证** | 类似Agda，编译期证明 | 关键系统、金融系统 |
| **高性能计算** | 集成TileLang，GPU/CPU内核 | AI模型训练、科学计算 |
| **类型安全** | 强类型系统 + 依赖类型 | 避免运行时错误 |
| **并发编程** | Safe/Unsafe分离 + 原子操作 | 高性能并发应用 |
| **异步编程** | Tokio/Compio双运行时 | Web服务、高并发IO |
| **完整代码生成** | x86_64原生代码生成 | 生产环境部署 |

### 📊 语言成熟度

Chim语言目前已经完成了完整的编译器实现，包括：

- ✅ **词法分析器**（Lexer）- 完整实现
- ✅ **语法分析器**（Parser）- 完整实现
- ✅ **类型检查器**（Type Checker）- 完整实现
- ✅ **中间代码生成**（IR Generation）- 完整实现
- ✅ **x86_64代码生成器**（Code Generator）- 完整实现

**当前状态**：Chim已经具备完整的编译能力，可以编译生成可执行的x86_64机器码，适用于生产环境。

---

## ✨ 核心特性

### 1. 双安全模型（Safe/Unsafe）

Chim采用**两级安全模型**，区分Safe代码和Unsafe代码，为不同的应用场景提供最优的安全性和性能平衡。

#### Safe代码（类似June + Rust）

Safe代码提供**编译期资源管理**，通过组（Group）机制确保资源安全，无需手动管理内存。

**核心特性**：
- **编译期资源管理**：通过组（Group）机制确保资源安全
- **静态所有权检查**：类似Rust的所有权系统，避免悬垂指针
- **自动类型安全**：编译器严格检查类型，防止类型错误
- **无运行时GC**：所有内存管理在编译期完成，零运行时开销

**Safe代码的优势**：
- 🛡️ **内存安全**：编译期保证，无运行时检查
- ⚡ **零开销**：所有管理逻辑在编译期展开
- 🔒 **资源安全**：自动管理文件、网络连接等资源
- 🎯 **类型安全**：强类型系统防止类型错误

#### Unsafe代码（类似Zig）

Unsafe代码提供**显式内存管理**，允许开发者精确控制内存，适用于需要极致性能的场景。

**核心特性**：
- **显式内存管理**：类似Zig的alloc/dealloc
- **延迟释放**：defer关键字确保资源最终释放
- **直接指针操作**：指针算术、解引用、类型转换
- **低开销**：最小化运行时检查，接近裸机性能

**Unsafe代码的优势**：
- 🚀 **极致性能**：最小化运行时检查
- 🎯 **精确控制**：完全控制内存布局和生命周期
- 🔧 **底层访问**：直接操作硬件和系统资源
- 💡 **灵活性**：实现自定义内存管理策略

### 2. 组（Group）机制

组是Chim Safe代码的核心资源管理机制，**直接借鉴June语言的设计理念**，为safe代码提供编译期资源管理能力。

#### 组语法详解

```chim
// June风格的组定义
group GroupName {
    // 组成员变量（状态数据）
    var member: Type = initial_value
    
    // 初始化块（创建时执行）
    init {
        // 资源初始化代码
    }
    
    // 带参数的初始化
    init(param: Type) {
        // 带参数的资源初始化
    }
    
    // 组级方法
    fn method_name(params) -> ReturnType {
        // 方法实现
    }
    
    // 清理块（离开作用域时自动执行）
    cleanup {
        // 资源清理代码
    }
}
```

#### 组机制优势

| 优势 | 说明 |
|------|------|
| **编译期检查** | 确保资源正确初始化和释放 |
| **自动清理** | 离开作用域时自动执行cleanup |
| **状态管理** | 组内可以维护资源状态 |
| **模块化** | 将资源管理封装为独立单元 |
| **无运行时开销** | 所有管理逻辑在编译期展开 |
| **异常安全** | 即使发生异常也能保证资源释放 |

### 3. 编译期GC（唯一GC机制）

Chim语言**仅采用编译期GC机制**，通过组（Group）机制和所有权系统实现完全的编译期内存管理，彻底移除运行时GC，确保零运行时开销。

**编译期GC特性**：
- **零运行时开销**：所有内存管理逻辑在编译期展开
- **编译期安全检查**：静态确保内存安全，无悬垂指针和内存泄漏
- **自动资源管理**：资源离开作用域时自动清理
- **精确内存控制**：无内存碎片，内存使用效率高
- **统一管理机制**：组机制 + 所有权系统协同工作

**与运行时GC的对比**：

| 特性 | 编译期GC（Chim） | 运行时GC（Java/Go） |
|------|------------------|---------------------|
| 运行时开销 | 零 | 显著 |
| 内存占用 | 精确，无碎片 | 有碎片，需要额外空间 |
| 暂停时间 | 无 | 可能导致STW |
| 性能可预测性 | 高 | 低 |
| 编译期检查 | 完整 | 有限 |

### 4. Agda风格的数学验证

Chim语言支持**Agda风格的依赖类型和数学证明系统**，允许开发者在编译时验证程序的正确性。

#### 依赖类型

```chim
// 依赖类型示例
type Vector(n: Nat) = {
    x: float[n];
}

type Matrix(m: Nat, n: Nat) = {
    data: float[m][n];
}

// 依赖类型函数
fn dot_product(m: Nat, n: Nat)(v1: Vector[n], v2: Vector[n]) -> float {
    let mut sum: float = 0.0;
    for i in 0..n {
        sum = sum + v1.x[i] * v2.x[i];
    }
    return sum;
}
```

#### 证明系统

```chim
// 定理
theorem add_commutes(x: Nat, y: Nat) : Nat
  = proof x + y == y + x
    refl(nat)

// 引理
lemma zero_add(x: Nat) : Nat
  = proof 0 + x == x
    refl(nat)

// 归纳
induction n: Nat
  base_case: proof P(0)
    refl(nat)
  inductive_step: proof P(n) -> P(n + 1)
    refl(nat)
```

#### 类型类和实例

```chim
// 类型类定义
class Eq(A: Type) {
    eq: A -> A -> Prop
}

// 实例
instance Eq(Nat) {
    eq = \x y => x == y
}

// 约束
where x: Nat, y: Nat
  x + y == y + x
  refl(nat)
```

### 5. TileLang高性能计算

Chim语言集成了TileLang的核心特性，允许开发者编写高性能的GPU/CPU内核，特别适合AI模型和科学计算场景。

#### 分块（Tile）

分块是TileLang的核心技术，通过将数据划分为适当大小的块，优化内存访问和计算调度：

```chim
// 矩阵分块示例
fn high_perf_matmul(a: &[float], b: &[float], c: &mut [float], M: int, N: int, K: int) {
    // 使用16x16x16的分块大小
    for m in tile(0..M, 16):
        for n in tile(0..N, 16):
            for k in tile(0..K, 16):
                // 块内计算
                for i in 0..16:
                    for j in 0..16:
                        c[(m+i)*N + (n+j)] += a[(m+i)*K + (k+l)] * b[(k+l)*N + (n+j)];
}
```

#### AI专用算子

```chim
// FlashAttention实现示例
@kernel
fn flash_attention(q: &[float], k: &[float], v: &[float], output: &mut [float], 
                   batch_size: int, seq_len: int, head_dim: int) {
    // 获取线程索引
    let batch = block_idx().x;
    let head = block_idx().y;
    
    // 计算注意力分数
    let scores = matmul(q.slice(batch, head), k.slice(batch, head).transpose();
    
    // 应用softmax
    let attention = softmax(scores / sqrt(head_dim as float));
    
    // 计算输出
    output.slice(batch, head) = matmul(attention, v.slice(batch, head));
}
```

### 6. 异步编程支持

Chim语言提供了强大的异步编程支持，允许开发者编写高性能的并发应用程序。支持两种主要的异步运行时模型：**Tokio（Readiness-based）** 和 **Compio（Completion-based）**，为不同的应用场景提供最优的异步编程体验。

#### Future Trait和核心原语

```chim
// Future表示一个可能还未完成的值
trait Future[T] {
    fn poll(&mut self, context: &mut Context) -> Poll[T];
}

// 潜询结果
enum Poll[T] {
    Ready(T),      // 任务完成
    Pending        // 任务未完成，等待下次轮询
}
```

#### 异步原语

```chim
// 异步睡眠
fn sleep(duration: int) -> SleepFuture

// 异步通道
struct AsyncChannel[T] {
    channel: Channel[T];
}

fn send_async[T](channel: &AsyncChannel[T], value: T) -> SendFuture[T]
fn receive_async[T](channel: &AsyncChannel[T]) -> ReceiveFuture[T]

// 异步互斥锁
struct AsyncMutex[T] {
    mutex: Mutex[T],
    waiters: List[AsyncWaiter]
}

fn lock_async[T](mutex: &AsyncMutex[T]) -> MutexLockFuture[T]
```

#### Tokio vs Compio

| 特性 | Tokio（Readiness-based） | Compio（Completion-based） |
|------|-------------------------|---------------------------|
| 适用场景 | 通用IO密集型任务 | 高性能网络服务 |
| 性能 | 良好 | 更优（io_uring） |
| 平台支持 | 跨平台 | Linux优先 |
| 复杂度 | 中等 | 较高 |

### 7. x86_64代码生成

Chim编译器包含完整的x86_64代码生成器，可以将Chim代码编译为高效的x86_64机器码。

#### 代码生成架构

```chim
// x86_64代码生成器架构
struct X86CodeGenerator {
    // 寄存器分配器
    reg_allocator: RegisterAllocator,
    
    // 栈帧管理
    stack_frame: StackFrame,
    
    // 生成的指令
    instructions: Vec[X86Instruction],
    
    // 标签管理
    labels: HashMap<BlockId, String>,
    
    // 当前函数
    current_function: Option<String>,
}
```

#### 支持的指令

Chim的x86_64代码生成器支持所有x86_64指令集：

- **算术指令**：ADD, SUB, MUL, DIV, INC, DEC
- **逻辑指令**：AND, OR, XOR, NOT
- **移位指令**：SHL, SHR, SAR, ROL, ROR
- **比较指令**：CMP, TEST
- **跳转指令**：JMP, JE, JNE, JG, JL, JGE, JLE
- **移动指令**：MOV, MOVZX, MOVSX
- **栈操作**：PUSH, POP
- **调用指令**：CALL, RET
- **SIMD指令**：ADDPS, MULPS, MOVAPS, etc.
- **原子指令**：LOCK ADD, LOCK XCHG, etc.

#### 代码生成示例

```chim
// Chim代码
fn add(a: int, b: int) -> int {
    return a + b;
}

// 生成的x86_64汇编
add:
    push rbp
    mov rbp, rsp
    mov [rbp-8], rdi    ; a
    mov [rbp-16], rsi   ; b
    mov eax, [rbp-8]
    add eax, [rbp-16]
    pop rbp
    ret
```

---

## 📝 词法规范

### 注释

Chim支持单行注释和多行注释：

```chim
// 这是单行注释
let x = 42 // 行尾注释

/* 这是多行注释
   可以跨越多行
   */
let y = 10 /* 行内多行注释 */
```

### 标识符

标识符规则：
- 必须以字母或下划线开头
- 可以包含字母、数字和下划线
- 区分大小写
- 保留关键字不能用作标识符

### 关键字

#### 英文关键字（基础）
- `fn` - 函数定义
- `let` - 不可变变量声明
- `var` - 可变变量声明
- `if` - 条件语句
- `else` - 条件语句else分支
- `match` - 模式匹配语句
- `while` - 循环语句
- `for` - 循环语句
- `in` - 循环和范围
- `return` - 返回语句
- `struct` - 结构体定义
- `true`/`false` - 布尔值
- `null` - 空值

#### 中文关键字（可选）
- `令` - 不可变变量声明（对应let）
- `设` - 可变变量声明（对应var）
- `分配` - 变量声明
- `如果` - 条件语句
- `否则` - 条件语句else分支
- `匹配` - 模式匹配语句
- `当`/`循环` - 循环语句
- `返回` - 返回语句
- `结构体` - 结构体定义
- `真`/`假` - 布尔值

#### Safe/Unsafe关键字
- `safe` - Safe代码块（编译期资源管理）
- `unsafe` - Unsafe代码块（显式内存管理）
- `group` - 资源组定义
- `init` - 组初始化
- `cleanup` - 组清理
- `defer` - 延迟释放

#### Agda风格关键字
- `proof` - 证明
- `theorem` - 定理
- `lemma` - 引理
- `induction` - 归纳
- `case` - 情况分析
- `refl` - 反射
- `cong` - 同余
- `sym` - 对称
- `trans` - 传递
- `rec` - 递归
- `fix` - 不动点
- `class` - 类型类
- `instance` - 实例
- `where` - 约束
- `eqprop` - 等式命题
- `jmeq` - 判断等价
- `rewrite` - 重写
- `with` - 绑定

#### 显式内存管理关键字
- `alloc` - 内存分配
- `alloc_aligned` - 对齐内存分配
- `free` - 内存释放
- `ptr` - 指针类型
- `ptr_add` - 指针加法
- `ptr_sub` - 指针减法
- `ptr_load` - 指针加载
- `ptr_store` - 指针存储
- `ptr_cast` - 指针转换
- `ptr_offset_of` - 指针偏移量
- `ptr_size_of` - 指针大小
- `align_of` - 对齐量

#### 异步编程关键字
- `async` - 异步函数
- `await` - 等待异步操作
- `spawn` - 创建异步任务
- `future` - Future类型
- `poll` - 潜询
- `ready` - 就绪状态
- `pending` - 等待状态

### 字面量

#### 整数

Chim支持**9种进制系统**的整数字面量，是目前支持进制最多的编程语言之一：

| 进制 | 前缀 | 数字范围 | 示例 | 十进制值 | 应用场景 |
|------|--------|----------|--------|----------|----------|
| 十进制 | 无 | 0-9 | `42` | 42 | 通用计算 |
| 十六进制 | `0x`, `0X` | 0-9, a-f | `0xFF` | 255 | 内存地址、颜色编码 |
| 二进制 | `0b`, `0B` | 0-1 | `0b1010` | 10 | 位操作、布尔逻辑 |
| 八进制 | `0o`, `0O` | 0-7 | `0o755` | 493 | Unix权限 |
| **三进制** | `0t`, `0T` | 0-2 | `0t120` | 15 | 计算机科学、信息论 |
| **平衡三进制** | `0e`, `0E` | -, 0, 1 | `0e1-0` | 2 | 高精度计算、数学研究 |
| **十二进制** | `0d`, `0D` | 0-9, a, b | `0d10` | 12 | 时间（英寸）、商业（打） |
| **二十四进制** | `0h`, `0H` | 0-9, a-n | `0h10` | 24 | 时间（小时） |
| **六十进制** | `0s`, `0S` | 0-9, a-z | `0s10` | 60 | 时间（分秒）、角度 |

**基本整数字面量：**
```chim
42
-10
0
999999
```

**多进制字面量示例：**
```chim
// 十六进制
0xFF        // 255
0XABCD      // 43981

// 二进制
0b1010      // 10
0B11110000  // 240

// 八进制
0o755       // 493
0O7777      // 4095

// 三进制
0t120       // 15
0T210       // 21

// 平衡三进制（接近e进制）
0e1-0       // 2
0E1-1       // 2

// 十二进制
0d10        // 12
0D1a        // 22

// 二十四进制
0h10        // 24
0H1a        // 34

// 六十进制
0s10        // 60
0S1a        // 70
```

**数字分隔符：**
所有进制都支持下划线作为数字分隔符，提高可读性：
```chim
1_000_000    // 十进制：1,000,000
0xFF_FF_FF    // 十六进制：16,777,215
0b1010_0101  // 二进制：165
0d1_000      // 十二进制：1,728
0h1_000      // 二十四进制：576
0s1_000      // 六十进制：36,000
```

#### 浮点数
```chim
3.14
-2.5
0.0
1.0
```

#### 字符串
```chim
"Hello, World!"
"这是字符串"
"包含\n换行符的字符串"
```

#### 布尔值
```chim
true
false
真
假
```

#### 空值
```chim
null
```

#### 原子类型

Chim支持**Rust风格的原子类型**，提供无锁并发编程的基础：

**原子类型：**
```chim
atomic counter: i32 = 0;
atomic flag: bool = false;
atomic value: i64 = 100;
```

**原子操作：**
```chim
// 原子加载
let value = atomic load counter relaxed;

// 原子存储
atomic store counter 10 release;

// 原子加法
atomic fetch_add counter 1 seqcst;

// 原子减法
atomic fetch_sub counter 1 seqcst;

// 原子按位与
atomic fetch_and counter 0xFF seqcst;

// 原子按位或
atomic fetch_or counter 0x01 seqcst;

// 原子按位异或
atomic fetch_xor counter 0x55 seqcst;

// 原子比较交换（CAS）
atomic compare_exchange counter 0 1 acquire release;

// 原子交换
atomic exchange counter 10 seqcst;

// 原子内存屏障
atomic fence seqcst;
```

**内存序：**
Chim支持**13种扩展内存序**，提供比C++更丰富的内存序保证：

| 内存序 | 说明 | 使用场景 |
|--------|------|----------|
| `relaxed` | 最弱保证，仅保证原子性 | 计数器、统计 |
| `consume` | 消费依赖 | 读取共享数据 |
| `acquire` | 获取操作，保证后续读写不会被重排 | 读取共享数据 |
| `release` | 释放操作，保证之前的读写不会被重排 | 写入共享数据 |
| `acqrel` | 获取释放操作，结合acquire和release | 读写共享数据 |
| `seqcst` | 最强保证，顺序一致性 | 全局同步 |
| `happensbefore` | 发生前关系 | 跨线程同步 |
| `volatile` | 易变访问 | 硬件寄存器访问 |
| `memorybarrier` | 内存屏障 | 强制内存访问顺序 |
| `wait` | 等待操作 | 条件变量 |
| `notify` | 通知操作 | 条件变量 |
| `notifyall` | 通知全部操作 | 条件变量 |
| `datadependency` | 数据依赖 | 编译器优化 |

---

## 📖 语法规范

### 程序结构

Chim程序由全局语句和函数定义组成。程序入口点为全局范围内的首句可执行代码，按顺序执行；函数定义用于封装可重用的代码块。

```chim
// 程序入口点 - 全局首句可执行代码
print("Hello, Chim World!")

// 其他函数定义
fn other_function():
    let result = calculate(10, 20)
    return result

fn calculate(a: int, b: int) -> int:
    return a + b
```

### 变量声明

#### 基本语法
```chim
let 变量名: 类型 = 值
let 变量名 = 值  // 类型推断
```

#### 示例
```chim
// 不可变变量声明（let/令）
let age: int = 25
let name = "张三"
令 分数: float = 95.5

// 可变变量声明（var/设）
var counter: int = 0
var message = "Hello"
设 总数 = 100

let is_student: bool = true
let numbers: List[int] = [1, 2, 3, 4, 5]

// 修改变量值
counter = counter + 1
message = message + " World"
```

### 函数定义

#### 基本语法
```chim
fn 函数名(参数列表) -> 返回类型 {
    // 函数体
    return 返回值
}

// 表达式风格（最简洁）
fn add(a: int, b: int) -> int = a + b

// 混合风格（推荐）
fn abs(n: int) -> int = {
    if n >= 0: return n
    return -n
}
```

### 控制流

#### 条件语句
```chim
if 条件:
    # 代码块1
elif 其他条件:
    # 代码块2
else:
    # 默认代码块
```

#### 模式匹配语句
```chim
匹配 值:
    模式1:
        # 处理逻辑1
    模式2:
        # 处理逻辑2
    _:
        # 默认处理
```

#### 循环语句
```chim
// while循环
while 条件:
    # 循环体

// for循环
for 变量 in 范围:
    # 循环体
```

### 显式内存管理（Unsafe）

#### 内存分配和释放
```chim
let buffer = alloc(1024) as *u8;
let aligned_ptr = alloc_align(128, 16) as *int;
let new_ptr = realloc(ptr, old_size, new_size) as *int;

unsafe {
    dealloc(buffer as *void, 1024);
}
```

#### 指针操作
```chim
let ptr: *int = &value;
let value = ptr.*;
ptr.* = 100;
let offset_ptr = ptr + 10;

let array_ptr: *[5]int = &[1, 2, 3, 4, 5];
let element = array_ptr[2];

let slice_ptr: []u8 = &buffer;
```

#### 内存操作原语
```chim
@memset(buffer, 0, 1024);
@memcpy(dest, src, size);
@memmove(dest, src, size);
@memcmp(ptr1, ptr2, size);
```

---

## 🔬 类型系统

### 基础类型

#### 数值类型
- `int` - 整数类型
- `float` - 浮点数类型
- `number` - 通用数值类型（兼容int和float）

#### 字符类型
- `string` - 字符串类型

#### 布尔类型
- `bool` - 布尔类型，值为`true`/`false`或`真`/`假`

#### 其他类型
- `void` - 空类型，用于无返回值的函数
- `null` - 空值类型

### 复合类型

#### List类型
```chim
let numbers: List[int] = [1, 2, 3, 4, 5]
let names: List[string] = ["张三", "李四", "王五"]
let mixed: List = [1, "hello", true]
```

#### 结构体类型
```chim
struct Student {
    name: string
    age: int
    grade: float
}

struct Point {
    x: float
    y: float
}
```

### 泛型类型
```chim
struct Container[T] {
    value: T
}

let int_container = Container[int]{value: 42}
let string_container = Container[string]{value: "hello"}
```

### 依赖类型
```chim
type Vector(n: Nat) = {
    x: float[n];
}

type Matrix(m: Nat, n: Nat) = {
    data: float[m][n];
}

fn dot_product(m: Nat, n: Nat)(v1: Vector[n], v2: Vector[n]) -> float {
    let mut sum: float = 0.0;
    for i in 0..n {
        sum = sum + v1.x[i] * v2.x[i];
    }
    return sum;
}
```

### 证明类型
```chim
type Prop = Type;

theorem add_commutes(x: Nat, y: Nat) : Prop
  = proof x + y == y + x
    refl(nat)
```

### 原子类型
```chim
atomic counter: i32 = 0;
atomic flag: bool = false;
atomic value: i64 = 100;
```

---

## 🛠️ 内置函数

### 输出函数
- `print(value)` - 输出值到控制台
- `println(value)` - 输出值并换行
- `printf(format, args...)` - 格式化输出

### 类型转换函数
- `to_string(value)` - 转换为字符串
- `to_float(value)` - 转换为浮点数
- `to_int(value)` - 转换为整数
- `as_float(value)` - 转换为浮点数
- `as_int(value)` - 转换为整数

### 字符串函数
```chim
let text = "Hello, World!"
let length = len(text)
let is_empty = text.is_empty()
let contains = text.contains("Hello")
let uppercase = text.to_uppercase()
let lowercase = text.to_lowercase()
let stripped = text.strip()
let split = text.split(",")
let replaced = text.replace("Hello", "Hi")
let formatted = format!("Hello, {}", name)
```

### 集合函数
```chim
let numbers = [1, 2, 3, 4, 5]
let size = len(numbers)
let is_empty = numbers.is_empty()
let contains = numbers.contains(3)
numbers.append(6)
numbers.remove(3)
let first = numbers.first()
let last = numbers.last()
let reversed = numbers.reversed()
let sorted = numbers.sorted()
```

### 内存管理函数
```chim
let ptr = alloc(8)
let value = ptr.*
ptr.* = 42
dealloc(ptr, 8)
let boxed = Box::new(42)
let rc = Rc::new(42)
let arc = Arc::new(42)
```

### 异步函数
```chim
let now = current_time()
let formatted_time = format_time(now, "%Y-%m-%d %H:%M:%S")
let sleep(1000)
```

### 文件操作函数
```chim
let file = open("file.txt", "r")
let content = file.read()
file.write("Hello")
file.close()
let exists = file_exists("file.txt")
let size = file_size("file.txt")
let lines = read_lines("file.txt")
```

### 输入函数
```chim
let input = input("Enter something: ")
let line = read_line()
let char = read_char()
```

### 错误处理函数
```chim
try {
    // 可能出错的代码
} catch {
    // 异常处理
}
let result = Ok(42)
let error = Error("Error message")
```

### 线程函数
```chim
let thread = spawn(fn() { /* 线程函数 */ })
thread.join()
let mutex = Mutex::new(0)
let guard = mutex.lock()
```

### 调试函数
```chim
debug_print("Debug message")
assert(condition, "Assert message")
```

### TileLang高性能计算函数

#### 分块操作函数
```chim
let tile = tile(x, (32, 32))
let tiled = tiled_op(x, y, (16, 16), fn(a, b) => a + b)
let vec_tile = tile(vec, 64)
```

#### 内存优化函数
```chim
let merged = merge_access(data, (8, 8))
let cached = cache(data)
let prefetch = prefetch(data)
```

#### 线程控制函数
```chim
sync_threads()
barrier()
let thread_id = thread_idx()
let block_id = block_idx()
let grid_dim = grid_dim()
```

#### 高性能数学函数
```chim
let c = matmul(a, b)
let c = matmul(a, b, (16, 16, 16))
let result = vec_add(a, b)
let result = vec_mul(a, b)
let relu = relu(x)
let gelu = gelu(x)
let softmax = softmax(x)
```

#### AI专用算子
```chim
let attn = flash_attention(q, k, v)
let lin_attn = linear_attention(q, k, v)
let dequant = dequantize(quant_data, scales, zero_points)
let quant = quantize(data, scales, zero_points)
let dequant_gemm = dequant_gemm(a_quant, b_quant, scales_a, scales_b)
```

### 数学函数
```chim
let abs_value = abs(-5)
let max_value = max(10, 20)
let min_value = min(10, 20)
let sqrt_value = sqrt(16)
let pow_value = pow(2, 3)
let ceil_value = ceil(3.14)
let floor_value = floor(3.14)
let round_value = round(3.14)
let sin_value = sin(3.14)
let cos_value = cos(3.14)
let tan_value = tan(3.14)
let random = rand()
```

### 范围函数
```chim
let range_data = range(1, 10)
let range_inclusive = range(1, 10, true)
let step_range = range(0, 10, 2)
```

---

## 📚 标准库

### 核心库

#### StringUtils
```chim
import "stdlib/core/StringUtils.chim"
let result = 字符串工具.连接(["hello", "world"])
```

#### FileIO
```chim
import "stdlib/core/FileIO.chim"
let content = 文件工具.读取("example.txt")
文件工具.写入("output.txt", "Hello, World!")
```

#### MemoryUtils
```chim
import "stdlib/core/MemoryUtils.chim"
let 内存信息 = 内存工具.获取使用情况()
```

### 物理学库

Chim语言提供了丰富的物理学标准库，涵盖多个物理学领域，支持高级物理学计算和模拟。

#### Physics（基础物理学）
```chim
import "stdlib/physics/Physics.chim"

// 物理学常数
let c = Physics::C  // 光速: 299792458 m/s
let G = Physics::G  // 引力常数: 6.67430e-11 m³/(kg·s²)
let h = Physics::H  // 普朗克常数: 6.62607015e-34 J·s
let e = Physics::E  // 元电荷: 1.602176634e-19 C
let k = Physics::K  // 玻尔兹曼常数: 1.380649e-23 J/K
let Na = Physics::NA  // 阿伏伽德罗常数: 6.02214076e23 mol⁻¹
let R = Physics::R  // 气体常数: 8.314462618 J/(mol·K)
```

#### Mechanics（力学）
```chim
import "stdlib/physics/Mechanics.chim" as Mech

// 经典力学
let projectile = Mech::Projectile {
    initial_velocity: 100.0m/s,
    angle: 45.0deg,
    initial_height: 0.0m
}
let trajectory = Mech::calculate_trajectory(projectile, 0.1s)

// 流体力学
let fluid = Mech::Fluid {
    density: 1000.0kg/m³,
    viscosity: 0.001Pa·s
}
let reynolds = Mech::reynolds_number(fluid, 2.0m/s, 0.1m)
```

#### Thermodynamics（热力学）
```chim
import "stdlib/physics/Thermodynamics.chim" as Thermo

// 热力学过程
let ideal_gas = Thermo::IdealGas {
    pressure: 101325.0Pa,
    volume: 0.0224m³,
    temperature: 273.15K,
    moles: 1.0mol
}
let final_state = Thermo::isothermal_expansion(ideal_gas, 0.0488m³)

// 热机效率
let carnot_efficiency = Thermo::carnot_efficiency(373.15K, 273.15K)
```

#### Electromagnetism（电磁学）
```chim
import "stdlib/physics/Electromagnetism.chim" as EM

// 电场和磁场
let capacitor = EM::Capacitor {
    capacitance: 1.0e-6F,
    voltage: 10.0V
}
let energy = EM::capacitor_energy(capacitor)

let inductor = EM::Inductor {
    inductance: 1.0e-3H,
    current: 2.0A
}
let magnetic_energy = EM::inductor_energy(inductor)

// 电磁波
let wavelength = EM::frequency_to_wavelength(5.0e9Hz)
```

#### Optics（光学）
```chim
import "stdlib/physics/Optics.chim" as Opt

// 几何光学
let lens = Opt::Lens {
    focal_length: 0.1m,
    aperture: 0.05m
}
let image = Opt::lens_formula(lens, 0.5m)

// 物理光学
let interference = Opt::double_slit_interference(
    wavelength: 500.0nm,
    slit_distance: 0.1mm,
    screen_distance: 1.0m
)
```

#### Relativity（相对论）
```chim
import "stdlib/physics/Relativity.chim" as Rel

// 狭义相对论
let rocket = Rel::RelativisticObject {
    rest_mass: 1000.0kg,
    velocity: 0.9c
}
let gamma = Rel::lorentz_factor(rocket.velocity)
let total_energy = Rel::total_energy(rocket)

// 广义相对论
let schwarzschild = Rel::schwarzschild_radius(1.989e30kg)
```

#### QuantumMechanics（量子力学）
```chim
import "stdlib/physics/QuantumMechanics.chim" as QM

// 量子力学基础
let photon = QM::Photon {
    wavelength: 500.0nm
}
let photon_energy = QM::photon_energy(photon)

// 氢原子
let hydrogen = QM::HydrogenAtom {
    principal_quantum_number: 1
}
let energy_level = QM::hydrogen_energy(hydrogen)
```

#### Astronomy（天文学）
```chim
import "stdlib/physics/Astronomy.chim" as Astro

// 天文常数
let au = Astro::AU  // 天文单位: 1.495978707e11 m
let ly = Astro::LY  // 光年: 9.4607e15 m
let pc = Astro::PC  // 秒差距: 3.0857e16 m

// 天体力学
let orbit = Astro::orbital_velocity(5.972e24kg, 6.371e6m)
let escape = Astro::escape_velocity(5.972e24kg, 6.371e6m)
```

#### Materials（材料学）
```chim
import "stdlib/physics/Materials.chim" as Mat

// 材料属性
let steel = Mat::Material {
    density: 7850.0kg/m³,
    youngs_modulus: 200.0e9Pa,
    thermal_conductivity: 50.2W/(m·K)
}

// 材料行为
let stress = Mat::calculate_stress(1000.0N, 0.01m²)
let strain = Mat::calculate_strain(0.001m, 1.0m)
```

### 模块系统

#### 导入模块
```chim
import "stdlib/core/StringUtils.chim"
import "mylib/utils.chim"
```

#### 模块别名
```chim
import "stdlib/core/StringUtils.chim" as StrUtil
let result = StrUtil.连接(["hello", "world"])
```

---

## 🔧 编译流程

Chim编译器采用多阶段编译流程，将Chim源代码编译为高效的x86_64机器码。

### 编译阶段

```
Chim源代码
    ↓
词法分析（Lexer）
    ↓
语法分析（Parser）
    ↓
类型检查（Type Checker）
    ↓
中间代码生成（IR Generation）
    ↓
x86_64代码生成（Code Generator）
    ↓
机器码（Executable）
```

### 编译命令

```bash
# 编译Chim程序
chim build main.chim

# 运行Chim程序
chim run main.chim

# 编译为可执行文件
chim build --release main.chim -o main

# 查看生成的汇编代码
chim build --asm main.chim
```

### 编译选项

| 选项 | 说明 |
|------|------|
| `--release` | 优化编译 |
| `--debug` | 调试模式 |
| `--asm` | 输出汇编代码 |
| `--ir` | 输出中间代码 |
| `-o` | 指定输出文件 |

---

## 📋 最佳实践

### 1. 优先使用Safe代码

在大多数情况下，优先使用Safe代码，利用编译期资源管理：

```chim
// 推荐：使用Safe代码
safe {
    group FileHandler {
        var file: File;
        init(path: string) {
            self.file = File::open(path);
        }
        cleanup {
            self.file.close();
        }
    }
}

// 避免：不必要的Unsafe代码
unsafe {
    let file = open_file("data.txt");
    // 容易忘记关闭文件
}
```

### 2. 合理使用Unsafe代码

在需要极致性能或底层访问时使用Unsafe代码：

```chim
// 高性能数据处理
unsafe {
    let buffer = alloc(1024 * 1024) as *u8;
    defer dealloc(buffer as *void, 1024 * 1024);
    
    // 直接内存操作，避免边界检查
    for i in 0..(1024 * 1024) {
        buffer[i] = 0;
    }
}
```

### 3. 使用数学验证提高可靠性

在关键系统中使用数学验证：

```chim
// 验证算法正确性
theorem sort_correct(n: Nat)(arr: [n]int)
  : proof is_sorted(sort(arr))
    refl(array)

// 验证数值计算
theorem add_correct(x: Nat, y: Nat)
  : proof add(x, y) == x + y
    refl(nat)
```

### 4. 利用TileLang优化性能

在计算密集型任务中使用TileLang：

```chim
// 使用分块优化矩阵乘法
fn optimized_matmul(a: &[float], b: &[float], c: &mut [float], M: int, N: int, K: int) {
    for m in tile(0..M, 16):
        for n in tile(0..N, 16):
            for k in tile(0..K, 16):
                // 块内计算
                for i in 0..16:
                    for j in 0..16:
                        c[(m+i)*N + (n+j)] += a[(m+i)*K + (k+l)] * b[(k+l)*N + (n+j)];
}
```

### 5. 选择合适的异步运行时

根据应用场景选择Tokio或Compio：

```chim
// 通用IO密集型任务：使用Tokio
fn web_server() {
    let runtime = TokioRuntime::new();
    runtime.block_on(async {
        // 处理HTTP请求
    });
}

// 高性能网络服务：使用Compio
fn high_perf_server() {
    let runtime = CompioRuntime::new();
    runtime.block_on(async {
        // 处理大量网络连接
    });
}
```

### 6. 使用原子操作实现无锁编程

在高并发场景中使用原子操作：

```chim
// 无锁计数器
atomic counter: i32 = 0;

fn increment() {
    atomic fetch_add counter 1 seqcst;
}
```

### 7. 合理使用依赖类型

在需要类型安全保证时使用依赖类型：

```chim
// 类型安全的向量操作
type Vector(n: Nat) = {
    x: float[n];
}

fn dot_product(n: Nat)(v1: Vector[n], v2: Vector[n]) -> float {
    // 编译期保证向量长度相同
    let mut sum: float = 0.0;
    for i in 0..n {
        sum = sum + v1.x[i] * v2.x[i];
    }
    return sum;
}
```

---

## 📊 总结

Chim语言设计注重代码的可读性和易用性，提供了丰富的语言特性和强大的类型系统。通过本语法规范，开发者可以深入了解Chim语言的各个方面，并能够编写出高效、可维护的Chim程序。

### 语言的设计哲学是"简洁而强大，安全而高效"

Chim语言融合了多种先进语言的设计理念：
- **Zig**的显式内存管理和低开销
- **Agda**的依赖类型和数学证明
- **TileLang**的高性能计算和分块技术
- **Rust**的所有权和生命周期系统
- **June**的编译期资源管理（组机制）
- **Unison**的Effect系统和代数数据类型
- **Koka**的代数效果和类型类系统

通过这种融合，Chim语言为开发者提供了：
1. **双安全模型**：Safe代码的编译期资源管理 + Unsafe代码的显式内存控制
2. **数学验证**：Agda风格的依赖类型和证明系统
3. **高性能计算**：TileLang的GPU/CPU内核和分块技术
4. **类型安全**：强类型系统 + 依赖类型 + 原子操作
5. **并发编程**：Safe/Unsafe分离 + 原子操作 + 异步运行时
6. **编译期GC**：组机制 + 所有权系统，零运行时开销
7. **完整代码生成**：x86_64代码生成器，可编译为原生机器码

Chim语言现在具备开发从系统编程到AI模型训练的各种应用场景的能力！

---

**版权声明**: 本语法规范采用木兰2.0开源许可证，允许自由使用、修改和分发。
