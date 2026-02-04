/**
 * @file optimizer.h
 * @brief Chim 3.1 优化器头文件
 */

#ifndef CHIM_OPTIMIZER_H
#define CHIM_OPTIMIZER_H

#include "ir.h"
#include "common.h"

/* 优化 Pass 类型 */
typedef enum {
    CHIM_PASS_NONE = 0,
    CHIM_PASS_CONSTANT_FOLDING,
    CHIM_PASS_CONSTANT_PROPAGATION,
    CHIM_PASS_DEAD_CODE_ELIMINATION,
    CHIM_PASS_COMMON_SUBEXPR_ELIM,
    CHIM_PASS_COPY_PROPAGATION,
    CHIM_PASS_LOOP_INVARIANT_CODE_MOTION,
    CHIM_PASS_DEAD_STORE_ELIMINATION,
    CHIM_PASS_INLINE,
    CHIM_PASS_DCE,          /* 死代码消除 */
    CHIM_PASS_CSE,          /* 公共子表达式消除 */
    CHIM_PASS_REASSOC,      /* 重结合 */
    CHIM_PASS_GVN,          /* 全局值编号 */
    CHIM_PASS_SCCP,         /* 稀疏条件常量传播 */
    CHIM_PASS_ALL,
} chim_optimization_pass_t;

/* 优化器配置 */
typedef struct {
    chim_optimization_level_t level;
    bool enable_inline;
    bool enable_lcm;           /* 循环不变式移动 */
    bool enable_gvn;
    bool enable_sccp;
    size_t max_inline_size;
    size_t max_loop_depth;
    bool verify_after_each_pass;
} chim_optimizer_config_t;

/* 优化器上下文 */
typedef struct {
    chim_ir_module_t* module;
    chim_optimizer_config_t config;
    int pass_count;
    int total_instrs_removed;
    int total_instrs_modified;
} chim_optimizer_t;

/* 优化器创建 */
chim_optimizer_t* chim_optimizer_create(chim_ir_module_t* module);
chim_optimizer_t* chim_optimizer_create_with_config(chim_ir_module_t* module,
    const chim_optimizer_config_t* config);
void chim_optimizer_destroy(chim_optimizer_t* optimizer);

/* 运行优化 */
bool chim_optimizer_run(chim_optimizer_t* optimizer);
bool chim_optimizer_run_passes(chim_optimizer_t* optimizer,
    const chim_optimization_pass_t* passes, size_t num_passes);

/* 按级别优化 */
bool chim_optimizer_optimize(chim_optimizer_t* optimizer, chim_optimization_level_t level);

/* 单独 Pass 执行 */

/* 常量折叠 */
bool chim_pass_constant_folding(chim_ir_module_t* module);
bool chim_pass_constant_folding_func(chim_ir_function_t* func);

/* 常量传播 */
bool chim_pass_constant_propagation(chim_ir_module_t* module);
bool chim_pass_constant_propagation_func(chim_ir_function_t* func);

/* 死代码消除 */
bool chim_pass_dead_code_elimination(chim_ir_module_t* module);
bool chim_pass_dead_code_elimination_func(chim_ir_function_t* func);

/* 公共子表达式消除 */
bool chim_pass_common_subexpr_elim(chim_ir_module_t* module);
bool chim_pass_common_subexpr_elim_func(chim_ir_function_t* func);

/* 拷贝传播 */
bool chim_pass_copy_propagation(chim_ir_module_t* module);
bool chim_pass_copy_propagation_func(chim_ir_function_t* func);

/* 循环不变式代码移动 */
bool chim_pass_loop_invariant_code_motion(chim_ir_module_t* module);

/* 死存储消除 */
bool chim_pass_dead_store_elimination(chim_ir_module_t* module);

/* 函数内联 */
bool chim_pass_inline(chim_ir_module_t* module);
bool chim_pass_inline_func(chim_ir_function_t* func);

/* 块合并 */
bool chim_pass_block_merging(chim_ir_module_t* module);

/* 活跃性分析 */
typedef struct {
    bool* in;
    bool* out;
    bool* use;
    bool* def;
} chim_liveness_info_t;

chim_liveness_info_t* chim_analyze_liveness(chim_ir_function_t* func);
void chim_destroy_liveness(chim_liveness_info_t* info);

/* 支配分析 */
typedef struct {
    chim_ir_basic_block_t** dom;
    chim_ir_basic_block_t* idom;
} chim_dominance_info_t;

chim_dominance_info_t* chim_analyze_dominance(chim_ir_function_t* func);
void chim_destroy_dominance(chim_dominance_info_t* info);

/* 循环分析 */
typedef struct {
    chim_ir_basic_block_t* header;
    chim_ir_basic_block_t* latch;
    chim_ir_basic_block_t** blocks;
    size_t num_blocks;
    int nesting_depth;
} chim_loop_info_t;

chim_loop_info_t** chim_analyze_loops(chim_ir_function_t* func, size_t* num_loops);
void chim_destroy_loops(chim_loop_info_t** loops, size_t num_loops);

/* 数据流分析 */
typedef struct {
    chim_ir_value_t** values;
    size_t num_values;
} chim_reaching_defs_t;

chim_reaching_defs_t* chim_analyze_reaching_defs(chim_ir_function_t* func);
void chim_destroy_reaching_defs(chim_reaching_defs_t* defs);

/* 统计信息 */
typedef struct {
    size_t num_functions;
    size_t num_basic_blocks;
    size_t num_instructions;
    size_t num_temp_values;
    size_t num_labels;
    size_t num_globals;
    size_t num_allocas;
} chim_ir_stats_t;

chim_ir_stats_t chim_collect_stats(chim_ir_module_t* module);
void chim_print_stats(chim_ir_stats_t* stats, FILE* output);

/* 优化配置工具 */
void chim_optimizer_config_init(chim_optimizer_config_t* config);
void chim_optimizer_config_set_level(chim_optimizer_config_t* config, chim_optimization_level_t level);

#endif /* CHIM_OPTIMIZER_H */
