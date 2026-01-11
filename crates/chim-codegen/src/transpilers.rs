use crate::{CodegenTarget, CodeGenerator, GeneratedCode, CodegenError};
use chim_ir::{IRModule, IRFunction, IRInst, BinaryOp, UnaryOp, Terminator, ValueId, BlockId};
use chim_semantic::{AnalyzedProgram, TypeId};
use std::collections::HashMap;

pub struct CTranspiler {
    value_names: HashMap<ValueId, String>,
    block_names: HashMap<BlockId, String>,
    next_value: usize,
    next_block: usize,
}

impl CTranspiler {
    pub fn new() -> Self {
        CTranspiler {
            value_names: HashMap::new(),
            block_names: HashMap::new(),
            next_value: 0,
            next_block: 0,
        }
    }

    fn get_value_name(&mut self, id: ValueId) -> String {
        if let Some(name) = self.value_names.get(&id) {
            name.clone()
        } else {
            let name = format!("v{}", id.0);
            self.value_names.insert(id, name.clone());
            name
        }
    }

    fn get_block_name(&mut self, id: BlockId) -> String {
        if let Some(name) = self.block_names.get(&id) {
            name.clone()
        } else {
            let name = format!("bb{}", id.0);
            self.block_names.insert(id, name.clone());
            name
        }
    }

    fn type_to_c(&self, ty: TypeId, program: &AnalyzedProgram) -> String {
        match program.type_of(ty) {
            chim_semantic::TypeData::Int(size) => match size {
                chim_semantic::IntSize::I8 => "int8_t".to_string(),
                chim_semantic::IntSize::I16 => "int16_t".to_string(),
                chim_semantic::IntSize::I32 => "int32_t".to_string(),
                chim_semantic::IntSize::I64 => "int64_t".to_string(),
                chim_semantic::IntSize::I128 => "__int128".to_string(),
                chim_semantic::IntSize::Isize => "intptr_t".to_string(),
            },
            chim_semantic::TypeData::Uint(size) => match size {
                chim_semantic::UintSize::U8 => "uint8_t".to_string(),
                chim_semantic::UintSize::U16 => "uint16_t".to_string(),
                chim_semantic::UintSize::U32 => "uint32_t".to_string(),
                chim_semantic::UintSize::U64 => "uint64_t".to_string(),
                chim_semantic::UintSize::U128 => "__uint128".to_string(),
                chim_semantic::UintSize::Usize => "uintptr_t".to_string(),
            },
            chim_semantic::TypeData::Float(size) => match size {
                chim_semantic::FloatSize::F32 => "float".to_string(),
                chim_semantic::FloatSize::F64 => "double".to_string(),
            },
            chim_semantic::TypeData::Bool => "bool".to_string(),
            chim_semantic::TypeData::Char => "char".to_string(),
            chim_semantic::TypeData::String => "const char*".to_string(),
            chim_semantic::TypeData::Pointer(inner, _) => {
                format!("{}*", self.type_to_c(*inner, program))
            }
            chim_semantic::TypeData::Reference(_, inner, _) => {
                format!("{}*", self.type_to_c(*inner, program))
            }
            chim_semantic::TypeData::Array(inner, size) => {
                format!("{}[{}]", self.type_to_c(*inner, program), size)
            }
            chim_semantic::TypeData::Slice(inner) => {
                format!("{}*", self.type_to_c(*inner, program))
            }
            chim_semantic::TypeData::Tuple(elems) => {
                let elem_types: Vec<String> = elems.iter()
                    .map(|ty| self.type_to_c(*ty, program))
                    .collect();
                format!("struct {{ {} }}", elem_types.join("; "))
            }
            chim_semantic::TypeData::Unit => "void".to_string(),
            _ => "void".to_string(),
        }
    }

    fn generate_function(&mut self, func: &IRFunction, program: &AnalyzedProgram) -> String {
        let mut output = String::new();

        let return_type = self.type_to_c(func.return_type, program);
        
        let params: Vec<String> = func.params.iter()
            .map(|p| format!("{} {}", self.type_to_c(p.ty, program), p.name))
            .collect();

        output.push_str(&format!("{} {}(", return_type, func.name));
        output.push_str(&params.join(", "));
        output.push_str(") {\n");

        for block in &func.body {
            output.push_str(&self.generate_block(block, program));
        }

        output.push_str("}\n\n");

        output
    }

