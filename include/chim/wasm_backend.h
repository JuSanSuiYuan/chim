/**
 * @file wasm_backend.h
 * @brief Chim 3.1 WebAssembly 后端头文件
 */

#ifndef CHIM_WASM_BACKEND_H
#define CHIM_WASM_BACKEND_H

#include "codegen.h"
#include "ir.h"

#define CHIM_WASM_MAGIC_NUMBER 0x6D736100
#define CHIM_WASM_VERSION 0x01

typedef enum {
    CHIM_WASM_TYPE_FUNC = 0x60,
    CHIM_WASM_TYPE_FUNC_REF = 0x70,
    CHIM_WASM_TYPE_REF = 0x6B,
} chim_wasm_type_t;

typedef enum {
    CHIM_WASM_SECTION_TYPE = 1,
    CHIM_WASM_SECTION_IMPORT = 2,
    CHIM_WASM_SECTION_FUNCTION = 3,
    CHIM_WASM_SECTION_TABLE = 4,
    CHIM_WASM_SECTION_MEMORY = 5,
    CHIM_WASM_SECTION_GLOBAL = 6,
    CHIM_WASM_SECTION_EXPORT = 7,
    CHIM_WASM_SECTION_START = 8,
    CHIM_WASM_SECTION_ELEMENT = 9,
    CHIM_WASM_SECTION_CODE = 10,
    CHIM_WASM_SECTION_DATA = 11,
} chim_wasm_section_type_t;

