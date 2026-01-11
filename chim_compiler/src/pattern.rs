// ==================== 穷尽性模式匹配检查器 ====================
// 参考 Agda 实现，支持穷尽性检查、模式覆盖分析

pub mod pattern {
    use crate::stdlib::prelude::{Option, Result, Vec, HashMap, HashSet, String, StringBuilder};
    use crate::stdlib::string::String as StdString;

    // ==================== 模式类型 ====================

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Pattern {
        // 变量模式
        Variable(String),           // x
        Wildcard,                   // _
        
        // 字面量模式
        Bool(bool),
        Int(i128),
        Float(f64),
        Char(char),
        String(StdString),
        Bytes(Vec<u8>),
        
        // 构造器模式
        Constructor {
            name: string,
            args: Vec<Pattern>,
        },
        
        // 元组模式
        Tuple(Vec<Pattern>),
        
        // 数组模式
        Array(Vec<Pattern>, Option<int>),  // 长度固定或可变
        
        // -record模式
        Record {
            name: string,
            fields: Vec<(string, Pattern)>,
        },
        
        // 命名构造器（用于避免歧义）
        NamedConstructor {
            enum_name: string,
            variant_name: string,
            args: Vec<Pattern>,
        },
        
        // 或模式
        Or(Vec<Pattern>),
        
        // 异步/延迟模式
        Async(Pattern),
        
        // 类型约束模式
        TypeAs {
            pattern: Box<Pattern>,
            type_: Type,
        },
    }

