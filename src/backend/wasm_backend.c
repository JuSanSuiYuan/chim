/**
 * @file wasm_backend.c
 * @brief Chim 3.1 WebAssembly 后端实现
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "wasm_backend.h"
#include "ir.h"
#include "common.h"

static void chim_wasm_buffer_init(chim_wasm_buffer_t* buffer) {
    buffer->data = NULL;
    buffer->size = 0;
    buffer->capacity = 0;
}

static void chim_wasm_buffer_destroy(chim_wasm_buffer_t* buffer) {
    if (buffer->data) {
        chim_free(buffer->data);
        buffer->data = NULL;
    }
    buffer->size = 0;
    buffer->capacity = 0;
}

static void chim_wasm_buffer_grow(chim_wasm_buffer_t* buffer, size_t needed) {
    if (buffer->size + needed <= buffer->capacity) {
        return;
    }

    size_t new_capacity = buffer->capacity ? buffer->capacity * 2 : 256;
    while (buffer->size + needed > new_capacity) {
        new_capacity *= 2;
    }

    buffer->data = (uint8_t*)chim_realloc(buffer->data, new_capacity);
    buffer->capacity = new_capacity;
}

static void chim_wasm_buffer_write(chim_wasm_buffer_t* buffer, const void* data, size_t size) {
    chim_wasm_buffer_grow(buffer, size);
    memcpy(buffer->data + buffer->size, data, size);
    buffer->size += size;
}

static void chim_wasm_buffer_putc(chim_wasm_buffer_t* buffer, uint8_t byte) {
    chim_wasm_buffer_grow(buffer, 1);
    buffer->data[buffer->size++] = byte;
}

void chim_wasm_encoder_emit_leb128(chim_wasm_buffer_t* buffer, uint64_t value, bool signed_encoding) {
    if (!signed_encoding) {
        while (value >= 0x80) {
            chim_wasm_buffer_putc(buffer, (uint8_t)(value & 0x7F) | 0x80);
            value >>= 7;
        }
        chim_wasm_buffer_putc(buffer, (uint8_t)value);
    } else {
        int64_t signed_value = (int64_t)value;
        bool done = false;
        while (!done) {
            uint8_t byte = signed_value & 0x7F;
            signed_value >>= 7;
            if ((signed_value == 0 && (byte & 0x40) == 0) ||
                (signed_value == -1 && (byte & 0x40) != 0)) {
                done = true;
            } else {
                byte | = 0x80;
            }
            chim_wasm_buffer_putc(buffer, byte);
        }
    }
}

void chim_wasm_encoder_emit_leb128_signed(chim_wasm_buffer_t* buffer, int64_t value) {
    bool done = false;
    while (!done) {
        uint8_t byte = value & 0x7F;
        value >>= 7;
        if ((value == 0 && (byte & 0x40) == 0) ||
            (value == -1 && (byte & 0x40) != 0)) {
            done = true;
        } else {
            byte |= 0x80;
        }
        chim_wasm_buffer_putc(buffer, byte);
    }
}

void chim_wasm_encoder_emit_valtype(chim_wasm_buffer_t* buffer, chim_wasm_valtype_t valtype) {
    chim_wasm_buffer_putc(buffer, valtype);
}

static void chim_wasm_encoder_emit_limits(chim_wasm_buffer_t* buffer, chim_wasm_limits_t* limits) {
    uint8_t flags = limits->flags;
    chim_wasm_buffer_putc(buffer, flags);
    chim_wasm_encoder_emit_leb128(buffer, limits->initial, false);
    if (flags & CHIM_WASM_LIMITS_FLAG_HAS_MAX) {
        chim_wasm_encoder_emit_leb128(buffer, limits->maximum, false);
    }
}

static void chim_wasm_encoder_emit_type_section(chim_wasm_encoder_t* encoder) {
    if (encoder->function_type_count == 0) {
        return;
    }

    chim_wasm_buffer_t* section = &encoder->type_section;

    for (uint32_t i = 0; i < encoder->function_type_count; i++) {
        chim_wasm_func_type_t* func_type = &encoder->function_types[i];

        chim_wasm_buffer_putc(section, CHIM_WASM_TYPE_FUNC);

        chim_wasm_encoder_emit_leb128(section, func_type->param_count, false);
        for (uint32_t j = 0; j < func_type->param_count; j++) {
            chim_wasm_encoder_emit_valtype(section, func_type->param_types[j]);
        }

        uint32_t result_count = func_type->result_type == CHIM_WASM_VALTYPE_VOID ? 0 : 1;
        chim_wasm_encoder_emit_leb128(section, result_count, false);
        if (result_count > 0) {
            chim_wasm_encoder_emit_valtype(section, func_type->result_type);
        }
    }

    chim_wasm_buffer_putc(section, 0);
    encoder->type_count = encoder->function_type_count;
}

static void chim_wasm_encoder_emit_function_section(chim_wasm_encoder_t* encoder) {
    chim_wasm_buffer_t* section = &encoder->function_section;

    chim_ir_function_t* func = encoder->module->functions;
    while (func) {
        chim_wasm_buffer_putc(section, 1);
        func = func->next;
    }

    if (encoder->function_count > 0) {
        chim_wasm_buffer_putc(section, 0);
        encoder->function_count = 1;
    }
}

static void chim_wasm_encoder_emit_code_section(chim_wasm_encoder_t* encoder) {
    chim_ir_function_t* func = encoder->module->functions;
    while (func) {
        chim_wasm_buffer_t* code_section = &encoder->code_section;

        chim_wasm_buffer_putc(code_section, CHIM_WASM_OPCODE_END);

        func = func->next;
    }

    if (encoder->code_section.size > 0) {
        chim_wasm_buffer_putc(&encoder->code_section, 0);
    }
}

static void chim_wasm_encoder_emit_memory_section(chim_wasm_encoder_t* encoder) {
    chim_wasm_buffer_t* section = &encoder->memory_section;

    chim_wasm_buffer_putc(section, 1);

    chim_wasm_memory_type_t memory;
    memory.type = 0x00;
    memory.limits.flags = 1;
    memory.limits.initial = 256;
    memory.limits.maximum = 512;

    chim_wasm_encoder_emit_limits(section, &memory.limits);
}

static void chim_wasm_encoder_emit_export_section(chim_wasm_encoder_t* encoder) {
    chim_wasm_buffer_t* section = &encoder->export_section;

    chim_wasm_buffer_putc(section, 1);

    const char* export_name = "memory";
    chim_wasm_buffer_putc(section, strlen(export_name));
    for (size_t i = 0; i < strlen(export_name); i++) {
        chim_wasm_buffer_putc(section, export_name[i]);
    }

    chim_wasm_buffer_putc(section, CHIM_WASM_EXPORT_KIND_MEMORY);
    chim_wasm_encoder_emit_leb128(section, 0, false);
}

static void chim_wasm_encoder_emit_start_section(chim_wasm_encoder_t* encoder) {
    if (!encoder->module->functions) {
        return;
    }

    chim_wasm_buffer_t* section = &encoder->start_section;
    chim_wasm_encoder_emit_leb128(section, 0, false);
}

static void chim_wasm_encoder_write_header(chim_wasm_encoder_t* encoder, FILE* fp) {
    fwrite("\0asm", 4, 1, fp);
    uint8_t version = 1;
    fwrite(&version, 1, 1, fp);
}

static void chim_wasm_encoder_write_section(FILE* fp, chim_wasm_buffer_t* section, uint8_t section_id) {
    if (section->size == 0) {
        return;
    }

    fputc(section_id, fp);
    size_t size = section->size;
    for (int i = 24; i >= 0; i -= 8) {
        fputc((size >> i) & 0xFF, fp);
    }
    fwrite(section->data, 1, section->size, fp);
}

chim_wasm_encoder_t* chim_wasm_encoder_create(chim_ir_module_t* module) {
    chim_wasm_encoder_t* encoder = (chim_wasm_encoder_t*)chim_alloc(sizeof(chim_wasm_encoder_t));
    if (!encoder) return NULL;

    memset(encoder, 0, sizeof(chim_wasm_encoder_t));

    chim_wasm_buffer_init(&encoder->type_section);
    chim_wasm_buffer_init(&encoder->import_section);
    chim_wasm_buffer_init(&encoder->function_section);
    chim_wasm_buffer_init(&encoder->table_section);
    chim_wasm_buffer_init(&encoder->memory_section);
    chim_wasm_buffer_init(&encoder->global_section);
    chim_wasm_buffer_init(&encoder->export_section);
    chim_wasm_buffer_init(&encoder->start_section);
    chim_wasm_buffer_init(&encoder->element_section);
    chim_wasm_buffer_init(&encoder->code_section);
    chim_wasm_buffer_init(&encoder->data_section);

    encoder->module = module;
    encoder->temp_counter = 0;
    encoder->label_counter = 0;

    encoder->function_type_count = 0;
    encoder->function_types = NULL;

    return encoder;
}

void chim_wasm_encoder_destroy(chim_wasm_encoder_t* encoder) {
    if (!encoder) return;

    chim_wasm_buffer_destroy(&encoder->type_section);
    chim_wasm_buffer_destroy(&encoder->import_section);
    chim_wasm_buffer_destroy(&encoder->function_section);
    chim_wasm_buffer_destroy(&encoder->table_section);
    chim_wasm_buffer_destroy(&encoder->memory_section);
    chim_wasm_buffer_destroy(&encoder->global_section);
    chim_wasm_buffer_destroy(&encoder->export_section);
    chim_wasm_buffer_destroy(&encoder->start_section);
    chim_wasm_buffer_destroy(&encoder->element_section);
    chim_wasm_buffer_destroy(&encoder->code_section);
    chim_wasm_buffer_destroy(&encoder->data_section);

    if (encoder->function_types) {
        for (uint32_t i = 0; i < encoder->function_type_count; i++) {
            if (encoder->function_types[i].param_types) {
                chim_free(encoder->function_types[i].param_types);
            }
        }
        chim_free(encoder->function_types);
    }

    chim_free(encoder);
}

bool chim_wasm_encode_module(chim_wasm_encoder_t* encoder, const char* filename) {
    if (!encoder || !filename) return false;

    FILE* fp = fopen(filename, "wb");
    if (!fp) {
        return false;
    }

    chim_wasm_encoder_write_header(encoder, fp);

    chim_wasm_encoder_emit_type_section(encoder);
    chim_wasm_encoder_write_section(fp, &encoder->type_section, CHIM_WASM_SECTION_TYPE);

    chim_wasm_encoder_emit_function_section(encoder);
    chim_wasm_encoder_write_section(fp, &encoder->function_section, CHIM_WASM_SECTION_FUNCTION);

    chim_wasm_encoder_emit_memory_section(encoder);
    chim_wasm_encoder_write_section(fp, &encoder->memory_section, CHIM_WASM_SECTION_MEMORY);

    chim_wasm_encoder_emit_export_section(encoder);
    chim_wasm_encoder_write_section(fp, &encoder->export_section, CHIM_WASM_SECTION_EXPORT);

    chim_wasm_encoder_emit_code_section(encoder);
    chim_wasm_encoder_write_section(fp, &encoder->code_section, CHIM_WASM_SECTION_CODE);

    fclose(fp);

    return true;
}

const uint8_t* chim_wasm_get_module_data(chim_wasm_encoder_t* encoder, size_t* size) {
    if (!encoder || !size) return NULL;

    size_t total_size = 8;
    total_size += encoder->type_section.size;
    total_size += encoder->function_section.size;
    total_size += encoder->memory_section.size;
    total_size += encoder->export_section.size;
    total_size += encoder->code_section.size;

    uint8_t* data = (uint8_t*)chim_alloc(total_size);
    if (!data) return NULL;

    size_t offset = 0;
    data[offset++] = '\0';
    data[offset++] = 'a';
    data[offset++] = 's';
    data[offset++] = 'm';
    data[offset++] = 1;

    if (encoder->type_section.size > 0) {
        data[offset++] = CHIM_WASM_SECTION_TYPE;
        for (int i = 24; i >= 0; i -= 8) {
            data[offset++] = (encoder->type_section.size >> i) & 0xFF;
        }
        memcpy(&data[offset], encoder->type_section.data, encoder->type_section.size);
        offset += encoder->type_section.size;
    }

    if (encoder->function_section.size > 0) {
        data[offset++] = CHIM_WASM_SECTION_FUNCTION;
        for (int i = 24; i >= 0; i -= 8) {
            data[offset++] = (encoder->function_section.size >> i) & 0xFF;
        }
        memcpy(&data[offset], encoder->function_section.data, encoder->function_section.size);
        offset += encoder->function_section.size;
    }

    if (encoder->memory_section.size > 0) {
        data[offset++] = CHIM_WASM_SECTION_MEMORY;
        for (int i = 24; i >= 0; i -= 8) {
            data[offset++] = (encoder->memory_section.size >> i) & 0xFF;
        }
        memcpy(&data[offset], encoder->memory_section.data, encoder->memory_section.size);
        offset += encoder->memory_section.size;
    }

    if (encoder->export_section.size > 0) {
        data[offset++] = CHIM_WASM_SECTION_EXPORT;
        for (int i = 24; i >= 0; i -= 8) {
            data[offset++] = (encoder->export_section.size >> i) & 0xFF;
        }
        memcpy(&data[offset], encoder->export_section.data, encoder->export_section.size);
        offset += encoder->export_section.size;
    }

    if (encoder->code_section.size > 0) {
        data[offset++] = CHIM_WASM_SECTION_CODE;
        for (int i = 24; i >= 0; i -= 8) {
            data[offset++] = (encoder->code_section.size >> i) & 0xFF;
        }
        memcpy(&data[offset], encoder->code_section.data, encoder->code_section.size);
        offset += encoder->code_section.size;
    }

    *size = offset;
    return data;
}

uint8_t chim_wasm_map_chim_type(chim_ir_type_t* type) {
    if (!type) return CHIM_WASM_VALTYPE_I32;

    switch (type->kind) {
        case CHIR_TYPE_BOOL:
        case CHIR_TYPE_INT8:
        case CHIR_TYPE_INT16:
        case CHIR_TYPE_INT32:
        case CHIR_TYPE_UINT8:
        case CHIR_TYPE_UINT16:
        case CHIR_TYPE_UINT32:
            return CHIM_WASM_VALTYPE_I32;

        case CHIR_TYPE_INT64:
        case CHIR_TYPE_UINT64:
            return CHIM_WASM_VALTYPE_I64;

        case CHIR_TYPE_FLOAT32:
            return CHIM_WASM_VALTYPE_F32;

        case CHIR_TYPE_FLOAT64:
            return CHIM_WASM_VALTYPE_F64;

        case CHIR_TYPE_VOID:
        case CHIR_TYPE_FN:
        case CHIR_TYPE_PTR:
        case CHIR_TYPE_STRING:
        case CHIR_TYPE_LIST:
        case CHIR_TYPE_STRUCT:
        case CHIR_TYPE_ENUM:
        default:
            return CHIM_WASM_VALTYPE_I32;
    }
}
