#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uchar, c_uint, c_ulonglong, c_void};
use std::ptr;

// 导入词法分析器和语法分析器
mod lexer;
mod parser;

use lexer::Lexer;
use parser::Parser;

#[repr(C)]
pub struct ChimIRFunction {
    pub name: *mut c_char,
    pub args: *mut ChimIRArg,
    pub arg_count: c_uint,
    pub return_type: *mut c_char,
}

#[repr(C)]
pub struct ChimIRModule {
    pub funcs: *mut ChimIRFunction,
    pub func_count: c_uint,
}

#[repr(C)]
pub struct ChimIRArg {
    pub name: *mut c_char,
    pub r#type: *mut c_char,
}

// 内部状态跟踪结构体
struct CompilerState {
    strings: Vec<CString>, // 用于保存字符串，避免过早释放
}

#[no_mangle]
pub extern "C" fn chim_version() -> c_uint { 2 } // 更新版本号

#[no_mangle]
pub extern "C" fn chim_lex(input_ptr: *const c_uchar, input_len: usize) -> *const c_char {
    eprintln!("[Rust] chim_lex called with len={}", input_len);
    
    if input_ptr.is_null() || input_len == 0 { 
        eprintln!("[Rust] Empty input");
        let empty = CString::new("").unwrap();
        return empty.into_raw();
    }
    
    let slice = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };
    let source = match std::str::from_utf8(slice) { 
        Ok(s) => {
            eprintln!("[Rust] Source decoded: {} chars", s.len());
            s
        }, 
        Err(e) => {
            eprintln!("[Rust] UTF-8 error: {:?}", e);
            let error = CString::new("Error: Invalid UTF-8").unwrap();
            return error.into_raw();
        }
    };
    
    eprintln!("[Rust] Creating lexer...");
    // 使用词法分析器
    let mut lexer = Lexer::new(source);
    
    eprintln!("[Rust] Running lex_all...");
    let tokens = lexer.lex_all();
    
    eprintln!("[Rust] Got {} tokens", tokens.len());
    
    // 构建结果字符串
    let mut result = String::new();
    for token in tokens {
        result.push_str(&format!("{:?}\n", token));
    }
    
    eprintln!("[Rust] Building result string: {} bytes", result.len());
    
    let cstring = CString::new(result).unwrap();
    let ptr = cstring.into_raw();
    
    eprintln!("[Rust] Returning pointer: {:p}", ptr);
    ptr
}

#[no_mangle]
pub extern "C" fn chim_build_ir(input_ptr: *const c_uchar, input_len: c_ulonglong, out_module: *mut ChimIRModule) -> c_uint {
    if input_ptr.is_null() || out_module.is_null() { return 0; }
    
    let slice = unsafe { std::slice::from_raw_parts(input_ptr, input_len as usize) };
    let source = match std::str::from_utf8(slice) { Ok(s) => s, Err(_) => return 0 };
    
    // 创建编译器状态
    let mut state = CompilerState {
        strings: Vec::new(),
    };
    
    // 首先进行词法分析
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex_all();
    
    // 然后进行语法分析
    let mut parser = Parser::new();
    let program = match parser.parse(source) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("Parse error: {:?}", err);
            return 0;
        },
    };
    
    // 从AST构建IR
    let mut funcs: Vec<ChimIRFunction> = Vec::new();
    
    // 扫描程序中的函数定义
    for statement in program.statements {
        if let parser::Statement::FunctionDefinition(func_def) = statement {
            // 提取函数名
            let name_c = CString::new(func_def.name.clone()).unwrap();
            state.strings.push(name_c.clone());
            
            // 提取返回类型
            let return_type_c = match func_def.return_type {
                Some(ref typ) => {
                    let type_str = format_type(typ);
                    let cstr = CString::new(type_str).unwrap();
                    state.strings.push(cstr.clone());
                    cstr
                },
                None => {
                    let cstr = CString::new("void").unwrap();
                    state.strings.push(cstr.clone());
                    cstr
                },
            };
            
            // 处理参数
            let mut args_vec: Vec<ChimIRArg> = Vec::new();
            for param in func_def.parameters {
                let arg_name_c = CString::new(param.name.clone()).unwrap();
                let arg_type_c = CString::new(format_type(&param.type_annotation)).unwrap();
                
                state.strings.push(arg_name_c.clone());
                state.strings.push(arg_type_c.clone());
                
                args_vec.push(ChimIRArg {
                    name: arg_name_c.into_raw(),
                    r#type: arg_type_c.into_raw(),
                });
            }
            
            let arg_count = args_vec.len() as c_uint;
            let boxed_args: Box<[ChimIRArg]> = args_vec.into_boxed_slice();
            let ptr_args = if arg_count > 0 { 
                Box::into_raw(boxed_args) as *mut ChimIRArg 
            } else { 
                ptr::null_mut() 
            };
            
            funcs.push(ChimIRFunction {
                name: name_c.into_raw(),
                args: ptr_args,
                arg_count,
                return_type: return_type_c.into_raw(),
            });
        }
    }

    // 构建数组并写入 out_module
    let count = funcs.len() as c_uint;
    let boxed: Box<[ChimIRFunction]> = funcs.into_boxed_slice();
    let ptr_funcs = Box::into_raw(boxed) as *mut ChimIRFunction;
    
    unsafe {
        (*out_module).funcs = ptr_funcs;
        (*out_module).func_count = count;
    }
    
    // 忘记state，让字符串保持在内存中直到IR被释放
    std::mem::forget(state);
    
    1
}

