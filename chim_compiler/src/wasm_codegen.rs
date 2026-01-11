use crate::ir::{Module, Function, Instruction, Type as IRType};

pub trait TargetCodeGenerator {
    fn generate(&self, module: &Module) -> String;
    fn get_target(&self) -> &str;
}

pub struct WASMGenerator;

impl WASMGenerator {
    pub fn new() -> Self {
        Self
    }
    
    fn type_to_wasm(ty: &IRType) -> String {
        match ty {
            IRType::Void => "[] -> []".to_string(),
            IRType::Int32 => "[] -> [i32]".to_string(),
            IRType::Int64 => "[] -> [i64]".to_string(),
            IRType::Float32 => "[] -> [f32]".to_string(),
            IRType::Float64 => "[] -> [f64]".to_string(),
            IRType::Bool => "[] -> [i32]".to_string(),
            IRType::String => "[] -> [i32]".to_string(),
            IRType::Ptr(inner) => format!("[] -> [{}]", Self::type_to_wasm(inner)),
            IRType::Ref(inner) => format!("[] -> [{}]", Self::type_to_wasm(inner)),
            IRType::MutRef(inner) => format!("[] -> [{}]", Self::type_to_wasm(inner)),
            IRType::Array(inner, _) => format!("[] -> [{}]", Self::type_to_wasm(inner)),
            IRType::Struct(name) => format!("[] -> [i32]  ;; struct {}", name),
        }
    }
    