typedef enum {
    CHIM_WASM_OPCODE_UNREACHABLE = 0x00,
    CHIM_WASM_OPCODE_NOP = 0x01,
    CHIM_WASM_OPCODE_BLOCK = 0x02,
    CHIM_WASM_OPCODE_LOOP = 0x03,
    CHIM_WASM_OPCODE_IF = 0x04,
    CHIM_WASM_OPCODE_ELSE = 0x05,
    CHIM_WASM_OPCODE_END = 0x0B,
    CHIM_WASM_OPCODE_BR = 0x0C,
    CHIM_WASM_OPCODE_BR_IF = 0x0D,
    CHIM_WASM_OPCODE_BR_TABLE = 0x0E,
    CHIM_WASM_OPCODE_RETURN = 0x0F,
    CHIM_WASM_OPCODE_CALL = 0x10,
    CHIM_WASM_OPCODE_CALL_INDIRECT = 0x11,
    CHIM_WASM_OPCODE_DROP = 0x1A,
    CHIM_WASM_OPCODE_SELECT = 0x1B,
    CHIM_WASM_OPCODE_LOCAL_GET = 0x20,
    CHIM_WASM_OPCODE_LOCAL_SET = 0x21,
    CHIM_WASM_OPCODE_LOCAL_TEE = 0x22,
    CHIM_WASM_OPCODE_GLOBAL_GET = 0x23,
    CHIM_WASM_OPCODE_GLOBAL_SET = 0x24,
    CHIM_WASM_OPCODE_I32_LOAD = 0x28,
    CHIM_WASM_OPCODE_I64_LOAD = 0x29,
    CHIM_WASM_OPCODE_F32_LOAD = 0x2A,
    CHIM_WASM_OPCODE_F64_LOAD = 0x2B,
    CHIM_WASM_OPCODE_I32_STORE = 0x36,
    CHIM_WASM_OPCODE_I64_STORE = 0x37,
    CHIM_WASM_OPCODE_F32_STORE = 0x38,
    CHIM_WASM_OPCODE_F64_STORE = 0x39,
    CHIM_WASM_OPCODE_I32_CONST = 0x41,
    CHIM_WASM_OPCODE_I64_CONST = 0x42,
    CHIM_WASM_OPCODE_F32_CONST = 0x43,
    CHIM_WASM_OPCODE_F64_CONST = 0x44,
    CHIM_WASM_OPCODE_I32_EQZ = 0x45,
    CHIM_WASM_OPCODE_I32_EQ = 0x46,
    CHIM_WASM_OPCODE_I32_NE = 0x47,
    CHIM_WASM_OPCODE_I32_LT_S = 0x48,
    CHIM_WASM_OPCODE_I32_LT_U = 0x49,
    CHIM_WASM_OPCODE_I32_GT_S = 0x4A,
    CHIM_WASM_OPCODE_I32_GT_U = 0x4B,
    CHIM_WASM_OPCODE_I32_LE_S = 0x4C,
    CHIM_WASM_OPCODE_I32_LE_U = 0x4D,
    CHIM_WASM_OPCODE_I32_GE_S = 0x4E,
    CHIM_WASM_OPCODE_I32_GE_U = 0x4F,
    CHIM_WASM_OPCODE_I64_EQZ = 0x50,
    CHIM_WASM_OPCODE_I64_EQ = 0x51,
    CHIM_WASM_OPCODE_I64_NE = 0x52,
    CHIM_WASM_OPCODE_I64_LT_S = 0x53,
    CHIM_WASM_OPCODE_I64_LT_U = 0x54,
    CHIM_WASM_OPCODE_I64_GT_S = 0x55,
    CHIM_WASM_OPCODE_I64_GT_U = 0x56,
    CHIM_WASM_OPCODE_I64_LE_S = 0x57,
    CHIM_WASM_OPCODE_I64_LE_U = 0x58,
    CHIM_WASM_OPCODE_I64_GE_S = 0x59,
    CHIM_WASM_OPCODE_I64_GE_U = 0x5A,
    CHIM_WASM_OPCODE_F32_EQ = 0x5B,
    CHIM_WASM_OPCODE_F32_NE = 0x5C,
    CHIM_WASM_OPCODE_F32_LT = 0x5D,
    CHIM_WASM_OPCODE_F32_GT = 0x5E,
    CHIM_WASM_OPCODE_F32_LE = 0x5F,
    CHIM_WASM_OPCODE_F32_GE = 0x60,
    CHIM_WASM_OPCODE_F64_EQ = 0x61,
    CHIM_WASM_OPCODE_F64_NE = 0x62,
    CHIM_WASM_OPCODE_F64_LT = 0x63,
    CHIM_WASM_OPCODE_F64_GT = 0x64,
    CHIM_WASM_OPCODE_F64_LE = 0x65,
    CHIM_WASM_OPCODE_F64_GE = 0x66,
    CHIM_WASM_OPCODE_I32_ADD = 0x6A,
    CHIM_WASM_OPCODE_I32_SUB = 0x6B,
    CHIM_WASM_OPCODE_I32_MUL = 0x6C,
    CHIM_WASM_OPCODE_I32_DIV_S = 0x6D,
    CHIM_WASM_OPCODE_I32_DIV_U = 0x6E,
    CHIM_WASM_OPCODE_I32_REM_S = 0x6F,
    CHIM_WASM_OPCODE_I32_REM_U = 0x70,
    CHIM_WASM_OPCODE_I32_AND = 0x71,
    CHIM_WASM_OPCODE_I32_OR = 0x72,
    CHIM_WASM_OPCODE_I32_XOR = 0x73,
    CHIM_WASM_OPCODE_I32_SHL = 0x74,
    CHIM_WASM_OPCODE_I32_SHR_S = 0x75,
    CHIM_WASM_OPCODE_I32_SHR_U = 0x76,
    CHIM_WASM_OPCODE_I32_ROTL = 0x77,
    CHIM_WASM_OPCODE_I32_ROTR = 0x78,
    CHIM_WASM_OPCODE_I64_ADD = 0x7C,
    CHIM_WASM_OPCODE_I64_SUB = 0x7D,
    CHIM_WASM_OPCODE_I64_MUL = 0x7E,
    CHIM_WASM_OPCODE_I64_DIV_S = 0x7F,
    CHIM_WASM_OPCODE_I64_DIV_U = 0x80,
    CHIM_WASM_OPCODE_I64_REM_S = 0x81,
    CHIM_WASM_OPCODE_I64_REM_U = 0x82,
    CHIM_WASM_OPCODE_I64_AND = 0x83,
    CHIM_WASM_OPCODE_I64_OR = 0x84,
    CHIM_WASM_OPCODE_I64_XOR = 0x85,
    CHIM_WASM_OPCODE_I64_SHL = 0x86,
    CHIM_WASM_OPCODE_I64_SHR_S = 0x87,
    CHIM_WASM_OPCODE_I64_SHR_U = 0x88,
    CHIM_WASM_OPCODE_I64_ROTL = 0x89,
    CHIM_WASM_OPCODE_I64_ROTR = 0x8A,
    CHIM_WASM_OPCODE_F32_ABS = 0x8B,
    CHIM_WASM_OPCODE_F32_NEG = 0x8C,
    CHIM_WASM_OPCODE_F32_CEIL = 0x8D,
    CHIM_WASM_OPCODE_F32_FLOOR = 0x8E,
    CHIM_WASM_OPCODE_F32_TRUNC = 0x8F,
    CHIM_WASM_OPCODE_F32_NEAREST = 0x90,
    CHIM_WASM_OPCODE_F32_SQRT = 0x91,
    CHIM_WASM_OPCODE_F32_ADD = 0x92,
    CHIM_WASM_OPCODE_F32_SUB = 0x93,
    CHIM_WASM_OPCODE_F32_MUL = 0x94,
    CHIM_WASM_OPCODE_F32_DIV = 0x95,
    CHIM_WASM_OPCODE_F32_MIN = 0x96,
    CHIM_WASM_OPCODE_F32_MAX = 0x97,
    CHIM_WASM_OPCODE_F32_COPYSIGN = 0x98,
    CHIM_WASM_OPCODE_F64_ABS = 0x99,
    CHIM_WASM_OPCODE_F64_NEG = 0x9A,
    CHIM_WASM_OPCODE_F64_CEIL = 0x9B,
    CHIM_WASM_OPCODE_F64_FLOOR = 0x9C,
    CHIM_WASM_OPCODE_F64_TRUNC = 0x9D,
    CHIM_WASM_OPCODE_F64_NEAREST = 0x9E,
    CHIM_WASM_OPCODE_F64_SQRT = 0x9F,
    CHIM_WASM_OPCODE_F64_ADD = 0xA0,
    CHIM_WASM_OPCODE_F64_SUB = 0xA1,
    CHIM_WASM_OPCODE_F64_MUL = 0xA2,
    CHIM_WASM_OPCODE_F64_DIV = 0xA3,
    CHIM_WASM_OPCODE_F64_MIN = 0xA4,
    CHIM_WASM_OPCODE_F64_MAX = 0xA5,
    CHIM_WASM_OPCODE_F64_COPYSIGN = 0xA6,
    CHIM_WASM_OPCODE_I32_TRUNC_SAT_F32_S = 0xA8,
    CHIM_WASM_OPCODE_I32_TRUNC_SAT_F32_U = 0xA9,
    CHIM_WASM_OPCODE_I32_TRUNC_SAT_F64_S = 0xAA,
    CHIM_WASM_OPCODE_I32_TRUNC_SAT_F64_U = 0xAB,
    CHIM_WASM_OPCODE_I64_TRUNC_SAT_F32_S = 0xAC,
    CHIM_WASM_OPCODE_I64_TRUNC_SAT_F32_U = 0xAD,
    CHIM_WASM_OPCODE_I64_TRUNC_SAT_F64_S = 0xAE,
    CHIM_WASM_OPCODE_I64_TRUNC_SAT_F64_U = 0xAF,
    CHIM_WASM_OPCODE_MEMORY_INIT = 0xFC8,
    CHIM_WASM_OPCODE_DATA_DROP = 0xFC9,
    CHIM_WASM_OPCODE_MEMORY_COPY = 0xFCB,
    CHIM_WASM_OPCODE_MEMORY_FILL = 0xFCC,
    CHIM_WASM_OPCODE_TABLE_INIT = 0xFCC,
    CHIM_WASM_OPCODE_ELEM_DROP = 0xFD3,
    CHIM_WASM_OPCODE_TABLE_COPY = 0xFD6,
    CHIM_WASM_OPCODE_TABLE_GROW = 0xFD9,
    CHIM_WASM_OPCODE_TABLE_SIZE = 0xFDA,
    CHIM_WASM_OPCODE_TABLE_FILL = 0xFDB,
} chim_wasm_opcode_t;

