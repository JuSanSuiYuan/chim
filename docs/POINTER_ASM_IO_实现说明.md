# Chim 语言新功能实现说明

## 概述

本文档说明 Chim 语言实现的三个重要底层特性,参照 Zig 语言的实现方式:

1. **原始指针 + 解引用**
2. **内联汇编**  
3. **端口 I/O**

## 1. 原始指针与解引用

### 语法设计

#### 取地址操作符: `&`
```chim
设 x := 42
设 px := &x  # px 是指向 x 的指针
```

#### 指针解引用 (支持两种风格)
```chim
# Zig 风格 (后缀): ptr.*
设 value1 := px.*

# C 风格 (前缀): *ptr  
设 value2 := *px
```

#### 指针类型声明
```chim
fn 函数(ptr: *整数) -> 整数:
    返回 ptr.*
```

### 实现细节

#### 词法分析器 (lexer.py)
- 添加 `TokenType.STAR` (用于 * 符号)
- 添加 `TokenType.AMPERSAND` (用于 & 符号)
- 添加 `TokenType.AT` (保留用于类型转换)

#### 解析器 (parser_.py)
新增 AST 节点:
- `PointerType`: 指针类型 `*T`
- `AddressOf`: 取地址表达式 `&variable`
- `Dereference`: 解引用表达式 `variable.*` 或 `*variable`

解析逻辑:
- `parse_unary()`: 处理前缀运算符 `&` 和 `*`
- `parse_postfix()`: 处理后缀解引用 `.*`
- `parse_type()`: 处理指针类型声明

#### 代码生成器 (zig_codegen.py)
Chim → Zig 转换:
- `&variable` → `&variable` (保持一致)
- `variable.*` 或 `*variable` → `variable.*` (统一为 Zig 风格)

## 2. 内联汇编

### 语法设计

```chim
# 基本语法
asm volatile ("汇编代码")

# 示例
fn 空操作():
    asm volatile ("nop")
    返回
```

完整格式(支持但简化处理):
```chim
asm volatile ("code" : outputs : inputs : clobbers)
```

### 实现细节

#### 词法分析器
- `TokenType.ASM`: `汇编` 或 `asm` 关键字
- `TokenType.VOLATILE`: `volatile` 关键字

#### 解析器
新增 AST 节点:
- `InlineAssembly`: 内联汇编节点
  - `code`: 汇编代码字符串
  - `is_volatile`: 是否标记为 volatile
  - `outputs/inputs/clobbers`: 约束列表(当前简化处理)

解析方法:
- `parse_inline_assembly()`: 解析内联汇编语句

#### 代码生成器
生成 Zig 内联汇编:
```zig
_ = asm volatile (
    \\ nop
);
```

## 3. 端口 I/O

### 语法设计

提供六个内置端口 I/O 函数:

```chim
# 读端口
value := inb(port)   # 读 u8
value := inw(port)   # 读 u16
value := ind(port)   # 读 u32

# 写端口
outb(port, value)    # 写 u8
outw(port, value)    # 写 u16
outd(port, value)    # 写 u32
```

### 实现细节

#### 词法分析器
将这些函数名保留为普通标识符,在代码生成阶段处理。

#### 代码生成器
在 `visit_call_expression()` 中检测端口 I/O 函数,生成对应的内联汇编:

##### inb (读字节)
```zig
asm volatile ("inb %[port], %[result]" 
    : [result] "={al}" (-> u8) 
    : [port] "{dx}" (port))
```

##### outb (写字节)
```zig
asm volatile ("outb %[value], %[port]" 
    : 
    : [port] "{dx}" (port), [value] "{al}" (value))
```

类似地实现 inw/outw (使用 ax), ind/outd (使用 eax)。

## 4. 附加功能: 十六进制字面量

### 语法
```chim
令 COM1_PORT := 0x3F8
令 mask := 0xFF
```

