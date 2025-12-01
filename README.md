# Chim 编程语言（中文名：启语）

**简介**
- 强类型静态编译语言
- 缩进式代码块、唯一分支 `match`（Swift风格增强）、统一括号构造与类型推断。
- 支持中文/英文关键字双轨，提供 `兼容/compat` 方言以平滑迁移。
- 并发原语采用通道与 `<-` 方向运算符，避免与赋值 `=` 冲突。

**核心特性**
- **Swift风格match语句**：
  - 元组模式匹配：`案例 (0, 0) ->` 
  - 范围匹配：`案例 90..100 ->`
  - Guard条件：`案例 x 当 x > 0 ->`
  - 模式变量绑定：自动解构并绑定
- **缩进式语法**：使用冒号 `:` + 缩进，无花括号
- **中英双轨**：`匹配`/`match`、`案例`/`case`、`当`/`when` 等

**工具链**
- `chim mod` 包管理器
  - 使用 `package.chim` 管理依赖，DSL 为 TOML 风格。
  - 采用类似 `pnpm` 的硬链接仓库：全局包存储复用，项目内通过硬链接落盘，减少重复体积。
  - 常用命令：
    - `chim mod init` 在当前目录初始化 `package.chim`
    - `chim mod add <name>@<version>` 添加依赖
    - `chim mod remove <name>` 移除依赖
    - `chim mod install` 按锁文件安装依赖
    - `chim mod update` 更新依赖到最新兼容版本
    - `chim mod link` 在本地包之间建立开发链接
    - `chim mod publish` 发布当前包
  - 文件与目录：
    - `package.chim` 项目清单（TOML 风格 DSL）
    - `chim.lock` 依赖锁定文件
    - `vendor/` 项目依赖目录（指向全局仓库的硬链接）

- `chim cl` 编译器
  - 常用命令：
    - `chim cl build` 构建项目
    - `chim cl run` 构建并运行入口文件
    - `chim cl test` 运行测试
  - 常用参数：
    - `--dialect compat|safe|system` 选择方言
    - `--out <path>` 指定输出路径
    - `--release` 以发布模式构建

- `chim fmt` 代码格式化工具
  - 统一缩进、构造/调用消歧、关键字风格与空白规范。
  - 常用命令：
    - `chim fmt .` 格式化当前项目
    - `chim fmt --check .` 仅检查不修改
    - `chim fmt --fix .` 自动修复可格式问题
  - 默认分支规范化：将 `默认 ->` 统一规范为 `_ ->`

**Rust + Zig 实现路线**
- Rust 前端（`Chim/compiler-rs`）：词法/语法/语义与IR构建，导出C ABI（`chim_frontend`）
- Zig 后端（`Chim/runtime-zig`）：代码生成与运行时，链接前端C头（`Chim/include/chim_ffi.h`）
- 自举策略：先以Rust+Zig替换编译器的性能关键路径，再用Stage1编译用Chim编写的编译器子集，实现自托管

**构建步骤（Windows）**
- 前端（Rust）：
  - 进入 `Chim/compiler-rs`，运行 `cargo build --release`
  - 生成库 `chim_frontend.dll`（FFI由编译器自动探测）
- 后端（Zig，占位）：
  - 进入 `Chim/runtime-zig`，运行 `zig build`
  - 生成库 `chim_backend`（当前未接入，后续用于目标生成）
- 编译示例：
  - `python Chim/compiler/main.py Chim/examples/hello.chim`
  - 输出至 `Chim/build/*.py`，可用 `python` 运行验证

**快速开始**
- 初始化项目：
  - `chim mod init`
- 创建入口文件 `src/main.chim`：
  ```chim
  兼容
  
  fn 主():
      令 x := 1
      匹配 x:
          案例 1 -> 输出("ok")
          _ -> 输出("no")
  ```
  
  **Match语句示例**：
  ```chim
  # 元组模式匹配
  匹配 点:
      案例 (0, 0) -> 输出("原点")
      案例 (x, 0) -> 输出("在X轴")
      _ -> 输出("其他")
  
  # 范围匹配
  匹配 分数:
      案例 90..100 -> 输出("优秀")
      案例 60..90 -> 输出("及格")
      _ -> 输出("不及格")
  
  # Guard条件
  匹配 值:
      案例 x 当 x > 0 -> 输出("正数")
      案例 x 当 x < 0 -> 输出("负数")
      _ -> 输出("零")
  ```
- 构建并运行：
  - `chim cl run`

**示例 package.chim（TOML 风格）**
```toml
[package]
name = "hello-chim"
version = "0.1.0"
dialect = "compat"

[dependencies]
nova-std = "0.1.*"
```

**并发与通道示例**
```chim
令 ch := chan<整数>(16)
ch <- 1
设 x := <- ch
```

**约定与风格**
- 类型名使用首字母大写；函数与变量小写。
- 构造使用命名参数；函数调用使用位置实参。
- 显式书写 `令/设` 指示可变性；禁止行尾分号。
- match语句使用 `匹配 值:` 而非 `匹配 值 |`（更符合缩进式语法）。
- 使用 `chim fmt` 保持风格一致。

**当前实现状态**
- ✅ Rust前端编译通过（词法、语法、IR框架）
- ✅ Python编译器正常工作（基础功能）
- ✅ Match语句Swift风格增强（元组、范围、guard）
- ⏳ 完整的语义分析（待实现）
- ⏳ Zig后端集成（待实现）
**方言扩展（预览）**
- `动态/dynamic`：更宽松的类型与多分派语义；用于原型与REPL；语法保持一致
- `DSL/toml`：将文件作为TOML领域配置（如 `package.chim`），交由DSL解析器处理