    fn generate_block(&mut self, block: &Block, program: &AnalyzedProgram) -> String {
        let mut output = String::new();
        let block_name = self.get_block_name(block.id);

        output.push_str(&format!("{}:\n", block_name));

        for inst in &block.instructions {
            output.push_str(&self.generate_instruction(inst, program));
        }

        output.push_str(&self.generate_terminator(&block.terminator, program));

        output
    }

    fn generate_instruction(&mut self, inst: &IRInst, program: &AnalyzedProgram) -> String {
        match inst {
            IRInst::Alloca { dest, ty, .. } => {
                let var_name = self.get_value_name(*dest);
                let c_type = self.type_to_c(*ty, program);
                format!("    {} {};\n", c_type, var_name)
            }
            IRInst::Load { dest, src, ty, .. } => {
                let dest_name = self.get_value_name(*dest);
                let src_name = self.get_value_name(*src);
                let c_type = self.type_to_c(*ty, program);
                format!("    {} = *{};\n", dest_name, src_name)
            }
            IRInst::Store { dest, src, ty, .. } => {
                let dest_name = self.get_value_name(*dest);
                let src_name = self.get_value_name(*src);
                let c_type = self.type_to_c(*ty, program);
                format!("    *{} = {};\n", dest_name, src_name)
            }
            IRInst::Binary { dest, op, left, right, ty, .. } => {
                let dest_name = self.get_value_name(*dest);
                let left_name = self.get_value_name(*left);
                let right_name = self.get_value_name(*right);
                let c_type = self.type_to_c(*ty, program);
                let op_str = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Rem => "%",
                    BinaryOp::And => "&&",
                    BinaryOp::Or => "||",
                    BinaryOp::Shl => "<<",
                    BinaryOp::Shr => ">>",
                    BinaryOp::FAdd => "+",
                    BinaryOp::FSub => "-",
                    BinaryOp::FMul => "*",
                    BinaryOp::FDiv => "/",
                    BinaryOp::FRem => "%",
                };
                format!("    {} {} = {} {} {};\n", c_type, dest_name, left_name, op_str, right_name)
            }
            IRInst::Unary { dest, op, operand, ty, .. } => {
                let dest_name = self.get_value_name(*dest);
                let operand_name = self.get_value_name(*operand);
                let c_type = self.type_to_c(*ty, program);
                let op_str = match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "!",
                    UnaryOp::FNeg => "-",
                };
                format!("    {} {} = {}{};\n", c_type, dest_name, op_str, operand_name)
            }
            IRInst::Call { dest, func, args, ty, .. } => {
                let func_name = self.get_value_name(*func);
                let args_str: Vec<String> = args.iter()
                    .map(|arg| self.get_value_name(*arg))
                    .collect();
                
                if let Some(dest_id) = dest {
                    let dest_name = self.get_value_name(*dest_id);
                    let c_type = self.type_to_c(*ty, program);
                    format!("    {} {} = {}({});\n", c_type, dest_name, func_name, args_str.join(", "))
                } else {
                    format!("    {}({});\n", func_name, args_str.join(", "))
                }
            }
            IRInst::Br { target, .. } => {
                let target_name = self.get_block_name(*target);
                format!("    goto {};\n", target_name)
            }
            IRInst::CondBr { condition, true_block, false_block, .. } => {
                let cond_name = self.get_value_name(*condition);
                let true_name = self.get_block_name(*true_block);
                let false_name = self.get_block_name(*false_block);
                format!("    if ({}) goto {}; else goto {};\n", cond_name, true_name, false_name)
            }
            IRInst::Ret { value, .. } => {
                if let Some(val) = value {
                    let val_name = self.get_value_name(*val);
                    format!("    return {};\n", val_name)
                } else {
                    "    return;\n".to_string()
                }
            }
            IRInst::RetVoid { .. } => {
                "    return;\n".to_string()
            }
            IRInst::Select { dest, condition, true_val, false_val, ty, .. } => {
                let dest_name = self.get_value_name(*dest);
                let cond_name = self.get_value_name(*condition);
                let true_name = self.get_value_name(*true_val);
                let false_name = self.get_value_name(*false_val);
                let c_type = self.type_to_c(*ty, program);
                format!("    {} = {} ? {} : {};\n", c_type, dest_name, cond_name, true_name, false_name)
            }
            IRInst::GetElementPtr { dest, ptr, indices, ty, .. } => {
                let dest_name = self.get_value_name(*dest);
                let ptr_name = self.get_value_name(*ptr);
                let indices_str: Vec<String> = indices.iter()
                    .map(|idx| self.get_value_name(*idx))
                    .collect();
                let c_type = self.type_to_c(*ty, program);
                format!("    {} = &{}[{}];\n", c_type, dest_name, ptr_name, indices_str.join("]["))
            }
            IRInst::Cast { dest, value, to_ty, op, .. } => {
                let dest_name = self.get_value_name(*dest);
                let value_name = self.get_value_name(*value);
                let to_type = self.type_to_c(*to_ty, program);
                let op_str = match op {
                    chim_ir::CastOp::Trunc => "(int)",
                    chim_ir::CastOp::ZExt => "(int)",
                    chim_ir::CastOp::SExt => "(int)",
                    chim_ir::CastOp::BitCast => "(int)",
                    _ => "(int)",
                };
                format!("    {} = ({}) {};\n", dest_name, to_type, op_str)
            }
            _ => format!("    ; unimplemented instruction\n"),
        }
    }

    fn generate_terminator(&mut self, terminator: &Terminator, program: &AnalyzedProgram) -> String {
        match terminator {
            Terminator::Return(value) => {
                if let Some(val) = value {
                    let val_name = self.get_value_name(*val);
                    format!("    return {};\n", val_name)
                } else {
                    "    return;\n".to_string()
                }
            }
            Terminator::Branch(target) => {
                let target_name = self.get_block_name(*target);
                format!("    goto {};\n", target_name)
            }
            Terminator::ConditionalBranch { condition, true_block, false_block } => {
                let cond_name = self.get_value_name(*condition);
                let true_name = self.get_block_name(*true_block);
                let false_name = self.get_block_name(*false_block);
                format!("    if ({}) goto {}; else goto {};\n", cond_name, true_name, false_name)
            }
            Terminator::Unreachable => {
                "    __builtin_unreachable();\n".to_string()
            }
            _ => format!("    ; unimplemented terminator\n"),
        }
    }
}