// 格式化类型为字符串
fn format_type(typ: &parser::Type) -> String {
    match typ {
        parser::Type::Identifier(id) => id.clone(),
        parser::Type::Tuple(types) => {
            let types_str = types.iter()
                .map(format_type)
                .collect::<Vec<String>>()
                .join(", ");
            format!("({})", types_str)
        },
        parser::Type::Array(elem_type) => {
            format!("[{}]", format_type(elem_type))
        },
        parser::Type::Function(params, ret) => {
            let params_str = params.iter()
                .map(format_type)
                .collect::<Vec<String>>()
                .join(", ");
            format!("fn({}) -> {}", params_str, format_type(ret))
        },
        parser::Type::Generic(id, args) => {
            let args_str = args.iter()
                .map(format_type)
                .collect::<Vec<String>>()
                .join(", ");
            format!("{}<{}>", id, args_str)
        },
    }
}

#[no_mangle]
pub extern "C" fn chim_ir_free(module: *mut ChimIRModule) {
    if module.is_null() { return; }
    unsafe {
        let count = (*module).func_count as usize;
        let funcs_ptr = (*module).funcs;
        if !funcs_ptr.is_null() && count > 0 {
            // 释放每个函数
            for i in 0..count {
                let func = funcs_ptr.add(i);
                if !(*func).name.is_null() {
                    let _ = CString::from_raw((*func).name);
                }
                // 释放参数
                let ac = (*func).arg_count as usize;
                let args_ptr = (*func).args;
                if !args_ptr.is_null() && ac > 0 {
                    for j in 0..ac {
                        let arg = args_ptr.add(j);
                        if !(*arg).name.is_null() { let _ = CString::from_raw((*arg).name); }
                        if !(*arg).r#type.is_null() { let _ = CString::from_raw((*arg).r#type); }
                    }
                    let slice_ptr2 = std::ptr::slice_from_raw_parts_mut(args_ptr, ac);
                    let _ = Box::from_raw(slice_ptr2);
                }
                if !(*func).return_type.is_null() { let _ = CString::from_raw((*func).return_type); }
            }
            // 释放函数数组本身
            let slice_ptr = std::ptr::slice_from_raw_parts_mut(funcs_ptr, count);
            let _ = Box::from_raw(slice_ptr);
        }
        (*module).funcs = ptr::null_mut();
        (*module).func_count = 0;
    }
}

// 添加新的FFI函数：解析源代码并返回AST
#[no_mangle]
pub extern "C" fn chim_parse(input_ptr: *const c_uchar, input_len: usize) -> *mut parser::Program {
    if input_ptr.is_null() || input_len == 0 { 
        return ptr::null_mut();
    }
    
    let slice = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };
    let source = match std::str::from_utf8(slice) { 
        Ok(s) => s, 
        Err(_) => return ptr::null_mut()
    };
    
    let mut parser = Parser::new();
    match parser.parse(source) {
        Ok(program) => {
            let program_box = Box::new(program);
            Box::into_raw(program_box)
        },
        Err(err) => {
            eprintln!("Parse error: {:?}", err);
            ptr::null_mut()
        },
    }
}

// 释放解析的程序AST
#[no_mangle]
pub extern "C" fn chim_free_program(program: *mut parser::Program) {
    if !program.is_null() {
        unsafe {
            Box::from_raw(program);
        }
    }
}

// 检查是否支持某个特性
#[no_mangle]
pub extern "C" fn chim_has_feature(feature: *const c_char) -> bool {
    let feature_str = unsafe {
        CStr::from_ptr(feature).to_str().unwrap_or("")
    };
    
    match feature_str {
        "lexer" => true,
        "parser" => true,
        "ast" => true,
        "ir" => true,
        "logos" => true,
        "pest" => true,
        _ => false,
    }
}

// 清理分配的字符串
#[no_mangle]
pub extern "C" fn chim_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            CString::from_raw(s);
        }
    }
}