    // ==================== 类型系统 ====================

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Type {
        Unit,
        Bool,
        Int(Option<usize>),
        Float,
        Char,
        String,
        Bytes,
        Tuple(Vec<Type>),
        Array(Box<Type>, Option<int>),
        Named(string),
        Enum(string, Vec<VariantDef>),
        Record(string, Vec<FieldDef>),
        Function(Box<Type>, Box<Type>),
        Var(string),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct VariantDef {
        pub name: string,
        pub fields: Vec<Type>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FieldDef {
        pub name: string,
        pub ty: Type,
        pub optional: bool,
    }

    // ==================== 构造器信息 ====================

    #[derive(Debug, Clone)]
    pub struct ConstructorInfo {
        pub name: string,
        pub enum_name: Option<string>,
        pub arity: int,
        pub field_types: Vec<Type>,
        pub is_record: bool,
        pub fields: Vec<string>,
    }

    // ==================== 构造器表 ====================

    #[derive(Debug, Clone)]
    pub struct ConstructorTable {
        constructors: HashMap<string, ConstructorInfo>,
        enum_variants: HashMap<string, Vec<string>>,  // enum_name -> variants
    }

    impl ConstructorTable {
        pub fn new() -> Self {
            ConstructorTable {
                constructors: HashMap::new(),
                enum_variants: HashMap::new(),
            }
        }

        pub fn register_enum(&mut self, name: string, variants: Vec<VariantDef>) {
            let variant_names: Vec<string> = variants.iter()
                .map(|v| v.name.clone())
                .collect();
            
            for (i, variant) in variants.into_iter().enumerate() {
                let full_name = format!("{}::{}", name, variant.name);
                self.constructors.insert(full_name.clone(), ConstructorInfo {
                    name: variant.name.clone(),
                    enum_name: Some(name.clone()),
                    arity: variant.fields.len() as int,
                    field_types: variant.fields,
                    is_record: false,
                    fields: Vec::new(),
                });
                
                // 也注册短名称
                if i == 0 {
                    self.constructors.insert(name.clone(), ConstructorInfo {
                        name: variant.name.clone(),
                        enum_name: Some(name.clone()),
                        arity: variant.fields.len() as int,
                        field_types: variant.fields,
                        is_record: false,
                        fields: Vec::new(),
                    });
                }
            }
            
            self.enum_variants.insert(name.clone(), variant_names);
        }

        pub fn register_record(&mut self, name: string, fields: Vec<FieldDef>) {
            self.constructors.insert(name.clone(), ConstructorInfo {
                name: name.clone(),
                enum_name: None,
                arity: fields.len() as int,
                field_types: fields.iter().map(|f| f.ty.clone()).collect(),
                is_record: true,
                fields: fields.iter().map(|f| f.name.clone()).collect(),
            });
        }

        pub fn get_constructor(&self, name: &string) -> Option<&ConstructorInfo> {
            self.constructors.get(name)
        }

        pub fn get_variants(&self, enum_name: &string) -> Option<&Vec<string>> {
            self.enum_variants.get(enum_name)
        }
    }

    // ==================== 穷尽性检查器 ====================

    pub struct ExhaustivenessChecker {
        constructors: ConstructorTable,
        uncovered: Vec<Pattern>,
        patterns: Vec<Pattern>,
    }

    impl ExhaustivenessChecker {
        pub fn new() -> Self {
            ExhaustivenessChecker {
                constructors: ConstructorTable::new(),
                uncovered: Vec::new(),
                patterns: Vec::new(),
            }
        }

        pub fn with_constructors(mut self, constructors: ConstructorTable) -> Self {
            self.constructors = constructors;
            self
        }

        /// 检查模式匹配是否穷尽
        pub fn check_exhaustive(
            &mut self,
            ty: &Type,
            patterns: &[Pattern],
        ) -> Result<ExhaustivenessResult, ExhaustivenessError> {
            // 1. 生成所有可能的构造器
            let all_constructors = self.all_constructors(ty)?;
            
            // 2. 计算已覆盖的模式
            let covered = self.covered_constructors(patterns);
            
            // 3. 找出未覆盖的模式
            self.uncovered = all_constructors.iter()
                .filter(|c| !covered.contains(c))
                .cloned()
                .collect();
            
            // 4. 生成遗漏报告
            if self.uncovered.is_empty() {
                Ok(ExhaustivenessResult {
                    is_exhaustive: true,
                    uncovered_patterns: Vec::new(),
                    warnings: Vec::new(),
                })
            } else {
                Ok(ExhaustivenessResult {
                    is_exhaustive: false,
                    uncovered_patterns: self.uncovered.clone(),
                    warnings: vec![format!(
                        "Non-exhaustive patterns! Missing {} case(s)",
                        self.uncovered.len()
                    )],
                })
            }
        }

        /// 生成所有可能的构造器
        fn all_constructors(&self, ty: &Type) -> Result<Vec<Pattern>, ExhaustivenessError> {
            match ty {
                Type::Bool => Ok(vec![
                    Pattern::Bool(true),
                    Pattern::Bool(false),
                ]),
                
                Type::Int(_) => Ok(vec![
                    Pattern::Wildcard,  // 整数无法穷尽，用通配符
                ]),
                
                Type::Float => Ok(vec![
                    Pattern::Wildcard,
                ]),
                
                Type::Char => Ok(vec![
                    Pattern::Wildcard,  // 字符无法穷尽
                ]),
                
                Type::String => Ok(vec![
                    Pattern::Wildcard,
                ]),
                
                Type::Unit => Ok(vec![
                    Pattern::Tuple(Vec::new()),
                ]),
                
                Type::Tuple(types) => {
                    let mut result = Vec::new();
                    // 元组的构造器是唯一的
                    result.push(Pattern::Tuple(
                        types.iter().map(|_| Pattern::Wildcard).collect()
                    ));
                    Ok(result)
                }
                
                Type::Array(_, Some(len)) => {
                    // 固定长度数组
                    let mut patterns = Vec::new();
                    for _ in 0..*len {
                        patterns.push(Pattern::Wildcard);
                    }
                    Ok(vec![Pattern::Array(patterns, Some(*len))])
                }
                
                Type::Array(_, None) => {
                    Ok(vec![Pattern::Array(Vec::new(), None)])
                }
                
                Type::Named(name) => {
                    if let Some(info) = self.constructors.get_constructor(name) {
                        self.all_constructors_for_constructor(info, name)
                    } else {
                        // 可能是类型变量
                        Ok(vec![Pattern::Wildcard])
                    }
                }
                
                Type::Enum(enum_name, _) => {
                    self.all_constructors_for_enum(enum_name)
                }
                
                Type::Record(name, _) => {
                    self.all_constructors_for_constructor(
                        self.constructors.get_constructor(name)?,
                        name
                    )
                }
                
                Type::Function(_, _) => Ok(vec![Pattern::Wildcard]),
                
                Type::Var(_) => Ok(vec![Pattern::Wildcard]),
            }
        }

        fn all_constructors_for_enum(&self, enum_name: &string) -> Result<Vec<Pattern>, ExhaustivenessError> {
            if let Some(variants) = self.constructors.get_variants(enum_name) {
                let mut result = Vec::new();
                for variant in variants {
                    let full_name = format!("{}::{}", enum_name, variant);
                    if let Some(info) = self.constructors.get_constructor(&full_name) {
                        let args: Vec<Pattern> = (0..info.arity)
                            .map(|_| Pattern::Wildcard)
                            .collect();
                        
                        result.push(Pattern::NamedConstructor {
                            enum_name: enum_name.clone(),
                            variant_name: variant.clone(),
                            args,
                        });
                    }
                }
                Ok(result)
            } else {
                Err(ExhaustivenessError::UnknownType {
                    type_name: enum_name.clone(),
                })
            }
        }

        fn all_constructors_for_constructor(
            &self,
            info: &ConstructorInfo,
            name: &string,
        ) -> Result<Vec<Pattern>, ExhaustivenessError> {
            let args: Vec<Pattern> = (0..info.arity)
                .map(|_| Pattern::Wildcard)
                .collect();
            
            if info.is_record {
                Ok(vec![Pattern::Record {
                    name: name.clone(),
                    fields: info.fields.iter()
                        .map(|f| (f.clone(), Pattern::Wildcard))
                        .collect(),
                }])
            } else {
                Ok(vec![Pattern::Constructor {
                    name: name.clone(),
                    args,
                }])
            }
        }

        /// 计算已覆盖的构造器
        fn covered_constructors(&self, patterns: &[Pattern]) -> HashSet<String> {
            let mut covered = HashSet::new();
            for pattern in patterns {
                self.add_covered(pattern, &mut covered);
            }
            covered
        }

        fn add_covered(&self, pattern: &Pattern, covered: &mut HashSet<String>) {
            match pattern {
                Pattern::Bool(b) => {
                    covered.insert(format!("Bool({})", b));
                }
                
                Pattern::Constructor { name, args } => {
                    covered.insert(name.clone());
                    for arg in args {
                        self.add_covered(arg, covered);
                    }
                }
                
                Pattern::NamedConstructor { enum_name, variant_name, args } => {
                    let full_name = format!("{}::{}", enum_name, variant_name);
                    covered.insert(full_name);
                    for arg in args {
                        self.add_covered(arg, covered);
                    }
                }
                
                Pattern::Tuple(patterns) => {
                    covered.insert("Tuple".to_string());
                    for p in patterns {
                        self.add_covered(p, covered);
                    }
                }
                
                Pattern::Array(patterns, _) => {
                    covered.insert("Array".to_string());
                    for p in patterns {
                        self.add_covered(p, covered);
                    }
                }
                
                Pattern::Record { name, fields } => {
                    covered.insert(name.clone());
                    for (_, p) in fields {
                        self.add_covered(p, covered);
                    }
                }
                
                Pattern::Or(patterns) => {
                    for p in patterns {
                        self.add_covered(p, covered);
                    }
                }
                
                Pattern::Wildcard => {
                    covered.insert("*".to_string());
                }
                
                Pattern::Variable(_) => {
                    covered.insert("*".to_string());
                }
                
                _ => {
                    covered.insert("*".to_string());
                }
            }
        }

        /// 生成详细的遗漏警告
        pub fn generate_report(&self, ty: &Type) -> String {
            let mut report = StringBuilder::new();
            
            if self.uncovered.is_empty() {
                report.push_str("✅ Pattern matching is exhaustive\n");
                return report.to_string();
            }
            
            report.push_str("⚠️ Non-exhaustive patterns!\n\n");
            report.push_str("Missing cases:\n");
            
            for (i, pattern) in self.uncovered.iter().enumerate() {
                report.push_str(&format!("  {}. {}\n", i + 1, pattern_to_string(pattern)));
            }
            
            // 建议补充的模式
            report.push_str("\nConsider adding:\n");
            if let Some(suggestion) = self.suggest_missing_pattern(ty) {
                report.push_str(&format!("  {}\n", suggestion));
            }
            
            report.to_string()
        }

        fn suggest_missing_pattern(&self, ty: &Type) -> Option<String> {
            match ty {
                Type::Enum(name, _) => {
                    if let Some(variants) = self.constructors.get_variants(name) {
                        let covered: HashSet<String> = self.covered_constructors(&self.patterns);
                        for variant in variants {
                            let full_name = format!("{}::{}", name, variant);
                            if !covered.contains(&full_name) {
                                return Some(format!("case {}::{{ ... }}", full_name));
                            }
                        }
                    }
                    None
                }
                Type::Bool => {
                    let covered: HashSet<String> = self.covered_constructors(&self.patterns);
                    if !covered.contains("Bool(true)") {
                        return Some("case true => ...".to_string());
                    }
                    if !covered.contains("Bool(false)") {
                        return Some("case false => ...".to_string());
                    }
                    None
                }
                _ => None,
            }
        }
    }

    // ==================== 辅助函数 ====================

    fn pattern_to_string(pattern: &Pattern) -> String {
        match pattern {
            Pattern::Variable(name) => name.clone(),
            Pattern::Wildcard => "_".to_string(),
            Pattern::Bool(true) => "true".to_string(),
            Pattern::Bool(false) => "false".to_string(),
            Pattern::Int(n) => n.to_string(),
            Pattern::Float(f) => f.to_string(),
            Pattern::Char(c) => format!("'{}'", c),
            Pattern::String(s) => format!("\"{}\"", s.as_str()),
            Pattern::Bytes(b) => format!("{:?}", b),
            Pattern::Constructor { name, args } => {
                if args.is_empty() {
                    name.clone()
                } else {
                    format!("{}({})", name, 
                        args.iter().map(pattern_to_string).collect::<Vec<_>>().join(", "))
                }
            }
            Pattern::NamedConstructor { enum_name, variant_name, args } => {
                if args.is_empty() {
                    format!("{}::{}", enum_name, variant_name)
                } else {
                    format!("{}::{}({})", enum_name, variant_name,
                        args.iter().map(pattern_to_string).collect::<Vec<_>>().join(", "))
                }
            }
            Pattern::Tuple(patterns) => {
                let s = patterns.iter().map(pattern_to_string).collect::<Vec<_>>().join(", ");
                format!("({})", s)
            }
            Pattern::Array(patterns, len) => {
                let s = patterns.iter().map(pattern_to_string).collect::<Vec<_>>().join(", ");
                match len {
                    Some(n) => format!("[{}; {}]", s, n),
                    None => format!("[{}]", s),
                }
            }
            Pattern::Record { name, fields } => {
                let s = fields.iter()
                    .map(|(n, p)| format!("{}: {}", n, pattern_to_string(p)))
                    .collect::<Vec<_>>().join(", ");
                format!("{} {{ {} }}", name, s)
            }
            Pattern::Or(patterns) => {
                patterns.iter().map(pattern_to_string).collect::<Vec<_>>().join(" | ")
            }
            Pattern::Async(p) => format!("async {}", pattern_to_string(p)),
            Pattern::TypeAs { pattern, type_ } => {
                format!("({} : {})", pattern_to_string(pattern), type_to_string(type_))
            }
        }
    }

    fn type_to_string(ty: &Type) -> String {
        match ty {
            Type::Unit => "()".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Int(None) => "int".to_string(),
            Type::Int(Some(n)) => format!("int{}", n * 8),
            Type::Float => "float".to_string(),
            Type::Char => "char".to_string(),
            Type::String => "string".to_string(),
            Type::Bytes => "bytes".to_string(),
            Type::Tuple(types) => {
                let s = types.iter().map(type_to_string).collect::<Vec<_>>().join(", ");
                format!("({})", s)
            }
            Type::Array(t, None) => format!("[{}]", type_to_string(t)),
            Type::Array(t, Some(n)) => format!("[{}; {}]", type_to_string(t), n),
            Type::Named(name) => name.clone(),
            Type::Enum(name, _) => name.clone(),
            Type::Record(name, _) => name.clone(),
            Type::Function(param, ret) => {
                format!("{} -> {}", type_to_string(param), type_to_string(ret))
            }
            Type::Var(name) => format!("'{}", name),
        }
    }

    // ==================== 结果和错误 ====================

    #[derive(Debug, Clone)]
    pub struct ExhaustivenessResult {
        pub is_exhaustive: bool,
        pub uncovered_patterns: Vec<Pattern>,
        pub warnings: Vec<String>,
    }

    impl ExhaustivenessResult {
        pub fn is_empty(&self) -> bool {
            self.uncovered_patterns.is_empty()
        }

        pub fn report(&self) -> String {
            let mut report = StringBuilder::new();
            
            if self.is_exhaustive {
                report.push_str("✅ All patterns covered\n");
            } else {
                report.push_str(&format!("⚠️ {} uncovered pattern(s)\n", self.uncovered_patterns.len()));
                for (i, pattern) in self.uncovered_patterns.iter().enumerate() {
                    report.push_str(&format!("  {}. {}\n", i + 1, pattern_to_string(pattern)));
                }
            }
            
            if !self.warnings.is_empty() {
                for warning in &self.warnings {
                    report.push_str(&format!("  Warning: {}\n", warning));
                }
            }
            
            report.to_string()
        }
    }

    #[derive(Debug, Clone)]
    pub enum ExhaustivenessError {
        UnknownType { type_name: string },
        InvalidPattern { pattern: string, reason: string },
        TypeMismatch { expected: Type, found: Type },
    }

    impl std::fmt::Display for ExhaustivenessError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ExhaustivenessError::UnknownType { type_name } => {
                    write!(f, "Unknown type: {}", type_name)
                }
                ExhaustivenessError::InvalidPattern { pattern, reason } => {
                    write!(f, "Invalid pattern '{}': {}", pattern, reason)
                }
                ExhaustivenessError::TypeMismatch { expected, found } => {
                    write!(f, "Type mismatch: expected {}, found {}", 
                        type_to_string(expected), type_to_string(found))
                }
            }
        }
    }
}