impl CodeGenerator for CTranspiler {
    fn generate(&self, module: &IRModule, program: &AnalyzedProgram) -> Result<GeneratedCode, CodegenError> {
        let mut transpiler = CTranspiler::new();
        let mut output = String::new();

        output.push_str("/* C code generation */\n");
        output.push_str("/* Generated by Chim Compiler */\n\n");

        output.push_str("#include <stdio.h>\n");
        output.push_str("#include <stdlib.h>\n");
        output.push_str("#include <stdint.h>\n\n");

        for func in &module.functions {
            output.push_str(&transpiler.generate_function(func, program));
        }

        output.push_str("int main() {\n");
        output.push_str("    printf(\"Hello from Chim!\\n\");\n");
        output.push_str("    return 0;\n");
        output.push_str("}\n");

        Ok(GeneratedCode {
            source: output,
            extension: String::from("c"),
            language: String::from("C"),
            is_executable: true,
        })
    }

    fn name(&self) -> &str {
        "C"
    }

    fn file_extension(&self) -> &str {
        "c"
    }

    fn target(&self) -> CodegenTarget {
        CodegenTarget::C
    }
}

pub struct CppTranspiler;

impl CppTranspiler {
    pub fn new() -> Self {
        CppTranspiler
    }
}

impl CodeGenerator for CppTranspiler {
    fn generate(&self, module: &IRModule, _program: &AnalyzedProgram) -> Result<GeneratedCode, CodegenError> {
        let mut output = String::new();

        output.push_str("// C++ code generation\n");
        output.push_str("// Generated by Chim Compiler\n\n");

        output.push_str("#include <iostream>\n");
        output.push_str("#include <string>\n\n");

        for func in &module.functions {
            output.push_str(&format!("int32_t {}(", func.name));

            let params: Vec<String> = func.params.iter()
                .map(|p| format!("int32_t {}", p.name))
                .collect();
            output.push_str(&params.join(", "));
            output.push_str(") {\n");
            output.push_str("    return 0;\n");
            output.push_str("}\n\n");
        }

        output.push_str("int main() {\n");
        output.push_str("    std::cout << \"Hello from Chim!\" << std::endl;\n");
        output.push_str("    return 0;\n");
        output.push_str("}\n");

        Ok(GeneratedCode {
            source: output,
            extension: String::from("cpp"),
            language: String::from("C++"),
            is_executable: true,
        })
    }