### 实现
在 `tokenize_number()` 中检测 `0x` 或 `0X` 前缀,解析为十六进制整数。

## 综合示例

```chim
system

# 串口输出函数
fn 串口写字节(data: 整数):
    令 COM1_PORT := 0x3F8
    
    # 等待串口就绪
    设 status := inb(COM1_PORT + 5)
    
    # 发送数据
    outb(COM1_PORT, data)
    返回

fn 测试指针():
    设 x := 42
    设 px := &x
    设 value := px.*
    串口写字节(value)
    返回

fn 测试汇编():
    asm volatile ("cli")  # 关中断
    asm volatile ("sti")  # 开中断
    返回
```

生成的 Zig 代码:
```zig
const std = @import("std");

fn 串口写字节(data: i64) void {
    const COM1_PORT = 1016;
    var status = asm volatile ("inb %[port], %[result]" 
        : [result] "={al}" (-> u8) 
        : [port] "{dx}" ((COM1_PORT + 5)));
    asm volatile ("outb %[value], %[port]" 
        : 
        : [port] "{dx}" (COM1_PORT), [value] "{al}" (data));
    return;
}

fn 测试指针() void {
    var x = 42;
    var px = &x;
    var value = px.*;
    串口写字节(value);
    return;
}

fn 测试汇编() void {
    _ = asm volatile (\\ cli);
    _ = asm volatile (\\ sti);
    return;
}
```

## 设计考虑

### 1. 安全性
- 这些功能仅在 `system` 方言中可用
- 需要显式声明 `system` 才能使用

### 2. 兼容性
- 指针解引用支持两种风格 (Zig 和 C)
- 最终都转换为 Zig 风格的 `.*`

### 3. 简化处理
- 内联汇编约束当前简化处理
- 端口 I/O 直接生成对应的汇编代码
- 未来可扩展更复杂的约束支持

## 测试验证

运行测试:
```bash
cd d:\PROJECT\Chim\build
python test_final_all.py
```

测试涵盖:
- ✓ 指针取地址和解引用
- ✓ 内联汇编 (volatile)
- ✓ 端口 I/O (inb/outb 等)
- ✓ 十六进制字面量
- ✓ 综合应用(串口通信)

## 文件修改清单

### 1. compiler/lexer.py
- 添加指针相关 token 类型
- 支持十六进制数字解析
- 添加端口 I/O 函数名识别

### 2. compiler/parser_.py  
- 添加指针、内联汇编相关 AST 节点
- 实现指针解引用解析 (两种风格)
- 实现内联汇编解析
- 修正运算符优先级处理

### 3. compiler/zig_codegen.py
- 实现指针和解引用的代码生成
- 实现内联汇编的代码生成
- 实现端口 I/O 的内联汇编生成

### 4. build/test_*.py
- test_final_all.py: 综合测试
- test_simple_pointer.py: 指针测试
- test_asm_only.py: 汇编测试

## 后续扩展建议

1. **类型系统增强**
   - 支持可空指针: `?*T`
   - 支持切片类型: `[]T`

2. **内联汇编增强**
   - 完整的约束支持
   - 多行汇编代码
   - AT&T 和 Intel 语法选择

3. **端口 I/O 增强**
   - 内存映射 I/O (MMIO)
   - DMA 支持
   - 中断处理

4. **更多底层特性**
   - 原子操作
   - 内存屏障
   - 位操作指令

## 总结

本次实现为 Chim 语言添加了三个重要的底层编程特性,使其能够:

1. 进行内存级别的精确控制(指针)
2. 直接嵌入汇编代码(内联汇编)
3. 与硬件直接交互(端口 I/O)

这些特性使 Chim 语言适合用于:
- 操作系统内核开发
- 设备驱动开发  
- 嵌入式系统编程
- 底层硬件控制

实现参照 Zig 语言的设计,保持了语法的简洁性和表达力,同时生成高效的 Zig 代码。
