# Chim 3.1 语法规范

> 简洁、无歧义、易实现

---

## 设计哲学

Chim 3.1 追求**极简主义**：

| 决策 | 原因 |
|------|------|
| **函数调用用括号** | `add(a, b)` 消除歧义 |
| **类型注解位置固定** | `name: Type`，规则简单 |
| **控制流用冒号+缩进** | 清晰的代码块边界 |
| **表达式导向** | 一切皆有返回值 |
| **顶层代码执行** | 无需 main() |

---

## 语言特性

| 特性 | 值 | 说明 |
|------|-----|------|
| 类型系统 | 静态类型 + 推断 | 编译期类型检查 |
| 数据可变性 | 默认不可变 | `let` 不可变，`var` 可变 |
| 代码风格 | 括号调用 + 缩进 | 明确无歧义 |
| 程序入口 | 顶层代码 | 无需 main() 函数 |
| 命名风格 | 蛇形命名 | `my_variable` |
| 类型命名 | 驼峰命名 | `MyType` |

---

## 语法规则

### 1. 变量声明

```chim
# 不可变变量（推荐）
let x: int = 42
let name = "Chim"           # 类型推断

# 可变变量（需要时）
var counter: int = 0

# 常量（编译期确定）
const pi = 3.14159
const max_size = 1000
```

### 2. 函数定义

```chim
# 单行函数
fn add(a: int, b: int): int = a + b
fn double(x: int): int = x * 2

# 多行函数（= 分隔签名和函数体）
fn fib(n: int): int =
  if n < 2:
    n
  else:
    fib(n - 1) + fib(n - 2)

# 无返回值
fn greet(name: string): void =
  println("Hello, " + name)

# 函数调用（必须用括号）
let x = add(1, 2)           # => 3
let y = double(fib(5))       # 嵌套调用
```

### 3. 控制流

```chim
# if 表达式
let max = if a > b: a else: b

# if 语句
if x > 0:
  println("positive")
elif x < 0:
  println("negative")
else:
  println("zero")

# for 循环
var sum = 0
for i in 1..10:
  sum = sum + i

# while 循环
var i = 0
while i < 10:
  i = i + 1

# match 表达式
let result = match x:
  0 => "zero"
  1 => "one"
  _ => "other"

# match 语句
match status:
  0 => println("zero")
  1 => println("one")
  _ => println("other")
```

### 4. 数据结构

```chim
# 列表
let numbers = [1, 2, 3, 4, 5]
let first = numbers[0]       # 索引访问

# 元组
let point = (10, 20)
let x = point[0]
let y = point[1]

# Record（键值对）
let user = (name: "Alice", age: 30)
let name = user.name
```

---

## 内置类型

```
int       # 整数
float     # 浮点数
bool      # 布尔值
string    # 字符串
char      # 字符
void      # 空类型

[T]       # 列表

(T1, T2)  # 元组
```

---

## 运算符

| 类别 | 运算符 |
|------|--------|
| 算术 | `+`, `-`, `*`, `/`, `%` |
| 比较 | `==`, `!=`, `<`, `<=`, `>`, `>=` |
| 位运算 | `&`, `|`, `^`, `<<`, `>>` |
| 逻辑 | `!`, `&&`, `||` |
| 范围 | `..` |

---

## 关键字

| 关键字 | 用途 |
|--------|------|
| `fn` | 函数定义 |
| `let` | 不可变变量声明 |
| `var` | 可变变量声明 |
| `const` | 常量声明 |
| `if`, `elif`, `else` | 条件分支 |
| `for`, `in` | 循环 |
| `while` | 条件循环 |
| `match`, `=>` | 模式匹配 |
| `return` | 返回值 |
| `break` | 跳出循环 |
| `continue` | 继续循环 |

---

## 缩进规则

- 使用空格缩进（推荐 2 个空格）
- 冒号 `:` 标记代码块开始
- 缩进增加表示进入新的代码块
- 缩进减少表示退出代码块

```chim
if condition:        # 冒号标记开始
  statement1        # 缩进的语句
  statement2
elif condition2:    # elif 也是缩进风格
  statement3
else:               # else 同样
  statement4
```

---

## 完整示例

### 示例 1：斐波那契数列

```chim
# 文件: fib.chim

fn fib(n: int): int =
  if n < 2:
    n
  else:
    fib(n - 1) + fib(n - 2)

fn sum_to(n: int): int =
  var sum = 0
  for i in 1..n:
    sum = sum + i
  sum

# 顶层代码按顺序执行
let f = fib(10)              # 55
let s = sum_to(10)           # 55
println("Fib(10) = " + str(f))
println("Sum(10) = " + str(s))
```

### 示例 2：素数判断

```chim
fn is_prime(n: int): bool =
  if n < 2:
    false
  elif n == 2:
    true
  else:
    var is_prime = true
    var i = 2
    while i * i <= n:
      if n % i == 0:
        is_prime = false
        break
      i = i + 1
    is_prime

# 测试
let test = 17
if is_prime(test):
  println(str(test) + " 是素数")
else:
  println(str(test) + " 不是素数")
```

