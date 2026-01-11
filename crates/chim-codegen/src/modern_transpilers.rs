use crate::{CodeGenerator, CodegenError, CodegenTarget, GeneratedCode};
use chim_ir::IRModule;
use chim_semantic::AnalyzedProgram;

pub struct SwiftTranspiler;

impl SwiftTranspiler {
    pub fn new() -> Self {
        SwiftTranspiler
    }

    fn generate_type(&self, ty: &str) -> String {
        match ty {
            "i32" | "Int32" => "Int32".to_string(),
            "i64" | "Int64" => "Int64".to_string(),
            "f32" | "Float32" => "Float".to_string(),
            "f64" | "Float64" => "Double".to_string(),
            "bool" | "Bool" => "Bool".to_string(),
            "str" | "String" => "String".to_string(),
            "void" | "()" => "Void".to_string(),
            _ => "Any".to_string(),
        }
    }

    fn generate_function(&self, func: &IRFunction) -> String {
        let mut output = String::new();

        let params: Vec<String> = func
            .params
            .iter()
            .map(|(name, ty)| format!("{}: {}", name, self.generate_type(ty)))
            .collect();

        let return_type = if func.return_type == "void" || func.return_type.is_empty() {
            String::new()
        } else {
            format!(" -> {}", self.generate_type(&func.return_type))
        };

        output.push_str(&format!(
            "func {}({}){} {{\n",
            func.name,
            params.join(", "),
            return_type
        ));
        output.push_str("}\n\n");

        output
    }
}

impl CodeGenerator for SwiftTranspiler {
    fn generate(
        &self,
        module: &IRModule,
        _program: &AnalyzedProgram,
    ) -> Result<GeneratedCode, CodegenError> {
        let mut output = String::new();

        output.push_str("// Swift 代码生成\n");
        output.push_str("// 由 Chim 编译器生成\n\n");

        output.push_str("import Foundation\n\n");

        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }

        output.push_str("main()\n");

        Ok(GeneratedCode {
            source: output,
            extension: String::from("swift"),
            language: String::from("Swift"),
            is_executable: true,
        })
    }

    fn name(&self) -> &str {
        "Swift"
    }

    fn file_extension(&self) -> &str {
        "swift"
    }

    fn target(&self) -> CodegenTarget {
        CodegenTarget::Swift
    }
}

pub struct KotlinTranspiler;

impl KotlinTranspiler {
    pub fn new() -> Self {
        KotlinTranspiler
    }

    fn generate_type(&self, ty: &str) -> String {
        match ty {
            "i32" | "Int32" => "Int".to_string(),
            "i64" | "Int64" => "Long".to_string(),
            "f32" | "Float32" => "Float".to_string(),
            "f64" | "Float64" => "Double".to_string(),
            "bool" | "Bool" => "Boolean".to_string(),
            "str" | "String" => "String".to_string(),
            "void" | "()" => "Unit".to_string(),
            _ => "Any?".to_string(),
        }
    }

    fn generate_function(&self, func: &IRFunction) -> String {
        let mut output = String::new();

        let params: Vec<String> = func
            .params
            .iter()
            .map(|(name, ty)| format!("{}: {}", name, self.generate_type(ty)))
            .collect();

        let return_type = if func.return_type == "void" || func.return_type.is_empty() {
            String::new()
        } else {
            format!(": {}", self.generate_type(&func.return_type))
        };

        output.push_str(&format!(
            "fun {}({}){} {{\n",
            func.name,
            params.join(", "),
            return_type
        ));
        output.push_str("}\n\n");

        output
    }
}

impl CodeGenerator for KotlinTranspiler {
    fn generate(
        &self,
        module: &IRModule,
        _program: &AnalyzedProgram,
    ) -> Result<GeneratedCode, CodegenError> {
        let mut output = String::new();

        output.push_str("// Kotlin 代码生成\n");
        output.push_str("// 由 Chim 编译器生成\n\n");

        output.push_str("package chim.generated\n\n");

        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }

        output.push_str("fun main(args: Array<String>) {\n");
        output.push_str("    main()\n");
        output.push_str("}\n");

        Ok(GeneratedCode {
            source: output,
            extension: String::from("kt"),
            language: String::from("Kotlin"),
            is_executable: true,
        })
    }

    fn name(&self) -> &str {
        "Kotlin"
    }

    fn file_extension(&self) -> &str {
        "kt"
    }

    fn target(&self) -> CodegenTarget {
        CodegenTarget::Kotlin
    }
}

pub struct MojoTranspiler;

impl MojoTranspiler {
    pub fn new() -> Self {
        MojoTranspiler
    }

    fn generate_type(&self, ty: &str) -> String {
        match ty {
            "i32" | "Int32" | "i64" | "Int64" => "Int".to_string(),
            "f32" | "Float32" => "Float32".to_string(),
            "f64" | "Float64" => "Float64".to_string(),
            "bool" | "Bool" => "Bool".to_string(),
            "str" | "String" => "String".to_string(),
            "void" | "()" => "None".to_string(),
            _ => "Float32".to_string(),
        }
    }

    fn generate_function(&self, func: &IRFunction) -> String {
        let mut output = String::new();

        let params: Vec<String> = func
            .params
            .iter()
            .map(|(name, ty)| format!("{}: {}", name, self.generate_type(ty)))
            .collect();

        output.push_str(&format!(
            "fn {}({}) -> {}:\n",
            func.name,
            params.join(", "),
            self.generate_type(&func.return_type)
        ));

        output.push_str("    pass\n\n");

        output
    }
}

impl CodeGenerator for MojoTranspiler {
    fn generate(
        &self,
        module: &IRModule,
        _program: &AnalyzedProgram,
    ) -> Result<GeneratedCode, CodegenError> {
        let mut output = String::new();

        output.push_str("# Mojo 代码生成\n");
        output.push_str("# 由 Chim 编译器生成\n\n");

        output.push_str("from memory import UnsafePointer\n");
        output.push_str("from math import sqrt, sin, cos, exp, log\n\n");

        output.push_str("# CPU 函数\n");
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }

        output.push_str("# 主函数\n");
        output.push_str("fn main():\n");
        output.push_str("    print(\"Chim Compiler - Mojo Backend\")\n");

        Ok(GeneratedCode {
            source: output,
            extension: String::from("mojo"),
            language: String::from("Mojo"),
            is_executable: true,
        })
    }

    fn name(&self) -> &str {
        "Mojo"
    }

    fn file_extension(&self) -> &str {
        "mojo"
    }

    fn target(&self) -> CodegenTarget {
        CodegenTarget::Mojo
    }
}

pub struct MoonBitTranspiler;

impl MoonBitTranspiler {
    pub fn new() -> Self {
        MoonBitTranspiler
    }

    fn generate_type(&self, ty: &str) -> String {
        match ty {
            "i32" | "Int32" => "Int".to_string(),
            "i64" | "Int64" => "Int64".to_string(),
            "f32" | "Float32" => "Float".to_string(),
            "f64" | "Float64" => "Double".to_string(),
            "bool" | "Bool" => "Bool".to_string(),
            "str" | "String" => "String".to_string(),
            "void" | "()" => "Unit".to_string(),
            _ => "Any".to_string(),
        }
    }

    fn generate_function(&self, func: &IRFunction) -> String {
        let mut output = String::new();

        let params: Vec<String> = func
            .params
            .iter()
            .map(|(name, ty)| format!("{} : {}", name, self.generate_type(ty)))
            .collect();

        let return_type = self.generate_type(&func.return_type);

        output.push_str(&format!(
            "fn {}({}) -> {} {{\n",
            func.name,
            params.join(", "),
            return_type
        ));
        output.push_str("}\n\n");

        output
    }
}