    fn name(&self) -> &str {
        "C++"
    }

    fn file_extension(&self) -> &str {
        "cpp"
    }

    fn target(&self) -> CodegenTarget {
        CodegenTarget::Cpp
    }
}

pub struct RustTranspiler;

impl RustTranspiler {
    pub fn new() -> Self {
        RustTranspiler
    }
}

impl CodeGenerator for RustTranspiler {
    fn generate(&self, module: &IRModule, _program: &AnalyzedProgram) -> Result<GeneratedCode, CodegenError> {
        let mut output = String::new();

        output.push_str("// Rust code generation\n");
        output.push_str("// Generated by Chim Compiler\n\n");

        for func in &module.functions {
            output.push_str(&format!("fn {}(", func.name));

            let params: Vec<String> = func.params.iter()
                .map(|p| format!("{}: i32", p.name))
                .collect();
            output.push_str(&params.join(", "));
            output.push_str(") -> i32 {\n");
            output.push_str("    0\n");
            output.push_str("}\n\n");
        }

        output.push_str("fn main() {\n");
        output.push_str("    println!(\"Hello from Chim!\");\n");
        output.push_str("}\n");

        Ok(GeneratedCode {
            source: output,
            extension: String::from("rs"),
            language: String::from("Rust"),
            is_executable: true,
        })
    }

    fn name(&self) -> &str {
        "Rust"
    }

    fn file_extension(&self) -> &str {
        "rs"
    }

    fn target(&self) -> CodegenTarget {
        CodegenTarget::Rust
    }
}

pub struct GoTranspiler;

impl GoTranspiler {
    pub fn new() -> Self {
        GoTranspiler
    }
}

impl CodeGenerator for GoTranspiler {
    fn generate(&self, module: &IRModule, _program: &AnalyzedProgram) -> Result<GeneratedCode, CodegenError> {
        let mut output = String::new();

        output.push_str("// Go code generation\n");
        output.push_str("// Generated by Chim Compiler\n\n");

        output.push_str("package main\n\n");
        output.push_str("import \"fmt\"\n\n");

        for func in &module.functions {
            output.push_str(&format!("func {}(", func.name));

            let params: Vec<String> = func.params.iter()
                .map(|p| format!("{} int32", p.name))
                .collect();
            output.push_str(&params.join(", "));
            output.push_str(") int32 {\n");
            output.push_str("    return 0\n");
            output.push_str("}\n\n");
        }

        output.push_str("func main() {\n");
        output.push_str("    fmt.Println(\"Hello from Chim!\")\n");
        output.push_str("}\n");

        Ok(GeneratedCode {
            source: output,
            extension: String::from("go"),
            language: String::from("Go"),
            is_executable: true,
        })
    }

    fn name(&self) -> &str {
        "Go"
    }

    fn file_extension(&self) -> &str {
        "go"
    }

    fn target(&self) -> CodegenTarget {
        CodegenTarget::Go
    }
}

pub struct PythonTranspiler;

impl PythonTranspiler {
    pub fn new() -> Self {
        PythonTranspiler
    }
}