### 示例 3：列表处理

```chim
fn map(f: fn(int) -> int, list: [int]): [int] =
  var result = []
  for item in list:
    result = result + [f(item)]
  result

fn filter(p: fn(int) -> bool, list: [int]): [int] =
  var result = []
  for item in list:
    if p(item):
      result = result + [item]
  result

# 使用
let numbers = [1, 2, 3, 4, 5]
let doubled = map(fn(x): int = x * 2, numbers)   # [2, 4, 6, 8, 10]
let evens = filter(fn(x): bool = x % 2 == 0, numbers)  # [2, 4]
```

---

## 语法规则（BNF）

```
Program        ::= TopLevel*

TopLevel       ::= FunctionDef | Statement

FunctionDef    ::= "fn" Identifier Params ReturnType? "=" Expr
Params         ::= "(" (Param ("," Param)*)? ")"
Param          ::= Identifier ":" Type
ReturnType     ::= ":" Type

Expr           ::= IfExpr | MatchExpr | ForExpr | WhileExpr
                  | BinaryOp | UnaryOp | Identifier "(" Args? ")"
                  | Literal | "(" Expr ")" | Identifier

IfExpr         ::= "if" Expr ":" Expr ("elif" Expr ":" Expr)* ("else" ":" Expr)?

MatchExpr      ::= "match" Expr ":" MatchArm ("," MatchArm)*
MatchArm       ::= Pattern "=>" Expr
Pattern        ::= Literal | Identifier | "_" | "_" Identifier

ForExpr        ::= "for" Identifier "in" Expr ":" Expr
WhileExpr      ::= "while" Expr ":" Expr

BinaryOp       ::= Expr Op Expr
UnaryOp        ::= Op Expr

Args           ::= Expr ("," Expr)*

Statement      ::= LetStmt | VarStmt | ExprStmt
LetStmt        ::= "let" Pattern ("=" Expr)? ("," LetStmt)*
VarStmt        ::= "var" Identifier (":" Type)? "=" Expr

Type           ::= Identifier | "[" Type "]"
Literal        ::= Number | String | Bool | Nil
Bool           ::= "true" | "false"
Nil            ::= "nil"

Op             ::= "+" | "-" | "*" | "/" | "%"
                  | "==" | "!=" | "<" | "<=" | ">" | ">="
                  | "&&" | "||" | "!" | "&" | "|" | "^" | "<<" | ">>"
```

---

## 实现路线图

### Phase 1：最小可行语言（2-3 周）

```
✓ 整型、字符串、布尔类型
✓ 变量声明 let/var
✓ 算术/比较运算符
✓ if/elif/else
✓ fn 函数定义 + 调用
✓ 编译到 JavaScript
```

### Phase 2：循环和匹配（1-2 周）

```
✓ for/in 循环
✓ while 循环
✓ match 表达式
✓ 列表字面量 [1, 2, 3]
✓ 索引访问 arr[i]
```

### Phase 3：类型系统（2-3 周）

```
✓ 类型推断
✓ 基础类型检查
✓ 函数类型注解
✓ 错误报告
```

---

## 与其他版本对比

| 特性 | Chim 1.0 | Chim 3.0 | **Chim 3.1** |
|------|----------|----------|---------------|
| 函数调用 | `add(a, b)` | `add a b` | `add(a, b)` ✓ |
| 函数定义 | `fn add(...)` | `add ... =` | `fn add(...) =` ✓ |
| 类型注解 | `: Type` | `: Type` | `: Type` ✓ |
| 控制流 | 缩进+冒号 | 缩进+换行 | 缩进+冒号 ✓ |
| match | `=>` | `->` | `=>` ✓ |
| 实现难度 | 中等 | 很高 | **低** ✓ |
| 歧义 | 极少 | 很多 | **无** ✓ |

---

## 代码量估算

| 组件 | 行数估计 |
|------|---------|
| 词法分析器 | ~300 行 |
| 递归下降解析器 | ~800 行 |
| 类型检查器 | ~600 行 |
| 代码生成器 (JS) | ~400 行 |
| REPL/CLI | ~200 行 |
| **总计** | **~2300 行** |

---

## 为什么容易实现？

| 问题 | 传统方案 | Chim 3.1 方案 |
|------|----------|---------------|
| 函数调用歧义 | 需要上下文分析 | 括号消除歧义 |
| 代码块边界 | 缩进+换行 | 冒号+缩进，边界清晰 |
| 类型注解位置 | 位置灵活 | 参数列表后，规则固定 |
| 运算符优先级 | 查表复杂 | 标准优先级，易处理 |
| 嵌套表达式 | 括号嵌套 | 括号明确作用域 |

---

## 版本信息

| 属性 | 值 |
|------|-----|
| 规范版本 | 3.1.0 |
| 设计哲学 | **简洁、无歧义、易实现** |
| 目标 | 快速原型开发 + 学习编译器原理 |

---

**核心原则**：让简单的事情保持简单，让复杂的事情成为可能。