    fn instruction_to_wasm(&self, inst: &Instruction, func: &Function) -> String {
        match inst {
            Instruction::Alloca { dest, ty } => {
                format!(
                    "  (local {} {})",
                    dest,
                    Self::local_type_to_wasm(ty)
                )
            },
            
            Instruction::Load { dest, src } => {
                format!(
                    "  (local.set {} (local.get {}))",
                    dest, src
                )
            },
            
            Instruction::Store { dest, src } => {
                format!(
                    "  (local.set {} (local.get {}))",
                    dest, src
                )
            },
            
            Instruction::Add { dest, left, right } => {
                format!(
                    "  (local.set {} (i32.add (local.get {}) (local.get {})))",
                    dest, left, right
                )
            },
            
            Instruction::Sub { dest, left, right } => {
                format!(
                    "  (local.set {} (i32.sub (local.get {}) (local.get {})))",
                    dest, left, right
                )
            },
            
            Instruction::Mul { dest, left, right } => {
                format!(
                    "  (local.set {} (i32.mul (local.get {}) (local.get {})))",
                    dest, left, right
                )
            },
            
            Instruction::Div { dest, left, right } => {
                format!(
                    "  (local.set {} (i32.div_s (local.get {}) (local.get {})))",
                    dest, left, right
                )
            },
            
            Instruction::Mod { dest, left, right } => {
                format!(
                    "  (local.set {} (i32.rem_s (local.get {}) (local.get {})))",
                    dest, left, right
                )
            },
            
            Instruction::Eq { dest, left, right } => {
                format!(
                    "  (local.set {} (i32.eq (local.get {}) (local.get {})))",
                    dest, left, right
                )
            },
            
            Instruction::Ne { dest, left, right } => {
                format!(
                    "  (local.set {} (i32.ne (local.get {}) (local.get {})))",
                    dest, left, right
                )
            },
            
            Instruction::Lt { dest, left, right } => {
                format!(
                    "  (local.set {} (i32.lt_s (local.get {}) (local.get {})))",
                    dest, left, right
                )
            },
            
            Instruction::Le { dest, left, right } => {
                format!(
                    "  (local.set {} (i32.le_s (local.get {}) (local.get {})))",
                    dest, left, right
                )
            },
            
            Instruction::Gt { dest, left, right } => {
                format!(
                    "  (local.set {} (i32.gt_s (local.get {}) (local.get {})))",
                    dest, left, right
                )
            },
            
            Instruction::Ge { dest, left, right } => {
                format!(
                    "  (local.set {} (i32.ge_s (local.get {}) (local.get {})))",
                    dest, left, right
                )
            },
            
            Instruction::And { dest, left, right } => {
                format!(
                    "  (local.set {} (i32.and (local.get {}) (local.get {})))",
                    dest, left, right
                )
            },
            
            Instruction::Or { dest, left, right } => {
                format!(
                    "  (local.set {} (i32.or (local.get {}) (local.get {})))",
                    dest, left, right
                )
            },
            
            Instruction::Not { dest, src } => {
                format!(
                    "  (local.set {} (i32.eqz (local.get {})))",
                    dest, src
                )
            },
            
            Instruction::Call { dest, func, args } => {
                if let Some(d) = dest {
                    format!(
                        "  (local.set {} (call ${} {}))",
                        d,
                        func,
                        args.iter()
                            .map(|a| format!("(local.get {})", a))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                } else {
                    format!(
                        "  (call ${} {})",
                        func,
                        args.iter()
                            .map(|a| format!("(local.get {})", a))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                }
            },
            
            Instruction::Br(label) => {
                format!("  (br {})", label)
            },
            
            Instruction::CondBr { cond, true_bb, false_bb } => {
                format!(
                    "  (if (local.get {}) (then (br {})) (else (br {})))",
                    cond, true_bb, false_bb
                )
            },
            
            Instruction::Label(name) => {
                format!("  (block {}", name)
            },
            
            Instruction::Return(Some(val)) => {
                format!("  (return (local.get {}))", val)
            },
            
            Instruction::Return(None) => {
                "  (return)".to_string()
            },
            
            Instruction::Nop => "  (nop)".to_string(),
            Instruction::Unreachable => "  (unreachable)".to_string(),
            
            Instruction::Borrow { dest, src, mutable: _ } => {
                format!(
                    "  (local.set {} (local.get {}))  ;; borrow",
                    dest, src
                )
            },
            
            Instruction::Deref { dest, src } => {
                format!(
                    "  (local.set {} (local.get {}))  ;; deref",
                    dest, src
                )
            },
            
            _ => format!("  ;; unimplemented: {:?}", inst),
        }
    }
    
    fn local_type_to_wasm(ty: &IRType) -> String {
        match ty {
            IRType::Int32 | IRType::Bool => "i32".to_string(),
            IRType::Int64 => "i64".to_string(),
            IRType::Float32 => "f32".to_string(),
            IRType::Float64 => "f64".to_string(),
            IRType::Void => "[]".to_string(),
            _ => "i32".to_string(),
        }
    }
    
    fn function_to_wasm(&self, func: &Function) -> String {
        let mut wasm = String::new();
        
        // 函数类型签名
        let params: Vec<String> = func.params
            .iter()
            .map(|(_, ty)| Self::local_type_to_wasm(ty))
            .collect();
        
        let result = Self::local_type_to_wasm(&func.return_type);
        
        // 导出函数
        if func.is_public || func.name == "main" {
            wasm.push_str(&format!("  (export \"{}\" (func ${}))\n", func.name, func.name));
        }
        
        // 函数定义
        wasm.push_str(&format!(
            "  (func ${} (param {}) (result {})\n",
            func.name,
            params.join(" "),
            result
        ));
        
        // 内核注解
        if func.is_kernel {
            wasm.push_str("    ;; @kernel\n");
        }
        
        // 局部变量
        let mut locals = Vec::new();
        for inst in &func.body {
            match inst {
                Instruction::Alloca { dest, ty } => {
                    locals.push(format!("    (local {} {})", dest, Self::local_type_to_wasm(ty)));
                },
                _ => {}
            }
        }
        wasm.push_str(&locals.join("\n"));
        if !locals.is_empty() {
            wasm.push_str("\n");
        }
        
        // 指令
        for inst in &func.body {
            let inst_str = self.instruction_to_wasm(inst, func);
            if !inst_str.trim().is_empty() && inst_str != "  (nop)" {
                wasm.push_str(&format!("{}\n", inst_str));
            }
        }
        
        wasm.push_str("  )\n");
        
        wasm
    }
}

impl TargetCodeGenerator for WASMGenerator {
    fn generate(&self, module: &Module) -> String {
        let mut wasm = String::new();
        
        // WebAssembly模块头
        wasm.push_str("(module\n");
        wasm.push_str("  ;; Chim Programming Language - WebAssembly Output\n");
        wasm.push_str(&format!("  ;; Generated {} functions, {} structs\n\n", 
            module.functions.len(), module.structs.len()));
        
        // 内存布局
        wasm.push_str("  ;; Memory configuration\n");
        wasm.push_str("  (memory (export \"memory\") 1)\n");
        wasm.push_str("  (global (export \"heap_base\") i32 (i32.const 0))\n\n");
        
        // 内置函数
        wasm.push_str("  ;; Built-in functions\n");
        wasm.push_str(r#"  (func $print (param $msg i32)
    (call $puts (local.get $msg))
  )
  (import "env" "puts" (func $puts (param i32)))
"#);
        wasm.push_str(r#"  (func $println (param $msg i32)
    (call $puts (local.get $msg))
    (call $putchar (i32.const 10))
  )
  (import "env" "putchar" (func $putchar (param i32)))
"#);
        wasm.push_str(r#"  (func $len (param $str i32) (result i32)
    (local $len i32)
    (local $i i32)
    (block $exit
      (loop $loop
        (br_if $exit (i32.eqz (i32.load8_u (local.get $i))))
        (local.set $len (i32.add (local.get $len) (i32.const 1)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $loop)
      )
    )
    (local.get $len)
  )
"#);
        
        wasm.push_str("\n");
        
        // 用户定义函数
        for func in &module.functions {
            wasm.push_str(&self.function_to_wasm(func));
            wasm.push_str("\n");
        }
        
        // WebAssembly模块尾
        wasm.push_str(")\n");
        
        wasm
    }
    
    fn get_target(&self) -> &str {
        "wasm"
    }
}

impl Default for WASMGenerator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct NativeGenerator;

impl NativeGenerator {
    fn generate_instruction(instr: &Instruction) -> String {
        match instr {
            Instruction::Alloca { dest, ty } => {
                let ty_str = match ty {
                    IRType::Int32 => "int",
                    IRType::Float32 => "float",
                    IRType::Bool => "int",
                    IRType::String => "char*",
                    IRType::Array(_, _) => "void*",
                    IRType::Struct(name) => return format!("    struct {} {};\n", name, dest),
                    _ => "void*",
                };
                format!("    {} {};\n", ty_str, dest)
            }
            
            Instruction::Load { dest, src } => {
                format!("    {} = *({});\n", dest, src)
            }
            
            Instruction::Store { dest, src } => {
                format!("    *({}) = {};\n", dest, src)
            }
            
            Instruction::GetPointer { dest, src, offset } => {
                format!("    {} = (void*)((char*){} + {});\n", dest, src, offset)
            }
            
            Instruction::Add { dest, left, right } => {
                format!("    {} = {} + {};\n", dest, left, right)
            }
            
            Instruction::Sub { dest, left, right } => {
                format!("    {} = {} - {};\n", dest, left, right)
            }
            
            Instruction::Mul { dest, left, right } => {
                format!("    {} = {} * {};\n", dest, left, right)
            }
            
            Instruction::Div { dest, left, right } => {
                format!("    {} = {} / {};\n", dest, left, right)
            }
            
            Instruction::Mod { dest, left, right } => {
                format!("    {} = {} % {};\n", dest, left, right)
            }
            
            Instruction::Eq { dest, left, right } => {
                format!("    {} = ({} == {});\n", dest, left, right)
            }
            
            Instruction::Ne { dest, left, right } => {
                format!("    {} = ({} != {});\n", dest, left, right)
            }
            
            Instruction::Lt { dest, left, right } => {
                format!("    {} = ({} < {});\n", dest, left, right)
            }
            
            Instruction::Le { dest, left, right } => {
                format!("    {} = ({} <= {});\n", dest, left, right)
            }
            
            Instruction::Gt { dest, left, right } => {
                format!("    {} = ({} > {});\n", dest, left, right)
            }
            
            Instruction::Ge { dest, left, right } => {
                format!("    {} = ({} >= {});\n", dest, left, right)
            }
            
            Instruction::And { dest, left, right } => {
                format!("    {} = ({} && {});\n", dest, left, right)
            }
            
            Instruction::Or { dest, left, right } => {
                format!("    {} = ({} || {});\n", dest, left, right)
            }
            
            Instruction::Not { dest, src } => {
                format!("    {} = !{};\n", dest, src)
            }
            
            Instruction::Call { dest, func, args } => {
                if let Some(d) = dest {
                    format!("    {} = {}({});\n", d, func, args.join(", "))
                } else {
                    format!("    {}({});\n", func, args.join(", "))
                }
            }
            
            Instruction::Br(label) => {
                format!("    goto {};\n", label)
            }
            
            Instruction::CondBr { cond, true_bb, false_bb } => {
                format!("    if ({}) goto {}; else goto {};\n", cond, true_bb, false_bb)
            }
            
            Instruction::Label(name) => {
                format!("{}:\n", name)
            }
            
            Instruction::Return(None) => {
                "    return;\n".to_string()
            }
            
            Instruction::Return(Some(val)) => {
                format!("    return {};\n", val)
            }
            
            Instruction::ReturnInPlace(_) => {
                "    return;\n".to_string()
            }
            
            Instruction::Borrow { dest, src, mutable: _ } => {
                format!("    {} = &{};\n", dest, src)
            }
            
            Instruction::Deref { dest, src } => {
                format!("    {} = *{};\n", dest, src)
            }
            
            Instruction::Phi { dest, incoming } => {
                let pairs: Vec<String> = incoming
                    .iter()
                    .map(|(val, _)| val.clone())
                    .collect();
                format!("    // phi: {} = {}\n    {} = {};\n", dest, pairs.join(", "), dest, pairs.first().unwrap_or(&dest))
            }
            
            Instruction::ExtractValue { dest, src, index } => {
                format!("    {} = {}.field_{};\n", dest, src, index)
            }
            
            Instruction::InsertValue { dest, src, value, index } => {
                format!("    {}.field_{} = {};\n    {} = {};\n", src, index, value, dest, src)
            }
            
            Instruction::GetElementPtr { dest, src, indices } => {
                let indices_str: Vec<String> = indices.iter().map(|i| i.to_string()).collect();
                format!("    {} = (void*)((char*){} + {});\n", dest, src, indices_str.join(" + "))
            }
            
            Instruction::Nop => {
                "    // nop\n".to_string()
            }
            
            Instruction::Unreachable => {
                "    // unreachable\n    return;\n".to_string()
            }
        }
    }
}

impl TargetCodeGenerator for NativeGenerator {
    fn generate(&self, module: &Module) -> String {
        let mut output = String::new();
        
        // C风格输出
        output.push_str("// Chim Programming Language - Native Output\n\n");
        output.push_str("#include <stdio.h>\n");
        output.push_str("#include <stdlib.h>\n");
        output.push_str("#include <string.h>\n\n");
        
        // 类型定义
        output.push_str("// Type definitions\n");
        output.push_str("typedef int chim_int;\n");
        output.push_str("typedef float chim_float;\n");
        output.push_str("typedef int chim_bool;\n");
        output.push_str("typedef char* chim_string;\n\n");
        
        // 函数定义
        output.push_str("// Function definitions\n");
        for func in &module.functions {
            let ret_type = match func.return_type {
                IRType::Int32 => "int",
                IRType::Float32 => "float",
                IRType::Bool => "int",
                IRType::Void => "void",
                _ => "int",
            };
            
            output.push_str(&format!("{} {}(", ret_type, func.name));
            
            let params: Vec<String> = func.params
                .iter()
                .map(|(n, t)| {
                    let ty = match t {
                        IRType::Int32 => "int",
                        IRType::Float32 => "float",
                        IRType::Bool => "int",
                        _ => "int",
                    };
                    format!("{} {}", ty, n)
                })
                .collect();
            output.push_str(&params.join(", "));
            output.push_str(") {\n");
            
            // 函数体生成
            for instr in &func.body {
                let c_instr = Self::generate_instruction(instr);
                output.push_str(&c_instr);
            }
            
            output.push_str("}\n\n");
        }
        
        // 结构体定义
        if !module.structs.is_empty() {
            output.push_str("// Struct definitions\n");
            for struct_ in &module.structs {
                output.push_str(&format!("struct {} {{\n", struct_.name));
                for (field_name, field_type) in &struct_.fields {
                    let ty = match field_type {
                        IRType::Int32 => "int",
                        IRType::Float32 => "float",
                        IRType::Bool => "int",
                        IRType::String => "char*",
                        IRType::Array(_, _) => "void*",
                        IRType::Struct(s) => &s,
                        IRType::Ptr(t) => match t.as_ref() {
                            IRType::Int32 => "int*",
                            IRType::Float32 => "float*",
                            IRType::Bool => "int*",
                            IRType::String => "char**",
                            _ => "void*",
                        },
                        _ => "void*",
                    };
                    output.push_str(&format!("    {} {};\n", ty, field_name));
                }
                output.push_str("};\n\n");
            }
        }
        
        // 全局变量
        if !module.globals.is_empty() {
            output.push_str("// Global variables\n");
            for global in &module.globals {
                let ty = match &global.ty {
                    IRType::Int32 => "int",
                    IRType::Float32 => "float",
                    IRType::Bool => "int",
                    IRType::String => "char*",
                    _ => "void*",
                };
                output.push_str(&format!("{} {};\n", ty, global.name));
            }
        }
        
        output
    }
    
    fn get_target(&self) -> &str {
        "native"
    }
}

impl Default for NativeGenerator {
    fn default() -> Self {
        Self
    }
}