impl CodeGenerator for MoonBitTranspiler {
    fn generate(
        &self,
        module: &IRModule,
        _program: &AnalyzedProgram,
    ) -> Result<GeneratedCode, CodegenError> {
        let mut output = String::new();

        output.push_str("// MoonBit 代码生成\n");
        output.push_str("// 由 Chim 编译器生成\n\n");

        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }

        Ok(GeneratedCode {
            source: output,
            extension: String::from("mbt"),
            language: String::from("MoonBit"),
            is_executable: true,
        })
    }

    fn name(&self) -> &str {
        "MoonBit"
    }

    fn file_extension(&self) -> &str {
        "mbt"
    }

    fn target(&self) -> CodegenTarget {
        CodegenTarget::MoonBit
    }
}

pub struct PythonTranspiler;

impl PythonTranspiler {
    pub fn new() -> Self {
        PythonTranspiler
    }

    fn generate_type(&self, ty: &str) -> String {
        match ty {
            "i32" | "Int32" | "i64" | "Int64" => "int".to_string(),
            "f32" | "Float32" | "f64" | "Float64" => "float".to_string(),
            "bool" | "Bool" => "bool".to_string(),
            "str" | "String" => "str".to_string(),
            "void" | "()" => "None".to_string(),
            _ => "Any".to_string(),
        }
    }

    fn generate_function(&self, func: &IRFunction) -> String {
        let mut output = String::new();

        let params: Vec<String> = func
            .params
            .iter()
            .map(|(name, ty)| format!("{}: {}", name, self.generate_type(ty)))
            .collect();

        let return_type = self.generate_type(&func.return_type);

        output.push_str(&format!(
            "def {}({}) -> {}:\n",
            func.name,
            params.join(", "),
            return_type
        ));
        output.push_str("    pass\n\n");

        output
    }
}

impl CodeGenerator for PythonTranspiler {
    fn generate(
        &self,
        module: &IRModule,
        _program: &AnalyzedProgram,
    ) -> Result<GeneratedCode, CodegenError> {
        let mut output = String::new();

        output.push_str("# Python 代码生成\n");
        output.push_str("# 由 Chim 编译器生成\n\n");

        output.push_str("from typing import Any\n\n");

        for func in &module.functions {
            output.push_str(&self.generate_function(func));
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

pub struct GoTranspiler;

impl GoTranspiler {
    pub fn new() -> Self {
        GoTranspiler
    }

    fn generate_type(&self, ty: &str) -> String {
        match ty {
            "i32" | "Int32" => "int32".to_string(),
            "i64" | "Int64" => "int64".to_string(),
            "f32" | "Float32" => "float32".to_string(),
            "f64" | "Float64" => "float64".to_string(),
            "bool" | "Bool" => "bool".to_string(),
            "str" | "String" => "string".to_string(),
            "void" | "()" => String::new(),
            _ => "interface{}".to_string(),
        }
    }

    fn generate_function(&self, func: &IRFunction) -> String {
        let mut output = String::new();

        let params: Vec<String> = func
            .params
            .iter()
            .map(|(name, ty)| format!("{} {}", name, self.generate_type(ty)))
            .collect();

        let ret_type = self.generate_type(&func.return_type);
        if !ret_type.is_empty() {
            output.push_str(&format!(
                "func {}({}) {} {{\n",
                func.name,
                params.join(", "),
                ret_type
            ));
        } else {
            output.push_str(&format!("func {}({}) {{\n", func.name, params.join(", ")));
        }

        output.push_str("}\n\n");

        output
    }
}

impl CodeGenerator for GoTranspiler {
    fn generate(
        &self,
        module: &IRModule,
        _program: &AnalyzedProgram,
    ) -> Result<GeneratedCode, CodegenError> {
        let mut output = String::new();

        output.push_str("// Go 代码生成\n");
        output.push_str("// 由 Chim 编译器生成\n\n");

        output.push_str("package main\n\n");
        output.push_str("import \"fmt\"\n\n");

        for func in &module.functions {
            output.push_str(&self.generate_function(func));
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
