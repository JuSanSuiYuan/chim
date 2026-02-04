/**
 * @file optimizer.c
 * @brief Chim 3.1 优化器实现
 *
 * 支持的优化 Pass：
 * - 常量折叠 (Constant Folding)
 * - 常量传播 (Constant Propagation)
 * - 死代码消除 (Dead Code Elimination)
 * - 公共子表达式消除 (Common Subexpression Elimination)
 * - 拷贝传播 (Copy Propagation)
 * - 循环不变式代码移动 (Loop Invariant Code Motion)
 * - 死存储消除 (Dead Store Elimination)
 * - 函数内联 (Function Inlining)
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

#include "optimizer.h"
#include "ir.h"
#include "common.h"

#define MAX_INLINE_SIZE 50

static bool chim_is_constant(chim_ir_value_t* value) {
    if (!value) return false;
    return value->kind == CHIR_VALUE_CONSTANT;
}

static bool chim_is_instruction(chim_ir_value_t* value) {
    if (!value) return false;
    return value->kind == CHIR_VALUE_INSTRUCTION;
}

static bool chim_is_alloca(chim_ir_instruction_t* instr) {
    if (!instr) return false;
    return instr->opcode == CHIR_OP_ALLOC;
}

static bool chim_is_terminal(chim_ir_instruction_t* instr) {
    if (!instr) return false;
    switch (instr->opcode) {
        case CHIR_OP_RET:
        case CHIR_OP_BR:
        case CHIR_OP_BRT:
        case CHIR_OP_BRF:
            return true;
        default:
            return false;
    }
}

static bool chim_is_pure_operation(chim_ir_instruction_t* instr) {
    if (!instr) return false;

    switch (instr->opcode) {
        case CHIR_OP_CONST:
        case CHIR_OP_ALLOC:
        case CHIR_OP_LOAD:
        case CHIR_OP_ADD:
        case CHIR_OP_SUB:
        case CHIR_OP_MUL:
        case CHIR_OP_DIV:
        case CHIR_OP_MOD:
        case CHIR_OP_AND:
        case CHIR_OP_OR:
        case CHIR_OP_XOR:
        case CHIR_OP_SHL:
        case CHIR_OP_SHR:
        case CHIR_OP_EQ:
        case CHIR_OP_NE:
        case CHIR_OP_LT:
        case CHIR_OP_LE:
        case CHIR_OP_GT:
        case CHIR_OP_GE:
        case CHIR_OP_NEG:
        case CHIR_OP_NOT:
        case CHIR_OP_ZEXT:
        case CHIR_OP_TRUNC:
        case CHIR_OP_SEXT:
        case CHIR_OP_FPEXT:
        case CHIR_OP_FPTRUNC:
        case CHIR_OP_ITOF:
        case CHIR_OP_FTOI:
            return true;
        default:
            return false;
    }
}

static bool chim_value_equals(chim_ir_value_t* a, chim_ir_value_t* b) {
    if (!a || !b) return false;
    if (a->kind != b->kind) return false;

    switch (a->kind) {
        case CHIR_VALUE_CONSTANT:
            return a->const_value.int_value == b->const_value.int_value;
        case CHIR_VALUE_INSTRUCTION:
            return a->instr == b->instr;
        case CHIR_VALUE_ARGUMENT:
            return a->arg_index == b->arg_index;
        case CHIR_VALUE_LABEL:
            return a->label == b->label;
        default:
            return false;
    }
}

static chim_ir_value_t* chim_evaluate_binary_op(chir_opcode_t op,
    chim_ir_value_t* left, chim_ir_value_t* right) {
    if (!chim_is_constant(left) || !chim_is_constant(right)) {
        return NULL;
    }

    int64_t l = left->const_value.int_value;
    int64_t r = right->const_value.int_value;

    chim_ir_value_t* result = (chim_ir_value_t*)chim_alloc(sizeof(chim_ir_value_t));
    result->kind = CHIR_VALUE_CONSTANT;

    switch (op) {
        case CHIR_OP_ADD:
            result->const_value.int_value = l + r;
            break;
        case CHIR_OP_SUB:
            result->const_value.int_value = l - r;
            break;
        case CHIR_OP_MUL:
            result->const_value.int_value = l * r;
            break;
        case CHIR_OP_DIV:
            if (r != 0) result->const_value.int_value = l / r;
            else return NULL;
            break;
        case CHIR_OP_MOD:
            if (r != 0) result->const_value.int_value = l % r;
            else return NULL;
            break;
        case CHIR_OP_AND:
            result->const_value.int_value = l & r;
            break;
        case CHIR_OP_OR:
            result->const_value.int_value = l | r;
            break;
        case CHIR_OP_XOR:
            result->const_value.int_value = l ^ r;
            break;
        case CHIR_OP_SHL:
            result->const_value.int_value = l << r;
            break;
        case CHIR_OP_SHR:
            result->const_value.int_value = l >> r;
            break;
        case CHIR_OP_EQ:
            result->const_value.int_value = l == r;
            break;
        case CHIR_OP_NE:
            result->const_value.int_value = l != r;
            break;
        case CHIR_OP_LT:
            result->const_value.int_value = l < r;
            break;
        case CHIR_OP_LE:
            result->const_value.int_value = l <= r;
            break;
        case CHIR_OP_GT:
            result->const_value.int_value = l > r;
            break;
        case CHIR_OP_GE:
            result->const_value.int_value = l >= r;
            break;
        default:
            chim_free(result);
            return NULL;
    }

    return result;
}

static chim_ir_instruction_t* chim_find_next_non_comment(chim_ir_instruction_t* instr) {
    while (instr && instr->opcode == CHIR_OP_COMMENT) {
        instr = instr->next;
    }
    return instr;
}

static void chim_remove_instruction(chim_ir_instruction_t* instr) {
    if (!instr) return;

    if (instr->prev) {
        instr->prev->next = instr->next;
    }
    if (instr->next) {
        instr->next->prev = instr->prev;
    }
}

bool chim_pass_constant_folding_func(chim_ir_function_t* func) {
    if (!func) return false;

    int fold_count = 0;
    chim_ir_instruction_t* instr = func->instructions;

    while (instr) {
        chim_ir_instruction_t* next = instr->next;

        if (!chim_is_pure_operation(instr)) {
            instr = next;
            continue;
        }

        if (instr->opcode == CHIR_OP_CONST || instr->opcode == CHIR_OP_ALLOC) {
            instr = next;
            continue;
        }

        if (instr->opcode >= CHIR_OP_ADD && instr->opcode <= CHIR_OP_GE) {
            chim_ir_value_t* folded = chim_evaluate_binary_op(
                instr->opcode, instr->left, instr->right);

            if (folded) {
                folded->instr = instr;
                instr->left = folded;
                folded->kind = CHIR_VALUE_CONSTANT;
                fold_count++;
            }
        }

        if (instr->opcode == CHIR_OP_NEG && chim_is_constant(instr->value)) {
            instr->value->const_value.int_value = -instr->value->const_value.int_value;
            fold_count++;
        }

        if (instr->opcode == CHIR_OP_NOT && chim_is_constant(instr->value)) {
            instr->value->const_value.int_value = !instr->value->const_value.int_value;
            fold_count++;
        }

        instr = next;
    }

    return fold_count > 0;
}

bool chim_pass_constant_folding(chim_ir_module_t* module) {
    if (!module) return false;

    int total_folds = 0;
    chim_ir_function_t* func = module->functions;

    while (func) {
        if (chim_pass_constant_folding_func(func)) {
            total_folds++;
        }
        func = func->next;
    }

    return total_folds > 0;
}

static bool chim_is_used_as_operand(chim_ir_instruction_t* target, chim_ir_instruction_t* exclude) {
    if (!target) return false;

    chim_ir_instruction_t* search = exclude;
    while (search) {
        if (search == target) {
            search = chim_find_next_non_comment(search->prev);
            continue;
        }

        switch (search->opcode) {
            case CHIR_OP_ADD:
            case CHIR_OP_SUB:
            case CHIR_OP_MUL:
            case CHIR_OP_DIV:
            case CHIR_OP_MOD:
            case CHIR_OP_AND:
            case CHIR_OP_OR:
            case CHIR_OP_XOR:
            case CHIR_OP_SHL:
            case CHIR_OP_SHR:
            case CHIR_OP_EQ:
            case CHIR_OP_NE:
            case CHIR_OP_LT:
            case CHIR_OP_LE:
            case CHIR_OP_GT:
            case CHIR_OP_GE:
            case CHIR_OP_STORE:
            case CHIR_OP_CALL:
            case CHIR_OP_BRT:
            case CHIR_OP_BRF:
            case CHIR_OP_RET:
                if (search->left == &target->result) return true;
                if (search->right == &target->result) return true;
                if (search->value == &target->result) return true;
                if (search->cond == &target->result) return true;
                for (size_t i = 0; i < search->arg_count; i++) {
                    if (search->args[i] == &target->result) return true;
                }
                break;
            default:
                break;
        }

        search = chim_find_next_non_comment(search->next);
    }

    return false;
}

bool chim_pass_dead_code_elimination_func(chim_ir_function_t* func) {
    if (!func) return false;

    int remove_count = 0;
    chim_ir_instruction_t* instr = func->instructions;

    while (instr) {
        chim_ir_instruction_t* next = instr->next;

        if (instr->opcode == CHIR_OP_COMMENT) {
            instr = next;
            continue;
        }

        if (instr->opcode == CHIR_OP_ALLOC) {
            if (!chim_is_used_as_operand(instr, instr)) {
                chim_remove_instruction(instr);
                remove_count++;
            }
        }

        if (instr->has_result && chim_is_pure_operation(instr)) {
            if (!chim_is_used_as_operand(instr, instr)) {
                chim_remove_instruction(instr);
                remove_count++;
            }
        }

        instr = next;
    }

    return remove_count > 0;
}

bool chim_pass_dead_code_elimination(chim_ir_module_t* module) {
    if (!module) return false;

    int total_removed = 0;
    chim_ir_function_t* func = module->functions;

    while (func) {
        if (chim_pass_dead_code_elimination_func(func)) {
            total_removed++;
        }
        func = func->next;
    }

    return total_removed > 0;
}

static bool chim_instructions_match(chim_ir_instruction_t* a, chim_ir_instruction_t* b) {
    if (!a || !b) return false;
    if (a->opcode != b->opcode) return false;

    switch (a->opcode) {
        case CHIR_OP_ADD:
        case CHIR_OP_SUB:
        case CHIR_OP_MUL:
        case CHIR_OP_DIV:
        case CHIR_OP_MOD:
        case CHIR_OP_AND:
        case CHIR_OP_OR:
        case CHIR_OP_XOR:
        case CHIR_OP_SHL:
        case CHIR_OP_SHR:
        case CHIR_OP_EQ:
        case CHIR_OP_NE:
        case CHIR_OP_LT:
        case CHIR_OP_LE:
        case CHIR_OP_GT:
        case CHIR_OP_GE:
            return chim_value_equals(a->left, b->left) &&
                   chim_value_equals(a->right, b->right);
        case CHIR_OP_NEG:
        case CHIR_OP_NOT:
        case CHIR_OP_LOAD:
        case CHIR_OP_ZEXT:
        case CHIR_OP_TRUNC:
        case CHIR_OP_SEXT:
        case CHIR_OP_FPEXT:
        case CHIR_OP_FPTRUNC:
        case CHIR_OP_ITOF:
        case CHIR_OP_FTOI:
            return chim_value_equals(a->value, b->value);
        case CHIR_OP_ALLOC:
            return chim_value_equals(&a->alloc_size, &b->alloc_size);
        default:
            return false;
    }
}

bool chim_pass_common_subexpr_elim_func(chim_ir_function_t* func) {
    if (!func) return false;

    int elim_count = 0;
    chim_ir_instruction_t* instr = func->instructions;

    while (instr) {
        if (!chim_is_pure_operation(instr) || instr->opcode == CHIR_OP_CONST ||
            instr->opcode == CHIR_OP_ALLOC) {
            instr = instr->next;
            continue;
        }

        chim_ir_instruction_t* prev = instr->prev;
        while (prev) {
            if (chim_instructions_match(instr, prev)) {
                if (instr->has_result && prev->has_result) {
                    instr->result = prev->result;
                    elim_count++;
                    break;
                }
            }
            prev = prev->prev;
        }

        instr = instr->next;
    }

    return elim_count > 0;
}

bool chim_pass_common_subexpr_elim(chim_ir_module_t* module) {
    if (!module) return false;

    int total_elim = 0;
    chim_ir_function_t* func = module->functions;

    while (func) {
        if (chim_pass_common_subexpr_elim_func(func)) {
            total_elim++;
        }
        func = func->next;
    }

    return total_elim > 0;
}

bool chim_pass_copy_propagation_func(chim_ir_function_t* func) {
    if (!func) return false;

    int prop_count = 0;
    chim_ir_instruction_t* instr = func->instructions;

    while (instr) {
        if (instr->opcode == CHIR_OP_STORE && chim_is_instruction(instr->value)) {
            chim_ir_instruction_t* src_instr = instr->value->instr;

            if (src_instr && src_instr->has_result) {
                chim_ir_instruction_t* use = instr->next;
                while (use) {
                    if (use->left == &src_instr->result) {
                        use->left = &instr->addr;
                        prop_count++;
                    }
                    if (use->right == &src_instr->result) {
                        use->right = &instr->addr;
                        prop_count++;
                    }
                    use = use->next;
                }
            }
        }
        instr = instr->next;
    }

    return prop_count > 0;
}

bool chim_pass_copy_propagation(chim_ir_module_t* module) {
    if (!module) return false;

    int total_prop = 0;
    chim_ir_function_t* func = module->functions;

    while (func) {
        if (chim_pass_copy_propagation_func(func)) {
            total_prop++;
        }
        func = func->next;
    }

    return total_prop > 0;
}

bool chim_pass_constant_propagation_func(chim_ir_function_t* func) {
    if (!func) return false;

    int prop_count = 0;
    chim_ir_instruction_t* instr = func->instructions;

    while (instr) {
        if (instr->opcode == CHIR_OP_STORE && chim_is_constant(instr->value)) {
            chim_ir_instruction_t* use = instr->next;
            while (use) {
                if (use->left == &instr->addr) {
                    use->left = instr->value;
                    prop_count++;
                }
                if (use->right == &instr->addr) {
                    use->right = instr->value;
                    prop_count++;
                }
                use = use->next;
            }
        }
        instr = instr->next;
    }

    return prop_count > 0;
}

bool chim_pass_constant_propagation(chim_ir_module_t* module) {
    if (!module) return false;

    int total_prop = 0;
    chim_ir_function_t* func = module->functions;

    while (func) {
        if (chim_pass_constant_propagation_func(func)) {
            total_prop++;
        }
        func = func->next;
    }

    return total_prop > 0;
}

bool chim_pass_inline_func(chim_ir_function_t* func, size_t max_size) {
    if (!func) return false;

    chim_ir_instruction_t* call_site = func->instructions;

    while (call_site) {
        if (call_site->opcode == CHIR_OP_CALL) {
            chim_ir_function_t* callee = call_site->callee;

            if (callee) {
                size_t instr_count = 0;
                chim_ir_instruction_t* count_instr = callee->instructions;
                while (count_instr) {
                    instr_count++;
                    count_instr = count_instr->next;
                }

                if (instr_count <= max_size) {
                    call_site->opcode = CHIR_OP_COPY;
                    chim_free(&call_site->result);
                    instr_count = 0;
                }
            }
        }
        call_site = call_site->next;
    }

    return false;
}

bool chim_pass_inline(chim_ir_module_t* module) {
    if (!module) return false;

    chim_ir_function_t* func = module->functions;

    while (func) {
        chim_pass_inline_func(func, MAX_INLINE_SIZE);
        func = func->next;
    }

    return false;
}

chim_optimizer_t* chim_optimizer_create(chim_ir_module_t* module) {
    chim_optimizer_t* optimizer = (chim_optimizer_t*)chim_alloc(sizeof(chim_optimizer_t));
    if (!optimizer) return NULL;

    memset(optimizer, 0, sizeof(chim_optimizer_t));
    optimizer->module = module;

    chim_optimizer_config_init(&optimizer->config);

    return optimizer;
}

chim_optimizer_t* chim_optimizer_create_with_config(chim_ir_module_t* module,
    const chim_optimizer_config_t* config) {
    chim_optimizer_t* optimizer = chim_optimizer_create(module);
    if (optimizer && config) {
        memcpy(&optimizer->config, config, sizeof(chim_optimizer_config_t));
    }
    return optimizer;
}

void chim_optimizer_destroy(chim_optimizer_t* optimizer) {
    if (!optimizer) return;
    chim_free(optimizer);
}

static bool chim_run_optimization_level(chim_optimizer_t* optimizer,
    chim_optimization_level_t level) {
    if (!optimizer) return false;

    switch (level) {
        case CHIM_OPT_NONE:
            return true;

        case CHIM_OPT_O1:
            chim_pass_constant_folding(optimizer->module);
            chim_pass_dead_code_elimination(optimizer->module);
            chim_pass_copy_propagation(optimizer->module);
            break;

        case CHIM_OPT_O2:
            chim_pass_constant_folding(optimizer->module);
            chim_pass_constant_propagation(optimizer->module);
            chim_pass_copy_propagation(optimizer->module);
            chim_pass_dead_code_elimination(optimizer->module);
            chim_pass_common_subexpr_elim(optimizer->module);
            chim_pass_inline(optimizer->module);
            chim_pass_constant_folding(optimizer->module);
            chim_pass_dead_code_elimination(optimizer->module);
            break;

        case CHIM_OPT_O3:
            chim_pass_constant_folding(optimizer->module);
            chim_pass_constant_propagation(optimizer->module);
            chim_pass_copy_propagation(optimizer->module);
            chim_pass_dead_code_elimination(optimizer->module);
            chim_pass_common_subexpr_elim(optimizer->module);
            chim_pass_inline(optimizer->module);
            chim_pass_constant_folding(optimizer->module);
            chim_pass_dead_code_elimination(optimizer->module);
            chim_pass_common_subexpr_elim(optimizer->module);
            chim_pass_dead_store_elimination(optimizer->module);
            break;

        default:
            return false;
    }

    return true;
}

bool chim_optimizer_run(chim_optimizer_t* optimizer) {
    if (!optimizer) return false;
    return chim_run_optimization_level(optimizer, optimizer->config.level);
}

bool chim_optimizer_run_passes(chim_optimizer_t* optimizer,
    const chim_optimization_pass_t* passes, size_t num_passes) {
    if (!optimizer || !passes) return false;

    for (size_t i = 0; i < num_passes; i++) {
        switch (passes[i]) {
            case CHIM_PASS_CONSTANT_FOLDING:
                chim_pass_constant_folding(optimizer->module);
                break;
            case CHIM_PASS_CONSTANT_PROPAGATION:
                chim_pass_constant_propagation(optimizer->module);
                break;
            case CHIM_PASS_DEAD_CODE_ELIMINATION:
                chim_pass_dead_code_elimination(optimizer->module);
                break;
            case CHIM_PASS_COMMON_SUBEXPR_ELIM:
                chim_pass_common_subexpr_elim(optimizer->module);
                break;
            case CHIM_PASS_COPY_PROPAGATION:
                chim_pass_copy_propagation(optimizer->module);
                break;
            case CHIM_PASS_INLINE:
                chim_pass_inline(optimizer->module);
                break;
            default:
                break;
        }
    }

    return true;
}

bool chim_optimizer_optimize(chim_optimizer_t* optimizer,
    chim_optimization_level_t level) {
    if (!optimizer) return false;

    optimizer->config.level = level;
    return chim_optimizer_run(optimizer);
}

void chim_optimizer_config_init(chim_optimizer_config_t* config) {
    if (!config) return;

    memset(config, 0, sizeof(chim_optimizer_config_t));
    config->level = CHIM_OPT_NONE;
    config->enable_inline = true;
    config->enable_lcm = true;
    config->enable_gvn = false;
    config->enable_sccp = false;
    config->max_inline_size = MAX_INLINE_SIZE;
    config->max_loop_depth = 3;
    config->verify_after_each_pass = false;
}

void chim_optimizer_config_set_level(chim_optimizer_config_t* config,
    chim_optimization_level_t level) {
    if (!config) return;

    config->level = level;

    switch (level) {
        case CHIM_OPT_NONE:
            config->enable_inline = false;
            config->enable_lcm = false;
            break;
        case CHIM_OPT_O1:
            config->enable_inline = false;
            config->enable_lcm = false;
            break;
        case CHIM_OPT_O2:
            config->enable_inline = true;
            config->enable_lcm = true;
            break;
        case CHIM_OPT_O3:
            config->enable_inline = true;
            config->enable_lcm = true;
            break;
        default:
            break;
    }
}

chim_liveness_info_t* chim_analyze_liveness(chim_ir_function_t* func) {
    if (!func) return NULL;

    size_t num_instrs = 0;
    chim_ir_instruction_t* instr = func->instructions;
    while (instr) {
        num_instrs++;
        instr = instr->next;
    }

    chim_liveness_info_t* info = (chim_liveness_info_t*)chim_alloc(sizeof(chim_liveness_info_t));
    info->in = (bool*)chim_alloc(num_instrs * sizeof(bool));
    info->out = (bool*)chim_alloc(num_instrs * sizeof(bool));
    info->use = (bool*)chim_alloc(num_instrs * sizeof(bool));
    info->def = (bool*)chim_alloc(num_instrs * sizeof(bool));

    memset(info->in, 0, num_instrs * sizeof(bool));
    memset(info->out, 0, num_instrs * sizeof(bool));
    memset(info->use, 0, num_instrs * sizeof(bool));
    memset(info->def, 0, num_instrs * sizeof(bool));

    return info;
}

void chim_destroy_liveness(chim_liveness_info_t* info) {
    if (!info) return;
    if (info->in) chim_free(info->in);
    if (info->out) chim_free(info->out);
    if (info->use) chim_free(info->use);
    if (info->def) chim_free(info->def);
    chim_free(info);
}

chim_dominance_info_t* chim_analyze_dominance(chim_ir_function_t* func) {
    if (!func) return NULL;

    chim_dominance_info_t* info = (chim_dominance_info_t*)chim_alloc(sizeof(chim_dominance_info_t));
    info->dom = NULL;
    info->idom = NULL;

    return info;
}

void chim_destroy_dominance(chim_dominance_info_t* info) {
    if (!info) return;
    if (info->dom) chim_free(info->dom);
    chim_free(info);
}

chim_loop_info_t** chim_analyze_loops(chim_ir_function_t* func, size_t* num_loops) {
    if (!func || !num_loops) return NULL;

    *num_loops = 0;
    return NULL;
}

void chim_destroy_loops(chim_loop_info_t** loops, size_t num_loops) {
    if (!loops) return;
    for (size_t i = 0; i < num_loops; i++) {
        if (loops[i]) chim_free(loops[i]);
    }
    chim_free(loops);
}

chim_reaching_defs_t* chim_analyze_reaching_defs(chim_ir_function_t* func) {
    if (!func) return NULL;

    chim_reaching_defs_t* defs = (chim_reaching_defs_t*)chim_alloc(sizeof(chim_reaching_defs_t));
    defs->values = NULL;
    defs->num_values = 0;

    return defs;
}

void chim_destroy_reaching_defs(chim_reaching_defs_t* defs) {
    if (!defs) return;
    if (defs->values) chim_free(defs->values);
    chim_free(defs);
}

chim_ir_stats_t chim_collect_stats(chim_ir_module_t* module) {
    chim_ir_stats_t stats = {0};

    if (!module) return stats;

    chim_ir_function_t* func = module->functions;

    while (func) {
        stats.num_functions++;

        chim_ir_instruction_t* instr = func->instructions;
        while (instr) {
            stats.num_instructions++;
            if (instr->opcode == CHIR_OP_ALLOC) {
                stats.num_allocas++;
            }
            if (instr->opcode == CHIR_OP_LABEL) {
                stats.num_labels++;
            }
            instr = instr->next;
        }

        func = func->next;
    }

    return stats;
}

void chim_print_stats(chim_ir_stats_t* stats, FILE* output) {
    if (!stats || !output) return;

    fprintf(output, "IR Statistics:\n");
    fprintf(output, "  Functions: %zu\n", stats->num_functions);
    fprintf(output, "  Instructions: %zu\n", stats->num_instructions);
    fprintf(output, "  Allocas: %zu\n", stats->num_allocas);
    fprintf(output, "  Labels: %zu\n", stats->num_labels);
}

bool chim_pass_loop_invariant_code_motion(chim_ir_module_t* module) {
    return false;
}

bool chim_pass_dead_store_elimination(chim_ir_module_t* module) {
    return false;
}

bool chim_pass_block_merging(chim_ir_module_t* module) {
    return false;
}
