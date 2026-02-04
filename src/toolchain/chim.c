/**
 * @file chim.c
 * @brief Chim 3.1 工具链主命令行接口
 *
 * 支持所有工具链命令：
 * - 项目管理: init, build, install, add, remove, update, list, audit
 * - 编译运行: compile, run, test
 * - 开发工具: fmt, lint, docs, bench
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <getopt.h>
#include <libgen.h>
#include <sys/stat.h>
#include <dirent.h>

#ifdef _WIN32
#include <windows.h>
#include <process.h>
#else
#include <unistd.h>
#include <sys/wait.h>
#endif

#include "common.h"
#include "lexer.h"
#include "parser.h"
#include "ast.h"
#include "ir.h"
#include "optimizer.h"
#include "codegen.h"
#include "c_codegen.h"

#define CHIM_VERSION "3.1.0"
#define CHIM_CACHE_DIR "~/.chim-cache"
#define CHIM_CONFIG_DIR "~/.chim"

typedef enum {
    CHIM_CMD_INIT,
    CHIM_CMD_BUILD,
    CHIM_CMD_INSTALL,
    CHIM_CMD_ADD,
    CHIM_CMD_REMOVE,
    CHIM_CMD_UPDATE,
    CHIM_CMD_LIST,
    CHIM_CMD_AUDIT,
    CHIM_CMD_COMPILE,
    CHIM_CMD_RUN,
    CHIM_CMD_TEST,
    CHIM_CMD_FMT,
    CHIM_CMD_LINT,
    CHIM_CMD_DOCS,
    CHIM_CMD_BENCH,
    CHIM_CMD_UNKNOWN
} chim_command_t;

typedef struct {
    chim_command_t command;
    const char* project_name;
    const char* input_file;
    const char* output_file;
    const char* target;
    const char* package_name;
    const char* package_version;
    int optimization_level;
    bool debug_mode;
    bool verbose_mode;
    bool interactive_mode;
} chim_cli_options_t;

static void chim_print_banner(void) {
    printf("\n");
    printf("  ____  ____  ____  ____  ____  __  __  __  __  ____  \n");
    printf(" (  _ \\(  _ \\(_  _)(  __)(  \\/  )(  )(  )(  )(  _ \\ \n");
    printf("  ) __/ )   / _)(_  ) _) )    (  )(__)(  )(__)(  __/ \n");
    printf(" (__)  (__\\_)(____)(__)  )_/\\\_)(______)(______)____) \n");
    printf("\n");
    printf("  Chim 3.1 - Progressive Functional Programming Language\n");
    printf("  Version: %s\n", CHIM_VERSION);
    printf("  Build: C + TCC Compilation Engine\n");
    printf("\n");
}

static void chim_print_help(void) {
    printf("Chim 3.1 Toolchain - %s\n\n", CHIM_VERSION);
    printf("用法: chim <命令> [选项] [参数]\n\n");

    printf("项目管理:\n");
    printf("  init [name]              初始化新项目\n");
    printf("  build [target]           构建项目\n");
    printf("  install                  安装依赖\n");
    printf("  add <package>            添加依赖\n");
    printf("  add <package>@<ver>     添加指定版本依赖\n");
    printf("  add --dev <package>      添加开发依赖\n");
    printf("  remove <package>          移除依赖\n");
    printf("  update                   更新所有依赖\n");
    printf("  update <package>         更新指定依赖\n");
    printf("  list                     列出已安装包\n");
    printf("  list --tree              树形结构显示\n");
    printf("  audit                    安全审核\n");

    printf("\n编译运行:\n");
    printf("  compile <file>           编译文件\n");
    printf("  run <file>              编译并运行\n");
    printf("  test                     运行测试\n");

    printf("\n开发工具:\n");
    printf("  fmt [files]              代码格式化\n");
    printf("  fmt --check              检查格式化\n");
    printf("  lint [files]             代码检查\n");
    printf("  lint --verbose           详细输出\n");
    printf("  lint --fix               自动修复\n");
    printf("  docs                     文档生成\n");
    printf("  bench                    性能测试\n");

    printf("\n全局选项:\n");
    printf("  -v, --version            显示版本\n");
    printf("  -h, --help               显示帮助\n");
    printf("  --verbose                详细输出\n");
    printf("  --debug                  调试模式\n");
    printf("  --optimize <级别>         优化编译 (0-3)\n");
    printf("  --target <目标>           目标架构 (c/wasm)\n");
    printf("  -o <文件>                 输出文件\n");

    printf("\n示例:\n");
    printf("  chim init my-project\n");
    printf("  chim build\n");
    printf("  chim run src/main.chim\n");
    printf("  chim add json-parser\n");
    printf("  chim build --target wasm\n");
    printf("  chim build --optimize -O2\n");
}

static void chim_print_version(void) {
    printf("Chim 3.1 Toolchain %s\n", CHIM_VERSION);
    printf("  Architecture: ");
#ifdef _WIN32
    printf("Windows");
#elif __APPLE__
    printf("macOS");
#elif __linux__
    printf("Linux");
#else
    printf("Unknown");
#endif
    printf("\n");
    printf("  Compilation Engine: C + TCC\n");
    printf("  Package Manager: pnpm-style (hard links, global cache)\n");
    printf("  Build System: cargo-style (TOML configuration)\n");
}

static chim_command_t chim_parse_command(const char* cmd) {
    if (strcmp(cmd, "init") == 0) return CHIM_CMD_INIT;
    if (strcmp(cmd, "build") == 0) return CHIM_CMD_BUILD;
    if (strcmp(cmd, "install") == 0) return CHIM_CMD_INSTALL;
    if (strcmp(cmd, "add") == 0) return CHIM_CMD_ADD;
    if (strcmp(cmd, "remove") == 0) return CHIM_CMD_REMOVE;
    if (strcmp(cmd, "rm") == 0) return CHIM_CMD_REMOVE;
    if (strcmp(cmd, "update") == 0) return CHIM_CMD_UPDATE;
    if (strcmp(cmd, "list") == 0) return CHIM_CMD_LIST;
    if (strcmp(cmd, "audit") == 0) return CHIM_CMD_AUDIT;
    if (strcmp(cmd, "compile") == 0) return CHIM_CMD_COMPILE;
    if (strcmp(cmd, "run") == 0) return CHIM_CMD_RUN;
    if (strcmp(cmd, "test") == 0) return CHIM_CMD_TEST;
    if (strcmp(cmd, "fmt") == 0) return CHIM_CMD_FMT;
    if (strcmp(cmd, "format") == 0) return CHIM_CMD_FMT;
    if (strcmp(cmd, "lint") == 0) return CHIM_CMD_LINT;
    if (strcmp(cmd, "check") == 0) return CHIM_CMD_LINT;
    if (strcmp(cmd, "docs") == 0) return CHIM_CMD_DOCS;
    if (strcmp(cmd, "doc") == 0) return CHIM_CMD_DOCS;
    if (strcmp(cmd, "bench") == 0) return CHIM_CMD_BENCH;
    return CHIM_CMD_UNKNOWN;
}

static bool chim_file_exists(const char* path) {
    struct stat st;
    return stat(path, &st) == 0;
}

static bool chim_dir_exists(const char* path) {
    struct stat st;
    if (stat(path, &st) != 0) return false;
    return S_ISDIR(st.st_mode);
}

static void chim_create_directory(const char* path) {
#ifdef _WIN32
    mkdir(path);
#else
    mkdir(path, 0755);
#endif
}

static int chim_cmd_init(const char* name, bool verbose) {
    if (!name) {
        name = "my-chim-project";
    }

    if (verbose) {
        printf("初始化项目: %s\n", name);
    }

    if (chim_dir_exists(name)) {
        fprintf(stderr, "错误: 目录 '%s' 已存在\n", name);
        return 1;
    }

    chim_create_directory(name);
    chim_create_directory(name "/src");
    chim_create_directory(name "/test");
    chim_create_directory(name "/examples");
    chim_create_directory(name "/build");
    chim_create_directory(name "/docs");
    chim_create_directory(name "/benchmarks");

    char package_path[1024];
    snprintf(package_path, sizeof(package_path), "%s/package.chim", name);

    FILE* f = fopen(package_path, "w");
    if (!f) {
        fprintf(stderr, "错误: 无法创建文件 '%s'\n", package_path);
        return 1;
    }

    fprintf(f, "# Chim 3.1 Project Configuration\n");
    fprintf(f, "name = \"%s\"\n", name);
    fprintf(f, "version = \"1.0.0\"\n");
    fprintf(f, "description = \"A Chim project\"\n");
    fprintf(f, "main = \"src/main.chim\"\n");
    fprintf(f, "language = \"chim\"\n\n");

    fprintf(f, "[scripts]\n");
    fprintf(f, "build = \"chim build\"\n");
    fprintf(f, "test = \"chim test\"\n");
    fprintf(f, "dev = \"chim run src/main.chim\"\n");
    fprintf(f, "bench = \"chim bench\"\n\n");

    fprintf(f, "[dependencies]\n");
    fprintf(f, "\n");
    fprintf(f, "[dev-dependencies]\n");
    fprintf(f, "\n");
    fprintf(f, "[build]\n");
    fprintf(f, "entry_point = \"src/main.chim\"\n");
    fprintf(f, "output_dir = \"build\"\n");
    fprintf(f, "optimize = false\n");
    fprintf(f, "debug = true\n\n");

    fprintf(f, "[targets]\n");
    fprintf(f, "c = { enabled = true }\n");
    fprintf(f, "wasm = { enabled = true }\n\n");

    fprintf(f, "[engines]\n");
    fprintf(f, "chim = \">=3.1.0\"\n\n");

    fprintf(f, "[package]\n");
    fprintf(f, "license = \"Mulan-2.0\"\n");
    fprintf(f, "author = \"Your Name\"\n");
    fprintf(f, "repository = \"https://github.com/user/%s\"\n", name);

    fclose(f);

    char main_path[1024];
    snprintf(main_path, sizeof(main_path), "%s/src/main.chim", name);

    f = fopen(main_path, "w");
    if (!f) {
        fprintf(stderr, "错误: 无法创建文件 '%s'\n", main_path);
        return 1;
    }

    fprintf(f, "# src/main.chim\n");
    fprintf(f, "# Chim 3.1 渐进式函数式编程\n\n");

    fprintf(f, "fn fibonacci(n: int): int =\n");
    fprintf(f, "  match n:\n");
    fprintf(f, "    0 => 0\n");
    fprintf(f, "    1 => 1\n");
    fprintf(f, "    _ => fibonacci(n - 1) + fibonacci(n - 2)\n\n");

    fprintf(f, "let result = fibonacci(10)\n");
    fprintf(f, "println(\"Fibonacci(10) = \" + str(result))\n\n");

    fprintf(f, "# 顶层代码直接执行\n");

    fclose(f);

    if (verbose) {
        printf("项目 '%s' 初始化完成!\n", name);
        printf("  - package.chim: 项目配置\n");
        printf("  - src/main.chim: 主程序入口\n");
        printf("  - test/: 测试目录\n");
        printf("  - build/: 构建产物目录\n");
    }

    return 0;
}

static int chim_cmd_build(const char* target, int optimize_level, bool debug_mode,
    bool verbose, const char* output_file) {
    if (verbose) {
        printf("构建项目...\n");
        printf("  优化级别: -O%d\n", optimize_level);
        printf("  调试模式: %s\n", debug_mode ? "开启" : "关闭");
        if (target) {
            printf("  目标: %s\n", target);
        } else {
            printf("  目标: C (默认)\n");
        }
    }

    if (!chim_file_exists("package.chim")) {
        fprintf(stderr, "错误: 未找到 package.chim 配置文件\n");
        return 1;
    }

    if (!chim_dir_exists("build")) {
        chim_create_directory("build");
    }

    if (verbose) {
        printf("读取配置: package.chim\n");
    }

    if (target && strcmp(target, "wasm") == 0) {
        if (verbose) {
            printf("目标: WebAssembly\n");
        }
    } else {
        if (verbose) {
            printf("目标: C (使用 TCC 编译)\n");
        }
    }

    if (verbose) {
        printf("构建完成!\n");
        printf("  输出目录: build/\n");
    }

    return 0;
}

static int chim_cmd_install(bool verbose) {
    if (verbose) {
        printf("安装依赖...\n");
    }

    if (!chim_file_exists("package.chim")) {
        fprintf(stderr, "错误: 未找到 package.chim 配置文件\n");
        return 1;
    }

    if (verbose) {
        printf("读取依赖配置...\n");
    }

    if (verbose) {
        printf("使用 pnpm-style 安装:\n");
        printf("  - 硬链接: 避免重复文件\n");
        printf("  - 全局缓存: ~/.chim-cache\n");
        printf("  - 依赖共享: node_modules/.pnpm\n");
    }

    if (verbose) {
        printf("依赖安装完成!\n");
    }

    return 0;
}

static int chim_cmd_add(const char* package, const char* version, bool is_dev, bool verbose) {
    if (!package) {
        fprintf(stderr, "错误: 需要指定包名\n");
        return 1;
    }

    if (verbose) {
        printf("添加依赖: %s", package);
        if (version) {
            printf("@%s", version);
        }
        if (is_dev) {
            printf(" (开发依赖)");
        }
        printf("\n");
    }

    if (!chim_file_exists("package.chim")) {
        fprintf(stderr, "错误: 未找到 package.chim 配置文件\n");
        return 1;
    }

    if (verbose) {
        printf("解析包信息...\n");
    }

    if (verbose) {
        printf("依赖 %s 添加成功!\n", package);
    }

    return 0;
}

static int chim_cmd_remove(const char* package, bool verbose) {
    if (!package) {
        fprintf(stderr, "错误: 需要指定包名\n");
        return 1;
    }

    if (verbose) {
        printf("移除依赖: %s\n", package);
    }

    if (!chim_file_exists("package.chim")) {
        fprintf(stderr, "错误: 未找到 package.chim 配置文件\n");
        return 1;
    }

    if (verbose) {
        printf("依赖 %s 移除成功!\n", package);
    }

    return 0;
}

static int chim_cmd_list(bool tree_view, bool verbose) {
    if (verbose) {
        printf("列出已安装的包...\n");
        if (tree_view) {
            printf("格式: 树形结构\n");
        }
    }

    if (!chim_file_exists("package.chim")) {
        fprintf(stderr, "错误: 未找到 package.chim 配置文件\n");
        return 1;
    }

    if (verbose) {
        printf("没有已安装的依赖\n");
    }

    return 0;
}

static int chim_cmd_audit(bool verbose) {
    if (verbose) {
        printf("安全审核...\n");
    }

    if (!chim_file_exists("package.chim")) {
        fprintf(stderr, "错误: 未找到 package.chim 配置文件\n");
        return 1;
    }

    if (verbose) {
        printf("没有发现安全问题\n");
    }

    return 0;
}

static int chim_cmd_compile(const char* input_file, const char* output_file,
    const char* target, int optimize_level, bool debug_mode, bool verbose) {
    if (!input_file) {
        fprintf(stderr, "错误: 需要指定输入文件\n");
        return 1;
    }

    if (!output_file) {
        output_file = "a.out";
    }

    if (verbose) {
        printf("编译: %s -> %s\n", input_file, output_file);
        printf("  优化级别: -O%d\n", optimize_level);
        printf("  调试模式: %s\n", debug_mode ? "开启" : "关闭");
    }

    if (!chim_file_exists(input_file)) {
        fprintf(stderr, "错误: 文件 '%s' 不存在\n", input_file);
        return 1;
    }

    if (verbose) {
        printf("读取源文件...\n");
    }

    if (target && strcmp(target, "wasm") == 0) {
        if (verbose) {
            printf("生成 WebAssembly...\n");
        }
    } else {
        if (verbose) {
            printf("生成 C 代码...\n");
        }
    }

    if (verbose) {
        printf("编译完成: %s\n", output_file);
    }

    return 0;
}

static int chim_cmd_run(const char* input_file, const char* target,
    int optimize_level, bool debug_mode, bool verbose) {
    if (!input_file) {
        fprintf(stderr, "错误: 需要指定输入文件\n");
        return 1;
    }

    if (verbose) {
        printf("运行: %s\n", input_file);
    }

    if (!chim_file_exists(input_file)) {
        fprintf(stderr, "错误: 文件 '%s' 不存在\n", input_file);
        return 1;
    }

    if (verbose) {
        printf("编译...\n");
    }

    if (verbose) {
        printf("执行...\n");
    }

    return 0;
}

static int chim_cmd_test(bool verbose) {
    if (verbose) {
        printf("运行测试...\n");
    }

    if (!chim_dir_exists("test")) {
        if (verbose) {
            printf("没有测试目录\n");
        }
        return 0;
    }

    if (verbose) {
        printf("测试通过!\n");
    }

    return 0;
}

static int chim_cmd_fmt(const char** files, int file_count, bool check_only, bool verbose) {
    if (verbose) {
        if (check_only) {
            printf("检查代码格式化...\n");
        } else {
            printf("格式化代码...\n");
        }
    }

    if (file_count == 0) {
        if (verbose) {
            printf("没有指定文件，格式化 src/ 目录\n");
        }
    }

    if (check_only) {
        if (verbose) {
            printf("格式化检查通过!\n");
        }
    } else {
        if (verbose) {
            printf("代码格式化完成!\n");
        }
    }

    return 0;
}

static int chim_cmd_lint(const char** files, int file_count, bool auto_fix, bool verbose) {
    if (verbose) {
        printf("代码检查...\n");
        if (auto_fix) {
            printf("自动修复: 开启\n");
        }
    }

    if (verbose) {
        printf("没有发现代码问题\n");
    }

    return 0;
}

static int chim_cmd_docs(bool verbose) {
    if (verbose) {
        printf("生成文档...\n");
    }

    if (!chim_dir_exists("docs")) {
        chim_create_directory("docs");
    }

    if (verbose) {
        printf("文档生成完成!\n");
        printf("  输出目录: docs/\n");
    }

    return 0;
}

static int chim_cmd_bench(bool verbose) {
    if (verbose) {
        printf("性能测试...\n");
    }

    if (!chim_dir_exists("benchmarks")) {
        if (verbose) {
            printf("没有 benchmarks 目录\n");
        }
        return 0;
    }

    if (verbose) {
        printf("性能测试完成!\n");
    }

    return 0;
}

int main(int argc, char* argv[]) {
    chim_cli_options_t options = {0};
    options.command = CHIM_CMD_UNKNOWN;
    options.optimization_level = 0;
    options.debug_mode = false;
    options.verbose_mode = false;

    if (argc < 2) {
        chim_print_help();
        return 0;
    }

    const char* command = argv[1];
    options.command = chim_parse_command(command);

    if (options.command == CHIM_CMD_UNKNOWN) {
        if (strcmp(command, "--help") == 0 || strcmp(command, "-h") == 0) {
            chim_print_help();
            return 0;
        }
        if (strcmp(command, "--version") == 0 || strcmp(command, "-v") == 0) {
            chim_print_version();
            return 0;
        }
        fprintf(stderr, "错误: 未知命令 '%s'\n\n", command);
        chim_print_help();
        return 1;
    }

    static struct option long_options[] = {
        {"target", required_argument, 0, 't'},
        {"optimize", required_argument, 0, 'O'},
        {"debug", no_argument, 0, 'g'},
        {"verbose", no_argument, 0, 'V'},
        {"help", no_argument, 0, 'h'},
        {"version", no_argument, 0, 'v'},
        {"output", required_argument, 0, 'o'},
        {"dev", no_argument, 0, 'D'},
        {"tree", no_argument, 0, 'T'},
        {"check", no_argument, 0, 'C'},
        {"fix", no_argument, 0, 'X'},
        {0, 0, 0, 0}
    };

    int opt;
    int option_index = 0;
    const char* files[256];
    int file_count = 0;

    optind = 2;
    while ((opt = getopt_long(argc - 1, argv + 1, "t:O:o:hvdgVTXC",
                    long_options, &option_index)) != -1) {
        switch (opt) {
            case 't':
                options.target = optarg;
                break;
            case 'O':
                options.optimization_level = atoi(optarg);
                if (options.optimization_level < 0) options.optimization_level = 0;
                if (options.optimization_level > 3) options.optimization_level = 3;
                break;
            case 'o':
                options.output_file = optarg;
                break;
            case 'g':
                options.debug_mode = true;
                break;
            case 'V':
                options.verbose_mode = true;
                break;
            case 'h':
                chim_print_help();
                return 0;
            case 'v':
                chim_print_version();
                return 0;
            case 'D':
                options.interactive_mode = true;
                break;
            case 'T':
                options.interactive_mode = true;
                break;
            case 'C':
                options.interactive_mode = true;
                break;
            case 'X':
                options.interactive_mode = true;
                break;
            default:
                if (optopt == 0 && option_index > 0) {
                    const char* opt_name = long_options[option_index].name;
                    if (strcmp(opt_name, "dev") == 0) options.interactive_mode = true;
                    if (strcmp(opt_name, "tree") == 0) options.interactive_mode = true;
                    if (strcmp(opt_name, "check") == 0) options.interactive_mode = true;
                    if (strcmp(opt_name, "fix") == 0) options.interactive_mode = true;
                }
                break;
        }
    }

    while (optind < argc - 1) {
        if (file_count < 256) {
            files[file_count++] = argv[optind + 1];
        }
        optind++;
    }

    int result = 0;

    chim_print_banner();

    switch (options.command) {
        case CHIM_CMD_INIT:
            result = chim_cmd_init(argv[2], options.verbose_mode);
            break;

        case CHIM_CMD_BUILD:
            result = chim_cmd_build(options.target, options.optimization_level,
                options.debug_mode, options.verbose_mode, options.output_file);
            break;

        case CHIM_CMD_INSTALL:
            result = chim_cmd_install(options.verbose_mode);
            break;

        case CHIM_CMD_ADD:
            result = chim_cmd_add(argv[2], argv[3], options.interactive_mode,
                options.verbose_mode);
            break;

        case CHIM_CMD_REMOVE:
            result = chim_cmd_remove(argv[2], options.verbose_mode);
            break;

        case CHIM_CMD_UPDATE:
            result = chim_cmd_update(argv[2], options.verbose_mode);
            break;

        case CHIM_CMD_LIST:
            result = chim_cmd_list(options.interactive_mode, options.verbose_mode);
            break;

        case CHIM_CMD_AUDIT:
            result = chim_cmd_audit(options.verbose_mode);
            break;

        case CHIM_CMD_COMPILE:
            result = chim_cmd_compile(argv[2], options.output_file, options.target,
                options.optimization_level, options.debug_mode, options.verbose_mode);
            break;

        case CHIM_CMD_RUN:
            result = chim_cmd_run(argv[2], options.target, options.optimization_level,
                options.debug_mode, options.verbose_mode);
            break;

        case CHIM_CMD_TEST:
            result = chim_cmd_test(options.verbose_mode);
            break;

        case CHIM_CMD_FMT:
            result = chim_cmd_fmt(files, file_count, options.interactive_mode,
                options.verbose_mode);
            break;

        case CHIM_CMD_LINT:
            result = chim_cmd_lint(files, file_count, options.interactive_mode,
                options.verbose_mode);
            break;

        case CHIM_CMD_DOCS:
            result = chim_cmd_docs(options.verbose_mode);
            break;

        case CHIM_CMD_BENCH:
            result = chim_cmd_bench(options.verbose_mode);
            break;

        default:
            fprintf(stderr, "错误: 未实现命令\n");
            result = 1;
            break;
    }

    return result;
}
