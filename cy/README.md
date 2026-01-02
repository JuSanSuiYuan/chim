# cy - Chim编程语言包管理器

cy是Chim编程语言的官方包管理器，采用类似pnpm的硬链接机制，实现高效的包存储和管理。

## 核心特性

### 1. Content-Addressable存储
- **内容寻址**：基于文件内容的哈希值进行存储，相同内容只存储一次
- **全局缓存**：所有项目共享同一个包存储，节省磁盘空间
- **快速安装**：通过硬链接创建包副本，无需重复下载和复制

### 2. TOML配置格式
使用`package.chim`作为配置文件（TOML格式），简洁易读：

```toml
[package]
name = "my-project"
version = "0.1.0"
description = "我的Chim项目"
authors = ["Your Name"]
license = "MIT"

[dependencies]
http = "^1.0.0"
json = "~0.5.0"

[dev-dependencies]
test-framework = "^2.0.0"

[scripts]
build = "chim build src/main.chim"
test = "chim test tests/"
```

### 3. 硬链接机制
类似pnpm的设计：
- **全局store**：`~/.cy/store/` 存储所有包的原始内容
- **项目依赖**：`node_modules/` 使用硬链接指向store中的文件
- **节省空间**：多个项目使用相同的包时，磁盘上只有一份副本

### 4. 依赖解析
- **语义化版本**：支持 `^`, `~`, `>=` 等版本范围
- **循环检测**：自动检测并报告循环依赖
- **拓扑排序**：确定正确的安装顺序
- **锁文件**：`cy-lock.toml` 确保依赖版本一致性

## 安装

```bash
# 从源码构建
cd cy
cargo build --release

# 安装到系统
cargo install --path .
```

## 快速开始

### 初始化项目

```bash
# 创建新项目
cy init my-project

# 或在现有目录初始化
cd my-project
cy init
```

这将创建以下目录结构：
```
my-project/
├── package.chim       # 项目配置
├── src/
│   └── main.chim      # 主程序
├── tests/             # 测试文件
└── .gitignore
```

### 管理依赖

```bash
# 添加依赖
cy add http@1.0.0
cy add json           # 使用最新版本

# 添加开发依赖
cy add --dev test-framework

# 安装所有依赖
cy install

# 移除依赖
cy remove http

# 更新依赖
cy update             # 更新所有依赖
cy update http        # 更新指定依赖
```

### 查看信息

```bash
# 列出所有已安装的包
cy list

# 查看包信息
cy info http

# 清理未使用的包
cy prune
```

## 命令详解

### `cy init [name]`
初始化一个新的Chim项目。

**选项**：
- `name` - 项目名称（可选，会交互式询问）

### `cy install`
安装`package.chim`中声明的所有依赖。

**功能**：
- 解析依赖树（包括传递依赖）
- 从注册表下载包
- 使用硬链接创建`node_modules`
- 生成/更新`cy-lock.toml`

### `cy add <package> [--dev]`
添加一个依赖到项目。

**参数**：
- `package` - 包名（格式: `name@version` 或 `name`）
- `--dev, -d` - 作为开发依赖添加

**示例**：
```bash
cy add lodash@4.17.21
cy add express
cy add --dev jest
```

### `cy remove <package>`
从项目中移除一个依赖。

### `cy update [package]`
更新依赖到最新版本。

### `cy list`
列出所有已安装的包及其依赖树。

### `cy info <package>`
显示包的详细信息。

### `cy prune`
清理全局store中未被任何项目使用的包。

## 配置文件

### package.chim
项目配置文件，使用TOML格式：

```toml
[package]
name = "项目名称"
version = "版本号"
description = "项目描述"
authors = ["作者1", "作者2"]
license = "许可证"
homepage = "项目主页"
keywords = ["关键词1", "关键词2"]

[dependencies]
包名 = "版本"

[dev-dependencies]
开发依赖 = "版本"

[scripts]
命令名 = "执行脚本"

[repository]
type = "git"
url = "仓库地址"
```

### cy-lock.toml
锁文件，记录精确的依赖版本和完整性哈希：

```toml
version = "1.0"

[packages.http]
version = "1.0.0"
integrity = "sha256-abc123..."
resolved = "https://registry.chim.dev/http/1.0.0"

[packages.http.dependencies]
utils = "^0.5.0"
```

## 存储结构

cy使用以下目录结构：

```
~/.cy/                      # cy主目录
├── store/                  # 全局包存储
│   ├── ab/                 # 哈希前缀目录
│   │   └── cdef123.../     # 完整哈希目录（包内容）
│   └── ...
└── cache/                  # 下载缓存
```

项目目录：
```
my-project/
├── package.chim            # 项目配置
├── cy-lock.toml            # 锁文件
├── node_modules/           # 依赖（硬链接）
│   ├── http/              # 硬链接到 ~/.cy/store/
│   └── ...
└── src/
    └── main.chim
```

## 性能优势

与传统包管理器对比：

| 特性 | npm/yarn | pnpm | cy |
|------|----------|------|-----|
| 磁盘占用 | 高 | 低 | **极低** |
| 安装速度 | 慢 | 快 | **极快** |
| 硬链接 | ❌ | ✅ | ✅ |
| Content-Addressable | ❌ | ✅ | ✅ |
| 配置格式 | JSON | JSON | **TOML** |

## 技术实现

### Content-Addressable存储
```rust
// 计算包的内容哈希
let hash = sha256(package_content);
let store_path = ~/.cy/store/{hash[0..2]}/{hash[2..]};

// 存储包
copy_to_store(package, store_path);

// 创建硬链接
hard_link(store_path, node_modules/package);
```

### 依赖解析算法
1. 解析`package.chim`的直接依赖
2. 递归获取传递依赖
3. 构建依赖图并检测循环
4. 拓扑排序确定安装顺序
5. 下载并存储到全局store
6. 创建硬链接到项目

## 贡献

欢迎贡献代码、报告问题或提出建议！

## 许可证

本项目采用木兰公共许可证第二版（Mulan PSL v2）开源许可。关于许可证的详细信息，请参阅 LICENSE 文件或访问木兰开源社区获取官方说明。
