use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CType {
    CVoid,
    CChar,
    CShort,
    CInt,
    CLong,
    CLongLong,
    CUChar,
    CUShort,
    CUInt,
    CULong,
    CULongLong,
    CFloat,
    CDouble,
    CPointer(Box<CType>),
    CConstPointer(Box<CType>),
    CStruct(Vec<(String, CType)>),
    CVoidPtr,
    CString,
}

#[derive(Debug)]
pub struct CFFIGenerator {
    type_mapping: HashMap<String, CType>,
    c_declarations: Vec<String>,
}

impl CFFIGenerator {
    pub fn new() -> Self {
        let mut generator = Self {
            type_mapping: HashMap::new(),
            c_declarations: Vec::new(),
        };
        generator.init_type_mapping();
        generator
    }

    fn init_type_mapping(&mut self) {
        self.type_mapping.insert("c_char".to_string(), CType::CChar);
        self.type_mapping.insert("c_int".to_string(), CType::CInt);
        self.type_mapping.insert("c_long".to_string(), CType::CLong);
        self.type_mapping.insert("c_short".to_string(), CType::CShort);
        self.type_mapping.insert("c_longlong".to_string(), CType::CLongLong);
        self.type_mapping.insert("c_uchar".to_string(), CType::CUChar);
        self.type_mapping.insert("c_uint".to_string(), CType::CUInt);
        self.type_mapping.insert("c_ulong".to_string(), CType::CULong);
        self.type_mapping.insert("c_ulonglong".to_string(), CType::CULongLong);
        self.type_mapping.insert("c_float".to_string(), CType::CFloat);
        self.type_mapping.insert("c_double".to_string(), CType::CDouble);
        self.type_mapping.insert("c_voidptr".to_string(), CType::CVoidPtr);
        self.type_mapping.insert("c_string".to_string(), CType::CString);
    }

    pub fn generate_c_header(&self, declarations: &[crate::Function]) -> String {
        let mut output = String::new();
        output.push_str("#ifndef CHIM_CFFI_H\n");
        output.push_str("#define CHIM_CFFI_H\n\n");
        output.push_str("#include <stdint.h>\n");
        output.push_str("#include <stdbool.h>\n");
        output.push_str("#include <stddef.h>\n\n");
        
        for decl in declarations {
            self.generate_function_declaration(decl, &mut output);
        }
        
        output.push_str("\n#endif /* CHIM_CFFI_H */\n");
        output
    }

    fn generate_function_declaration(&self, fun: &crate::Function, output: &mut String) {
        if fun.is_extern && fun.name.starts_with(b"c_") {
            let c_name = String::from_utf8_lossy(&fun.name[2..]).to_string();
            let return_type = self.map_type_to_c(fun.return_type);
            
            output.push_str(&format!("{} {}(", return_type, c_name));
            
            let mut first = true;
            for param in &fun.params {
                if !first {
                    output.push_str(", ");
                }
                first = false;
                
                let param_type = self.compiler_get_variable_type(param.var_id);
                let c_type = self.map_type_to_c(param_type);
                let param_name = String::from_utf8_lossy(self.compiler_get_variable_name(param.var_id));
                
                output.push_str(&format!("{} {}", c_type, param_name));
            }
            
            output.push_str(");\n");
        }
    }

    pub fn map_type_to_c(&self, type_id: crate::TypeId) -> String {
        match self.compiler_get_type(type_id) {
            crate::Type::CVoid => "void".to_string(),
            crate::Type::CChar => "char".to_string(),
            crate::Type::CShort => "short".to_string(),
            crate::Type::CInt => "int".to_string(),
            crate::Type::CLong => "long".to_string(),
            crate::Type::CLongLong => "long long".to_string(),
            crate::Type::CUChar => "unsigned char".to_string(),
            crate::Type::CUShort => "unsigned short".to_string(),
            crate::Type::CUInt => "unsigned int".to_string(),
            crate::Type::CULong => "unsigned long".to_string(),
            crate::Type::CULongLong => "unsigned long long".to_string(),
            crate::Type::CFloat => "float".to_string(),
            crate::Type::CDouble => "double".to_string(),
            crate::Type::CVoidPtr => "void*".to_string(),
            crate::Type::CString => "const char*".to_string(),
            crate::Type::CPointer(inner) => {
                format!("{}*", self.map_type_to_c(*inner))
            }
            crate::Type::CConstPointer(inner) => {
                format!("const {}*", self.map_type_to_c(*inner))
            }
            crate::Type::ISize => "intptr_t".to_string(),
            crate::Type::USize => "uintptr_t".to_string(),
            _ => "void*".to_string(),
        }
    }

    pub fn generate_c_call(&self, fun: &crate::Function, args: &[NodeId]) -> String {
        if fun.is_extern && fun.name.starts_with(b"c_") {
            let c_name = String::from_utf8_lossy(&fun.name[2..]).to_string();
            format!("{}({})", c_name, args.len())
        } else {
            format!("function_{}", 0)
        }
    }

    fn compiler_get_variable_type(&self, var_id: crate::VarId) -> crate::TypeId {
        crate::TypeId(0)
    }

    fn compiler_get_variable_name(&self, var_id: crate::VarId) -> &[u8] {
        b"var"
    }

    fn compiler_get_type(&self, type_id: crate::TypeId) -> &crate::Type {
        &crate::Type::CVoid
    }
}

pub fn generate_c_ffi(compiler: &crate::Compiler) -> CFFIGenerator {
    let mut generator = CFFIGenerator::new();
    generator
}

pub fn map_chim_type_to_c(chim_type: &crate::Type) -> String {
    match chim_type {
        crate::Type::CVoid => "void".to_string(),
        crate::Type::CBool => "bool".to_string(),
        crate::Type::CChar => "char".to_string(),
        crate::Type::CShort => "short".to_string(),
        crate::Type::CUShort => "unsigned short".to_string(),
        crate::Type::CInt => "int".to_string(),
        crate::Type::CUInt => "unsigned int".to_string(),
        crate::Type::CLong => "long".to_string(),
        crate::Type::CULong => "unsigned long".to_string(),
        crate::Type::CLongLong => "long long".to_string(),
        crate::Type::CULongLong => "unsigned long long".to_string(),
        crate::Type::CFloat => "float".to_string(),
        crate::Type::CDouble => "double".to_string(),
        crate::Type::CStr => "const char*".to_string(),
        crate::Type::CVoidPtr => "void*".to_string(),
        crate::Type::ISize => "intptr_t".to_string(),
        crate::Type::USize => "uintptr_t".to_string(),
        crate::Type::Pointer { target, .. } => {
            format!("{}*", map_chim_type_to_c(target))
        }
        crate::Type::Function { params, ret, .. } => {
            let params_str: Vec<String> = params.iter()
                .map(|t| map_chim_type_to_c(t))
                .collect();
            let ret_str = map_chim_type_to_c(ret);
            format!("{} (*)({})", ret_str, params_str.join(", "))
        }
        crate::Type::Struct { name, .. } => {
            format!("struct {}", String::from_utf8_lossy(name))
        }
        _ => "void*".to_string(),
    }
}