typedef enum {
    CHIM_WASM_VALTYPE_I32 = 0x7F,
    CHIM_WASM_VALTYPE_I64 = 0x7E,
    CHIM_WASM_VALTYPE_F32 = 0x7D,
    CHIM_WASM_VALTYPE_F64 = 0x7C,
    CHIM_WASM_VALTYPE_FUNCREF = 0x70,
    CHIM_WASM_VALTYPE_EXTERNREF = 0x6F,
} chim_wasm_valtype_t;

typedef enum {
    CHIM_WASM_LIMITS_FLAG_HAS_MAX = 0x01,
    CHIM_WASM_LIMITS_FLAG_SHARED = 0x02,
} chim_wasm_limits_flag_t;

typedef struct {
    uint8_t flags;
    uint32_t initial;
    uint32_t maximum;
} chim_wasm_limits_t;

typedef struct {
    uint8_t type;
    chim_wasm_limits_t limits;
} chim_wasm_memory_type_t;

typedef struct {
    uint8_t element_type;
    chim_wasm_limits_t limits;
} chim_wasm_table_type_t;

typedef struct {
    uint8_t valtype;
} chim_wasm_global_type_t;

typedef enum {
    CHIM_WASM_EXPORT_KIND_FUNCTION = 0,
    CHIM_WASM_EXPORT_KIND_TABLE = 1,
    CHIM_WASM_EXPORT_KIND_MEMORY = 2,
    CHIM_WASM_EXPORT_KIND_GLOBAL = 3,
} chim_wasm_export_kind_t;