impl CodeGenerator for PythonTranspiler {
    fn generate(&self, module: &IRModule, _program: &AnalyzedProgram) -> Result<GeneratedCode, CodegenError> {
        let mut output = String::new();

        output.push_str("# Python code generation\n");
        output.push_str("# Generated by Chim Compiler\n\n");

        for func in &module.functions {
            output.push_str(&format!("def {}(", func.name));

            let params: Vec<String> = func.params.iter()
                .map(|p| p.name.clone())
                .collect();
            output.push_str(&params.join(", "));
            output.push_str("):\n");
            output.push_str("    return 0\n\n");
        }

        output.push_str("def main():\n");
        output.push_str("    print(\"Hello from Chim!\")\n\n");
        output.push_str("if __name__ == \"__main__\":\n");
        output.push_str("    main()\n");

        Ok(GeneratedCode {
            source: output,
            extension: String::from("py"),
            language: String::from("Python"),
            is_executable: true,
        })
    }

    fn name(&self) -> &str {
        "Python"
    }

    fn file_extension(&self) -> &str {
        "py"
    }

    fn target(&self) -> CodegenTarget {
        CodegenTarget::Python
    }
}

pub struct JavaScriptTranspiler;

impl JavaScriptTranspiler {
    pub fn new() -> Self {
        JavaScriptTranspiler
    }
}

impl CodeGenerator for JavaScriptTranspiler {
    fn generate(&self, module: &IRModule, _program: &AnalyzedProgram) -> Result<GeneratedCode, CodegenError> {
        let mut output = String::new();

        output.push_str("// JavaScript code generation\n");
        output.push_str("// Generated by Chim Compiler\n\n");

        for func in &module.functions {
            output.push_str(&format!("function {}(", func.name));

            let params: Vec<String> = func.params.iter()
                .map(|p| p.name.clone())
                .collect();
            output.push_str(&params.join(", "));
            output.push_str(") {\n");
            output.push_str("    return 0;\n");
            output.push_str("}\n\n");
        }

        output.push_str("function main() {\n");
        output.push_str("    console.log(\"Hello from Chim!\");\n");
        output.push_str("}\n\n");
        output.push_str("main();\n");

        Ok(GeneratedCode {
            source: output,
            extension: String::from("js"),
            language: String::from("JavaScript"),
            is_executable: true,
        })
    }

    fn name(&self) -> &str {
        "JavaScript"
    }

    fn file_extension(&self) -> &str {
        "js"
    }

    fn target(&self) -> CodegenTarget {
        CodegenTarget::JavaScript
    }
}

pub struct TypeScriptTranspiler;

impl TypeScriptTranspiler {
    pub fn new() -> Self {
        TypeScriptTranspiler
    }
}

impl CodeGenerator for TypeScriptTranspiler {
    fn generate(&self, module: &IRModule, _program: &AnalyzedProgram) -> Result<GeneratedCode, CodegenError> {
        let mut output = String::new();

        output.push_str("// TypeScript code generation\n");
        output.push_str("// Generated by Chim Compiler\n\n");

        for func in &module.functions {
            output.push_str(&format!("function {}(", func.name));

            let params: Vec<String> = func.params.iter()
                .map(|p| format!("{}: number", p.name))
                .collect();
            output.push_str(&params.join(", "));
            output.push_str("): number {\n");
            output.push_str("    return 0;\n");
            output.push_str("}\n\n");
        }

        output.push_str("function main(): void {\n");
        output.push_str("    console.log(\"Hello from Chim!\");\n");
        output.push_str("}\n\n");
        output.push_str("main();\n");

        Ok(GeneratedCode {
            source: output,
            extension: String::from("ts"),
            language: String::from("TypeScript"),
            is_executable: true,
        })
    }

    fn name(&self) -> &str {
        "TypeScript"
    }

    fn file_extension(&self) -> &str {
        "ts"
    }

    fn target(&self) -> CodegenTarget {
        CodegenTarget::TypeScript
    }
}

pub struct JavaTranspiler;

impl JavaTranspiler {
    pub fn new() -> Self {
        JavaTranspiler
    }
}