typedef struct {
    chim_wasm_export_kind_t kind;
    chim_wasm_limits_t limits;
} chim_wasm_export_desc_t;

typedef struct {
    const char* name;
    size_t name_len;
    uint8_t kind;
    uint32_t index;
} chim_wasm_export_t;

typedef struct {
    uint32_t type_index;
    uint32_t local_count;
} chim_wasm_function_body_t;

typedef struct {
    chim_wasm_valtype_t result_type;
    uint32_t param_count;
    chim_wasm_valtype_t* param_types;
} chim_wasm_func_type_t;

typedef struct {
    uint8_t* data;
    size_t size;
    size_t capacity;
} chim_wasm_buffer_t;

typedef struct {
    chim_wasm_buffer_t type_section;
    chim_wasm_buffer_t import_section;
    chim_wasm_buffer_t function_section;
    chim_wasm_buffer_t table_section;
    chim_wasm_buffer_t memory_section;
    chim_wasm_buffer_t global_section;
    chim_wasm_buffer_t export_section;
    chim_wasm_buffer_t start_section;
    chim_wasm_buffer_t element_section;
    chim_wasm_buffer_t code_section;
    chim_wasm_buffer_t data_section;

    uint32_t type_count;
    uint32_t function_count;
    uint32_t table_count;
    uint32_t memory_count;
    uint32_t global_count;
    uint32_t export_count;
    uint32_t element_count;
    uint32_t data_count;

    chim_wasm_func_type_t* function_types;
    uint32_t function_type_count;

    chim_ir_module_t* module;
    int temp_counter;
    int label_counter;
} chim_wasm_encoder_t;

chim_wasm_encoder_t* chim_wasm_encoder_create(chim_ir_module_t* module);
void chim_wasm_encoder_destroy(chim_wasm_encoder_t* encoder);

bool chim_wasm_encode_module(chim_wasm_encoder_t* encoder, const char* filename);

const uint8_t* chim_wasm_get_module_data(chim_wasm_encoder_t* encoder, size_t* size);

void chim_wasm_encoder_emit_leb128(chim_wasm_buffer_t* buffer, uint64_t value, bool signed_encoding);
void chim_wasm_encoder_emit_leb128_signed(chim_wasm_buffer_t* buffer, int64_t value);
void chim_wasm_encoder_emit_valtype(chim_wasm_buffer_t* buffer, chim_wasm_valtype_t valtype);

uint8_t chim_wasm_map_chim_type(chim_ir_type_t* type);

#endif /* CHIM_WASM_BACKEND_H */