impl CodeGenerator for JavaTranspiler {
    fn generate(&self, module: &IRModule, _program: &AnalyzedProgram) -> Result<GeneratedCode, CodegenError> {
        let mut output = String::new();

        output.push_str("// Java code generation\n");
        output.push_str("// Generated by Chim Compiler\n\n");

        output.push_str("public class Main {\n");

        for func in &module.functions {
            output.push_str(&format!("    static int {}(", func.name));

            let params: Vec<String> = func.params.iter()
                .map(|p| format!("int {}", p.name))
                .collect();
            output.push_str(&params.join(", "));
            output.push_str(") {\n");
            output.push_str("        return 0;\n");
            output.push_str("    }\n\n");
        }

        output.push_str("    public static void main(String[] args) {\n");
        output.push_str("        System.out.println(\"Hello from Chim!\");\n");
        output.push_str("    }\n");
        output.push_str("}\n");

        Ok(GeneratedCode {
            source: output,
            extension: String::from("java"),
            language: String::from("Java"),
            is_executable: true,
        })
    }

    fn name(&self) -> &str {
        "Java"
    }

    fn file_extension(&self) -> &str {
        "java"
    }

    fn target(&self) -> CodegenTarget {
        CodegenTarget::Java
    }
}

pub struct CSharpTranspiler;

impl CSharpTranspiler {
    pub fn new() -> Self {
        CSharpTranspiler
    }
}

impl CodeGenerator for CSharpTranspiler {
    fn generate(&self, module: &IRModule, _program: &AnalyzedProgram) -> Result<GeneratedCode, CodegenError> {
        let mut output = String::new();

        output.push_str("// C# code generation\n");
        output.push_str("// Generated by Chim Compiler\n\n");

        output.push_str("using System;\n\n");
        output.push_str("class Main {\n");

        for func in &module.functions {
            output.push_str(&format!("    static int {}(", func.name));

            let params: Vec<String> = func.params.iter()
                .map(|p| format!("int {}", p.name))
                .collect();
            output.push_str(&params.join(", "));
            output.push_str(") {\n");
            output.push_str("        return 0;\n");
            output.push_str("    }\n\n");
        }

        output.push_str("    static void Main() {\n");
        output.push_str("        Console.WriteLine(\"Hello from Chim!\");\n");
        output.push_str("    }\n");
        output.push_str("}\n");

        Ok(GeneratedCode {
            source: output,
            extension: String::from("cs"),
            language: String::from("C#"),
            is_executable: true,
        })
    }

    fn name(&self) -> &str {
        "C#"
    }

    fn file_extension(&self) -> &str {
        "cs"
    }

    fn target(&self) -> CodegenTarget {
        CodegenTarget::CSharp
    }
}

pub struct WasmTranspiler;

impl WasmTranspiler {
    pub fn new() -> Self {
        WasmTranspiler
    }
}

impl CodeGenerator for WasmTranspiler {
    fn generate(&self, module: &IRModule, _program: &AnalyzedProgram) -> Result<GeneratedCode, CodegenError> {
        let mut output = String::new();

        output.push_str("; WebAssembly code generation\n");
        output.push_str("; Generated by Chim Compiler\n\n");

        output.push_str("(module\n");
        output.push_str("  (func $main (result i32)\n");
        output.push_str("    i32.const 0\n");
        output.push_str("  )\n");
        output.push_str("  (export \"main\" (func $main))\n");
        output.push_str(")\n");

        Ok(GeneratedCode {
            source: output,
            extension: String::from("wat"),
            language: String::from("WebAssembly Text"),
            is_executable: false,
        })
    }

    fn name(&self) -> &str {
        "WebAssembly"
    }

    fn file_extension(&self) -> &str {
        "wat"
    }

    fn target(&self) -> CodegenTarget {
        CodegenTarget::Wasm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_transpiler() {
        let transpiler = CTranspiler::new();
        assert_eq!(transpiler.name(), "C");
        assert_eq!(transpiler.file_extension(), "c");
    }

    #[test]
    fn test_cpp_transpiler() {
        let transpiler = CppTranspiler::new();
        assert_eq!(transpiler.name(), "C++");
        assert_eq!(transpiler.file_extension(), "cpp");
    }

    #[test]
    fn test_python_transpiler() {
        let transpiler = PythonTranspiler::new();
        assert_eq!(transpiler.name(), "Python");
        assert_eq!(transpiler.file_extension(), "py");
    }
}
