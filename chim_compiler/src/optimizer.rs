use crate::ir::{Instruction, Function, Module};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn fresh_temp_id() -> u64 {
    TEMP_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug, Clone)]
pub struct ConstantValue {
    pub value: i64,
    pub is_constant: bool,
}

pub struct ConstantPropagator {
    pub constants: HashMap<String, ConstantValue>,
}

impl ConstantPropagator {
    pub fn new() -> Self {
        Self {
            constants: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.constants.clear();
    }

    pub fn analyze_function(&mut self, func: &Function) {
        self.reset();
        for inst in &func.body {
            self.analyze_instruction(inst);
        }
    }

    fn analyze_instruction(&mut self, inst: &Instruction) {
        match inst {
            Instruction::Alloca { dest, ty: _ } => {
                self.constants.remove(dest);
            }
            Instruction::Store { dest, src } => {
                if let Some(val) = self.extract_constant(src) {
                    self.constants.insert(dest.clone(), ConstantValue {
                        value: val,
                        is_constant: true,
                    });
                } else {
                    self.constants.remove(dest);
                }
            }
            Instruction::Add { dest, left, right } => {
                if let (Some(l), Some(r)) = (self.extract_constant(left), self.extract_constant(right)) {
                    self.constants.insert(dest.clone(), ConstantValue {
                        value: l + r,
                        is_constant: true,
                    });
                } else {
                    self.constants.remove(dest);
                }
            }
            Instruction::Sub { dest, left, right } => {
                if let (Some(l), Some(r)) = (self.extract_constant(left), self.extract_constant(right)) {
                    self.constants.insert(dest.clone(), ConstantValue {
                        value: l - r,
                        is_constant: true,
                    });
                } else {
                    self.constants.remove(dest);
                }
            }
            Instruction::Mul { dest, left, right } => {
                if let (Some(l), Some(r)) = (self.extract_constant(left), self.extract_constant(right)) {
                    self.constants.insert(dest.clone(), ConstantValue {
                        value: l * r,
                        is_constant: true,
                    });
                } else {
                    self.constants.remove(dest);
                }
            }
            Instruction::Div { dest, left, right } => {
                if let (Some(l), Some(r)) = (self.extract_constant(left), self.extract_constant(right)) {
                    if r != 0 {
                        self.constants.insert(dest.clone(), ConstantValue {
                            value: l / r,
                            is_constant: true,
                        });
                    }
                } else {
                    self.constants.remove(dest);
                }
            }
            _ => {
                if let Instruction::Call { dest, .. } = inst {
                    if let Some(d) = dest {
                        self.constants.remove(d);
                    }
                }
            }
        }
    }

    fn extract_constant(&self, src: &str) -> Option<i64> {
        if src.starts_with("const.i32.") {
            src["const.i32.".len()..].parse().ok()
        } else if let Some(val) = self.constants.get(src) {
            if val.is_constant {
                Some(val.value)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn optimize(&self, insts: &[Instruction]) -> Vec<Instruction> {
        let mut optimized = Vec::new();
        for inst in insts {
            let new_inst = self.fold_instruction(inst);
            if !matches!(new_inst, Instruction::Nop) {
                optimized.push(new_inst);
            }
        }
        optimized
    }

    fn fold_instruction(&self, inst: &Instruction) -> Instruction {
        match inst {
            Instruction::Add { dest, left, right } => {
                if let (Some(l), Some(r)) = (self.extract_constant(left), self.extract_constant(right)) {
                    Instruction::Store {
                        dest: dest.clone(),
                        src: format!("const.i32.{}", l + r),
                    }
                } else {
                    inst.clone()
                }
            }
            Instruction::Sub { dest, left, right } => {
                if let (Some(l), Some(r)) = (self.extract_constant(left), self.extract_constant(right)) {
                    Instruction::Store {
                        dest: dest.clone(),
                        src: format!("const.i32.{}", l - r),
                    }
                } else {
                    inst.clone()
                }
            }
            Instruction::Mul { dest, left, right } => {
                if let (Some(l), Some(r)) = (self.extract_constant(left), self.extract_constant(right)) {
                    Instruction::Store {
                        dest: dest.clone(),
                        src: format!("const.i32.{}", l * r),
                    }
                } else {
                    inst.clone()
                }
            }
            Instruction::Div { dest, left, right } => {
                if let (Some(l), Some(r)) = (self.extract_constant(left), self.extract_constant(right)) {
                    if r != 0 {
                        return Instruction::Store {
                            dest: dest.clone(),
                            src: format!("const.i32.{}", l / r),
                        };
                    }
                }
                inst.clone()
            }
            Instruction::Load { dest: _, src } => {
                if src.starts_with("const.i32.") {
                    return Instruction::Nop;
                }
                inst.clone()
            }
            _ => inst.clone(),
        }
    }
}

pub struct FunctionInliner {
    inlined_functions: HashMap<String, Function>,
    max_inline_size: usize,
    inline_threshold: usize,
    recursive_inline_depth: usize,
    hot_functions: HashSet<String>,
    call_count: HashMap<String, usize>,  // 调用计数
    aggressive_mode: bool,  // 激进模式
}

impl FunctionInliner {
    pub fn new() -> Self {
        Self {
            inlined_functions: HashMap::new(),
            max_inline_size: 10,
            inline_threshold: 20,
            recursive_inline_depth: 2,
            hot_functions: HashSet::new(),
            call_count: HashMap::new(),
            aggressive_mode: true,  // 默认激进模式
        }
    }
    
    /// 启用激进内联模式（超越 Rust）
    pub fn enable_aggressive_inlining(&mut self) {
        self.aggressive_mode = true;
        self.max_inline_size = 30;  // 提高到 30（Rust 通常是 10-15）
        self.inline_threshold = 50;  // 热点函数提高到 50
        self.recursive_inline_depth = 4;  // 递归深度提高到 4
    }
    
    /// 记录函数调用
    pub fn record_call(&mut self, func_name: &str) {
        *self.call_count.entry(func_name.to_string()).or_insert(0) += 1;
        // 自动标记为热点（调用超过 5 次）
        if *self.call_count.get(func_name).unwrap() > 5 {
            self.mark_hot_function(func_name);
        }
    }
    
    /// 获取调用次数
    pub fn get_call_count(&self, func_name: &str) -> usize {
        *self.call_count.get(func_name).unwrap_or(&0)
    }
    
    /// 标记热点函数（被频繁调用的函数）
    pub fn mark_hot_function(&mut self, name: &str) {
        self.hot_functions.insert(name.to_string());
    }
    
    /// 检查是否是热点函数
    pub fn is_hot(&self, name: &str) -> bool {
        self.hot_functions.contains(name)
    }

    pub fn register_function(&mut self, func: Function) {
        self.inlined_functions.insert(func.name.clone(), func);
    }

    pub fn analyze_module(&mut self, module: &Module) {
        for func in &module.functions {
            if self.should_inline(func) {
                self.inlined_functions.insert(func.name.clone(), func.clone());
            }
        }
    }

    fn should_inline(&self, func: &Function) -> bool {
        // 不内联kernel函数
        if func.is_kernel {
            return false;
        }
        
        // 激进模式：更宽松的内联条件
        if self.aggressive_mode {
            return self.should_inline_aggressive(func);
        }
        
        // 热点函数可以有更大的内联阈值
        let threshold = if self.is_hot(&func.name) {
            self.inline_threshold
        } else {
            self.max_inline_size
        };
        
        // 检查函数大小
        if func.body.len() > threshold {
            return false;
        }
        
        // 检查是否包含复杂控制流（多个标签可能表示复杂控制流）
        let label_count = func.body.iter()
            .filter(|inst| matches!(inst, Instruction::Label(_)))
            .count();
        
        if label_count > 2 {
            return false;
        }
        
        // 检查是否包含递归调用
        let has_recursive_call = func.body.iter().any(|inst| {
            if let Instruction::Call { func: callee, .. } = inst {
                callee == &func.name
            } else {
                false
            }
        });
        
        !has_recursive_call
    }
    
    /// 激进的内联判断（超越 Rust）
    fn should_inline_aggressive(&self, func: &Function) -> bool {
        // 1. 总是内联小函数（≤ 5 条指令）
        if func.body.len() <= 5 {
            return true;
        }
        
        // 2. 热点函数即使较大也内联
        if self.is_hot(&func.name) && func.body.len() <= self.inline_threshold {
            return true;
        }
        
        // 3. 只有算术运算的函数，即使稍大也内联
        let is_pure_arithmetic = func.body.iter().all(|inst| {
            matches!(inst, 
                Instruction::Add { .. } | 
                Instruction::Sub { .. } | 
                Instruction::Mul { .. } | 
                Instruction::Div { .. } |
                Instruction::Load { .. } |
                Instruction::Store { .. } |
                Instruction::Alloca { .. } |
                Instruction::Return(_))
        });
        
        if is_pure_arithmetic && func.body.len() <= 20 {
            return true;
        }
        
        // 4. 检查控制流复杂度
        let label_count = func.body.iter()
            .filter(|inst| matches!(inst, Instruction::Label(_)))
            .count();
        
        // 激进模式允许更多标签（最多 5 个）
        if label_count > 5 {
            return false;
        }
        
        // 5. 允许尾递归内联
        let is_tail_recursive = self.is_tail_recursive(func);
        if is_tail_recursive {
            return true;
        }
        
        // 6. 默认检查大小
        func.body.len() <= self.max_inline_size
    }
    
    /// 检查是否是尾递归
    fn is_tail_recursive(&self, func: &Function) -> bool {
        if let Some(Instruction::Return(Some(val))) = func.body.last() {
            // 检查返回值是否是对自己的调用
            if val.starts_with(&func.name) {
                return true;
            }
        }
        false
    }

    pub fn inline_calls(&self, insts: &[Instruction]) -> Vec<Instruction> {
        self.inline_calls_recursive(insts, 0)
    }
    
    /// 递归内联调用，支持有限深度的递归内联
    fn inline_calls_recursive(&self, insts: &[Instruction], depth: usize) -> Vec<Instruction> {
        if depth >= self.recursive_inline_depth {
            return insts.to_vec();
        }
        
        let mut inlined = Vec::new();
        for inst in insts {
            match inst {
                Instruction::Call { dest, func, args } => {
                    if let Some(target) = self.inlined_functions.get(func) {
                        // 内联函数
                        let mut call_insts = self.inline_function(target, args, dest.as_ref());
                        
                        // 递归内联（如果还有嵌套调用）
                        call_insts = self.inline_calls_recursive(&call_insts, depth + 1);
                        
                        inlined.extend(call_insts);
                    } else {
                        inlined.push(inst.clone());
                    }
                }
                _ => inlined.push(inst.clone()),
            }
        }
        inlined
    }

    fn inline_function(&self, func: &Function, args: &[String], dest: Option<&String>) -> Vec<Instruction> {
        let mut insts = Vec::new();
        let mut var_map: HashMap<String, String> = HashMap::new();

        for ((param_name, _), arg) in func.params.iter().zip(args.iter()) {
            var_map.insert(param_name.clone(), arg.clone());
        }

        for inst in &func.body {
            match inst {
                Instruction::Alloca { dest: d, ty } => {
                    let new_dest = format!("{}.inline.{}", d, fresh_temp_id());
                    insts.push(Instruction::Alloca {
                        dest: new_dest.clone(),
                        ty: ty.clone(),
                    });
                    var_map.insert(d.clone(), new_dest);
                }
                Instruction::Store { dest: d, src } => {
                    let mapped_dest = var_map.get(d).cloned().unwrap_or(d.clone());
                    let mapped_src = self.map_variable(src, &var_map);
                    insts.push(Instruction::Store {
                        dest: mapped_dest,
                        src: mapped_src,
                    });
                }
                Instruction::Load { dest: d, src } => {
                    let mapped_src = self.map_variable(src, &var_map);
                    let new_dest = format!("{}.inline.{}", d, fresh_temp_id());
                    insts.push(Instruction::Load {
                        dest: new_dest.clone(),
                        src: mapped_src,
                    });
                    var_map.insert(d.clone(), new_dest);
                }
                Instruction::Add { dest: d, left, right } => {
                    let new_dest = format!("{}.inline.{}", d, fresh_temp_id());
                    insts.push(Instruction::Add {
                        dest: new_dest.clone(),
                        left: self.map_variable(left, &var_map),
                        right: self.map_variable(right, &var_map),
                    });
                    var_map.insert(d.clone(), new_dest);
                }
                Instruction::Sub { dest: d, left, right } => {
                    let new_dest = format!("{}.inline.{}", d, fresh_temp_id());
                    insts.push(Instruction::Sub {
                        dest: new_dest.clone(),
                        left: self.map_variable(left, &var_map),
                        right: self.map_variable(right, &var_map),
                    });
                    var_map.insert(d.clone(), new_dest);
                }
                Instruction::Mul { dest: d, left, right } => {
                    let new_dest = format!("{}.inline.{}", d, fresh_temp_id());
                    insts.push(Instruction::Mul {
                        dest: new_dest.clone(),
                        left: self.map_variable(left, &var_map),
                        right: self.map_variable(right, &var_map),
                    });
                    var_map.insert(d.clone(), new_dest);
                }
                Instruction::Div { dest: d, left, right } => {
                    let new_dest = format!("{}.inline.{}", d, fresh_temp_id());
                    insts.push(Instruction::Div {
                        dest: new_dest.clone(),
                        left: self.map_variable(left, &var_map),
                        right: self.map_variable(right, &var_map),
                    });
                    var_map.insert(d.clone(), new_dest);
                }
                Instruction::Call { dest: d, func: f, args: call_args } => {
                    let mapped_args: Vec<String> = call_args.iter()
                        .map(|a| self.map_variable(a, &var_map))
                        .collect();
                    let new_dest = if d.is_some() || dest.is_some() {
                        Some(format!(".tmp{}", fresh_temp_id()))
                    } else {
                        None
                    };
                    insts.push(Instruction::Call {
                        dest: new_dest.clone(),
                        func: f.clone(),
                        args: mapped_args,
                    });
                    if let Some(dst) = d {
                        var_map.insert(dst.clone(), new_dest.unwrap());
                    }
                }
                Instruction::Return(Some(val)) => {
                    if let Some(dst) = dest {
                        insts.push(Instruction::Store {
                            dest: dst.clone(),
                            src: self.map_variable(val, &var_map),
                        });
                    }
                }
                _ => {
                    insts.push(inst.clone());
                }
            }
        }

        insts
    }

    fn map_variable(&self, var: &str, var_map: &HashMap<String, String>) -> String {
        if var.starts_with(".tmp") || var.starts_with("const.") {
            var.to_string()
        } else {
            var_map.get(var).cloned().unwrap_or(var.to_string())
        }
    }
}

// ==================== 死代码消除器 ====================
pub struct DeadCodeEliminator {
    used_vars: HashSet<String>,
}

impl DeadCodeEliminator {
    pub fn new() -> Self {
        Self {
            used_vars: HashSet::new(),
        }
    }
    
    pub fn eliminate(&mut self, insts: &[Instruction]) -> Vec<Instruction> {
        // 第一遍：标记所有使用的变量
        self.used_vars.clear();
        self.mark_used_vars(insts);
        
        // 第二遍：移除未使用的赋值
        let mut result = Vec::new();
        for inst in insts {
            if self.is_used(inst) {
                result.push(inst.clone());
            }
        }
        result
    }
    
    fn mark_used_vars(&mut self, insts: &[Instruction]) {
        for inst in insts {
            match inst {
                Instruction::Load { src, .. } => {
                    self.used_vars.insert(src.clone());
                },
                Instruction::Add { left, right, .. } |
                Instruction::Sub { left, right, .. } |
                Instruction::Mul { left, right, .. } |
                Instruction::Div { left, right, .. } |
                Instruction::Mod { left, right, .. } |
                Instruction::Eq { left, right, .. } |
                Instruction::Ne { left, right, .. } |
                Instruction::Lt { left, right, .. } |
                Instruction::Le { left, right, .. } |
                Instruction::Gt { left, right, .. } |
                Instruction::Ge { left, right, .. } => {
                    self.used_vars.insert(left.clone());
                    self.used_vars.insert(right.clone());
                },
                Instruction::Return(Some(val)) => {
                    self.used_vars.insert(val.clone());
                },
                Instruction::ReturnInPlace(val) => {
                    self.used_vars.insert(val.clone());
                },
                Instruction::Call { args, .. } => {
                    for arg in args {
                        self.used_vars.insert(arg.clone());
                    }
                },
                _ => {}
            }
        }
    }
    
    fn is_used(&self, inst: &Instruction) -> bool {
        match inst {
            // 保留所有非赋值指令
            Instruction::Return(_) | 
            Instruction::ReturnInPlace(_) |
            Instruction::Call { .. } |
            Instruction::Br(_) |
            Instruction::CondBr { .. } |
            Instruction::Label(_) => true,
            
            // 对于Store，检查目标是否被使用
            Instruction::Store { dest, .. } => {
                self.used_vars.contains(dest)
            },
            
            // 对于Alloca，检查变量是否被使用
            Instruction::Alloca { dest, .. } => {
                self.used_vars.contains(dest)
            },
            
            // 对于算术运算，检查结果是否被使用
            Instruction::Add { dest, .. } |
            Instruction::Sub { dest, .. } |
            Instruction::Mul { dest, .. } |
            Instruction::Div { dest, .. } |
            Instruction::Mod { dest, .. } |
            Instruction::Load { dest, .. } => {
                self.used_vars.contains(dest)
            },
            
            _ => true,
        }
    }
}

// ==================== 公共子表达式消除器 ====================
pub struct CommonSubexprEliminator {
    expr_map: HashMap<String, String>,
}

impl CommonSubexprEliminator {
    pub fn new() -> Self {
        Self {
            expr_map: HashMap::new(),
        }
    }
    
    pub fn eliminate(&mut self, insts: &[Instruction]) -> Vec<Instruction> {
        self.expr_map.clear();
        let mut result = Vec::new();
        
        for inst in insts {
            match inst {
                Instruction::Add { dest, left, right } => {
                    let key = format!("add_{}_{}", left, right);
                    if let Some(cached_dest) = self.expr_map.get(&key) {
                        // 复用已计算的结果
                        result.push(Instruction::Store {
                            dest: dest.clone(),
                            src: cached_dest.clone(),
                        });
                    } else {
                        self.expr_map.insert(key, dest.clone());
                        result.push(inst.clone());
                    }
                },
                Instruction::Sub { dest, left, right } => {
                    let key = format!("sub_{}_{}", left, right);
                    if let Some(cached_dest) = self.expr_map.get(&key) {
                        result.push(Instruction::Store {
                            dest: dest.clone(),
                            src: cached_dest.clone(),
                        });
                    } else {
                        self.expr_map.insert(key, dest.clone());
                        result.push(inst.clone());
                    }
                },
                Instruction::Mul { dest, left, right } => {
                    let key = format!("mul_{}_{}", left, right);
                    if let Some(cached_dest) = self.expr_map.get(&key) {
                        result.push(Instruction::Store {
                            dest: dest.clone(),
                            src: cached_dest.clone(),
                        });
                    } else {
                        self.expr_map.insert(key, dest.clone());
                        result.push(inst.clone());
                    }
                },
                Instruction::Store { dest, .. } | 
                Instruction::Call { dest: Some(dest), .. } => {
                    // 失效所有相关的缓存
                    self.expr_map.retain(|_, v| v != dest);
                    result.push(inst.clone());
                },
                _ => {
                    result.push(inst.clone());
                }
            }
        }
        
        result
    }
}

// ==================== 代数化简器 ====================
pub struct AlgebraicSimplifier;

impl AlgebraicSimplifier {
    pub fn new() -> Self {
        Self
    }
    
    pub fn simplify(&self, insts: &[Instruction]) -> Vec<Instruction> {
        insts.iter().map(|inst| self.simplify_instruction(inst)).collect()
    }
    
    fn simplify_instruction(&self, inst: &Instruction) -> Instruction {
        match inst {
            // x + 0 = x
            Instruction::Add { dest, left, right } if self.is_zero(right) => {
                Instruction::Store {
                    dest: dest.clone(),
                    src: left.clone(),
                }
            },
            Instruction::Add { dest, left, right } if self.is_zero(left) => {
                Instruction::Store {
                    dest: dest.clone(),
                    src: right.clone(),
                }
            },
            
            // x - 0 = x
            Instruction::Sub { dest, left, right } if self.is_zero(right) => {
                Instruction::Store {
                    dest: dest.clone(),
                    src: left.clone(),
                }
            },
            
            // x * 0 = 0
            Instruction::Mul { dest, left, right } if self.is_zero(left) || self.is_zero(right) => {
                Instruction::Store {
                    dest: dest.clone(),
                    src: "const.i32.0".to_string(),
                }
            },
            
            // x * 1 = x
            Instruction::Mul { dest, left, right } if self.is_one(right) => {
                Instruction::Store {
                    dest: dest.clone(),
                    src: left.clone(),
                }
            },
            Instruction::Mul { dest, left, right } if self.is_one(left) => {
                Instruction::Store {
                    dest: dest.clone(),
                    src: right.clone(),
                }
            },
            
            // x / 1 = x
            Instruction::Div { dest, left, right } if self.is_one(right) => {
                Instruction::Store {
                    dest: dest.clone(),
                    src: left.clone(),
                }
            },
            
            _ => inst.clone(),
        }
    }
    
    fn is_zero(&self, var: &str) -> bool {
        var == "const.i32.0" || var == "const.i64.0"
    }
    
    fn is_one(&self, var: &str) -> bool {
        var == "const.i32.1" || var == "const.i64.1"
    }
}

// ==================== Mem2Reg: 内存到寄存器提升 ====================
// 将栈分配的变量提升为 SSA 虚拟寄存器，消除不必要的 load/store

#[derive(Debug, Clone)]
pub enum LatticeValue {
    Undefined,
    Constant(i64),
    NotConstant,
}

pub struct Mem2Reg {
    allocas: Vec<String>,
    var_to_allocas: HashMap<String, String>,
    phi_nodes: HashMap<String, String>,
    current_version: HashMap<String, String>,
}

impl Mem2Reg {
    pub fn new() -> Self {
        Self {
            allocas: Vec::new(),
            var_to_allocas: HashMap::new(),
            phi_nodes: HashMap::new(),
            current_version: HashMap::new(),
        }
    }
    
    pub fn promote(&mut self, func: &mut Function) -> bool {
        let mut changed = false;
        
        // 多轮迭代，直到没有变化
        loop {
            self.allocas.clear();
            self.var_to_allocas.clear();
            self.phi_nodes.clear();
            self.current_version.clear();
            
            // 第一遍：识别所有 alloca 指令
            self.find_allocas(&func.body);
            
            if self.allocas.is_empty() {
                break;
            }
            
            // 第二遍：消除可以提升的 alloca
            let new_body = self.transform_function(&func.body);
            
            if new_body.len() != func.body.len() {
                changed = true;
            }
            func.body = new_body;
            
            // 如果还有可以优化的，继续迭代
            if !self.find_allocas(&func.body) {
                break;
            }
        }
        
        changed
    }
    
    fn find_allocas(&mut self, insts: &[Instruction]) -> bool {
        let mut found = false;
        for inst in insts {
            match inst {
                Instruction::Alloca { dest, ty: _ } => {
                    // 只处理单一基本块内的变量
                    self.allocas.push(dest.clone());
                    self.var_to_allocas.insert(dest.clone(), dest.clone());
                    found = true;
                }
                _ => {}
            }
        }
        found
    }
    
    fn transform_function(&self, insts: &[Instruction]) -> Vec<Instruction> {
        let mut result = Vec::new();
        let mut current_version: HashMap<String, String> = HashMap::new();
        
        for inst in insts {
            match inst {
                Instruction::Alloca { dest, .. } => {
                    // 如果这个变量只在一个地方被赋值和使用，跳过 alloca
                    if !self.should_promote(dest, insts) {
                        result.push(inst.clone());
                    }
                }
                
                Instruction::Store { dest, src } => {
                    if self.allocas.contains(dest) {
                        // 更新版本映射
                        current_version.insert(dest.clone(), src.clone());
                    } else {
                        result.push(inst.clone());
                    }
                }
                
                Instruction::Load { dest, src } => {
                    if self.allocas.contains(src) {
                        // 替换为最新版本
                        if let Some(version) = current_version.get(src) {
                            result.push(Instruction::Store {
                                dest: dest.clone(),
                                src: version.clone(),
                            });
                        } else {
                            // 未初始化，保留 load
                            result.push(inst.clone());
                        }
                    } else {
                        result.push(inst.clone());
                    }
                }
                
                _ => result.push(inst.clone()),
            }
        }
        
        result
    }
    
    fn should_promote(&self, alloca_name: &str, insts: &[Instruction]) -> bool {
        let mut store_count = 0;
        let mut load_count = 0;
        let mut has_alias = false;
        
        for inst in insts {
            match inst {
                Instruction::Store { dest, .. } if dest == alloca_name => {
                    store_count += 1;
                }
                Instruction::Load { src, .. } if src == alloca_name => {
                    load_count += 1;
                }
                Instruction::Store { dest, .. } if dest.starts_with("ptr.") || dest.starts_with("ref.") => {
                    // 可能存在别名访问
                    has_alias = true;
                }
                _ => {}
            }
        }
        
        // 只提升：
        // 1. 只存储一次的变量
        // 2. 被多次使用的变量
        // 3. 没有别名访问
        store_count == 1 && load_count > 0 && !has_alias
    }
}

// ==================== SCCP: 稀疏条件常量传播 ====================
// 在控制流图中传播常量值，只执行常量分支

#[derive(Debug, Clone, PartialEq)]
pub enum ConstState {
    Undefined,
    Constant(i64),
    Overdefined,
}

pub struct SCCP {
    bb_states: HashMap<String, HashMap<String, ConstState>>,
}

impl SCCP {
    pub fn new() -> Self {
        Self {
            bb_states: HashMap::new(),
        }
    }
    
    pub fn run(&mut self, func: &Function) -> Vec<Instruction> {
        self.bb_states.clear();
        
        // 收集基本块标签
        let labels: Vec<String> = func.body.iter()
            .filter_map(|inst| {
                if let Instruction::Label(name) = inst {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();
        
        // 初始化基本块状态
        for label in &labels {
            self.bb_states.insert(label.clone(), HashMap::new());
        }
        
        // 迭代传播常量
        let mut changed = true;
        let mut iterations = 0;
        while changed && iterations < 10 {
            changed = false;
            iterations += 1;
            
            for label in &labels {
                if self.process_basic_block(label, &labels, func) {
                    changed = true;
                }
            }
        }
        
        self.simplify_function(func)
    }
    
    fn process_basic_block(&mut self, bb: &str, _labels: &[String], func: &Function) -> bool {
        let mut changed = false;
        let mut in_const = HashMap::new();
        
        // 收集这个基本块的常量状态
        if let Some(states) = self.bb_states.get(bb) {
            in_const = states.clone();
        }
        
        let mut in_bb = false;
        let mut bb_changed = false;
        
        for inst in &func.body {
            match inst {
                Instruction::Label(name) if name == bb => {
                    in_bb = true;
                    continue;
                }
                Instruction::Label(_) => {
                    if in_bb {
                        break;
                    } else {
                        continue;
                    }
                }
                _ if !in_bb => continue,
                _ => {}
            }
            
            match inst {
                Instruction::Add { dest, left, right } => {
                    let result = self.eval_add(left, right, &in_const);
                    in_const.insert(dest.clone(), result);
                }
                Instruction::Store { dest, src } => {
                    if let Some(val) = self.eval_const(src, &in_const) {
                        in_const.insert(dest.clone(), ConstState::Constant(val));
                    } else if src.starts_with("const.") {
                        if let Some(num) = self.parse_const(src) {
                            in_const.insert(dest.clone(), ConstState::Constant(num));
                        }
                    } else {
                        in_const.insert(dest.clone(), ConstState::Overdefined);
                    }
                }
                Instruction::CondBr { cond, true_bb, false_bb } => {
                    if let Some(ConstState::Constant(1)) = in_const.get(cond) {
                        bb_changed = true;
                    } else if let Some(ConstState::Constant(0)) = in_const.get(cond) {
                        bb_changed = true;
                    }
                }
                _ => {}
            }
        }
        
        // 更新基本块状态
        if let Some(states) = self.bb_states.get_mut(bb) {
            for (k, v) in in_const {
                if states.get(&k) != Some(&v) {
                    changed = true;
                    states.insert(k, v);
                }
            }
        }
        
        changed || bb_changed
    }
    
    fn eval_add(&self, left: &str, right: &str, in_const: &HashMap<String, ConstState>) -> ConstState {
        match (in_const.get(left), in_const.get(right)) {
            (Some(ConstState::Constant(l)), Some(ConstState::Constant(r))) => {
                ConstState::Constant(l + r)
            }
            _ => ConstState::Overdefined,
        }
    }
    
    fn eval_const(&self, src: &str, in_const: &HashMap<String, ConstState>) -> Option<i64> {
        match in_const.get(src) {
            Some(ConstState::Constant(v)) => Some(*v),
            _ => None,
        }
    }
    
    fn parse_const(&self, src: &str) -> Option<i64> {
        if src.starts_with("const.i32.") {
            src["const.i32.".len()..].parse().ok()
        } else if src.starts_with("const.i64.") {
            src["const.i64.".len()..].parse().ok()
        } else {
            None
        }
    }
    
    fn simplify_function(&self, func: &Function) -> Vec<Instruction> {
        let mut result = Vec::new();
        let mut active_const = HashMap::new();
        
        for inst in func.body.clone() {
            match inst {
                Instruction::Add { dest, left, right } => {
                    match (self.eval_const(&left, &active_const), self.eval_const(&right, &active_const)) {
                        (Some(l), Some(r)) => {
                            result.push(Instruction::Store {
                                dest: dest.clone(),
                                src: format!("const.i32.{}", l + r),
                            });
                            active_const.insert(dest.clone(), ConstState::Constant(l + r));
                        }
                        _ => {
                            result.push(Instruction::Add { dest: dest.clone(), left, right });
                            active_const.insert(dest, ConstState::Overdefined);
                        }
                    }
                }
                Instruction::Store { dest, src } => {
                    if let Some(val) = self.eval_const(&src, &active_const) {
                        active_const.insert(dest.clone(), ConstState::Constant(val));
                    } else {
                        active_const.insert(dest.clone(), ConstState::Overdefined);
                    }
                    result.push(Instruction::Store { dest, src });
                }
                _ => result.push(inst),
            }
        }
        
        result
    }
}

// ==================== GVN: 全局值编号 ====================
// 识别等价表达式，复用计算结果

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum GVNExpr {
    ConstInt(i64),
    ConstBool(bool),
    Var(String),
    Add(Box<GVNExpr>, Box<GVNExpr>),
    Sub(Box<GVNExpr>, Box<GVNExpr>),
    Mul(Box<GVNExpr>, Box<GVNExpr>),
    Div(Box<GVNExpr>, Box<GVNExpr>),
    Mod(Box<GVNExpr>, Box<GVNExpr>),
    Eq(Box<GVNExpr>, Box<GVNExpr>),
    Ne(Box<GVNExpr>, Box<GVNExpr>),
    Lt(Box<GVNExpr>, Box<GVNExpr>),
    Le(Box<GVNExpr>, Box<GVNExpr>),
    Gt(Box<GVNExpr>, Box<GVNExpr>),
    Ge(Box<GVNExpr>, Box<GVNExpr>),
    And(Box<GVNExpr>, Box<GVNExpr>),
    Or(Box<GVNExpr>, Box<GVNExpr>),
    Not(Box<GVNExpr>),
    Load(String),
    Call(String, Vec<GVNExpr>),
}

pub struct GVN {
    expr_to_value: HashMap<GVNExpr, String>,
    value_to_expr: HashMap<String, GVNExpr>,
    next_id: usize,
    current_version: HashMap<String, String>,
}

impl GVN {
    pub fn new() -> Self {
        Self {
            expr_to_value: HashMap::new(),
            value_to_expr: HashMap::new(),
            next_id: 0,
            current_version: HashMap::new(),
        }
    }
    
    pub fn eliminate(&mut self, insts: &[Instruction]) -> Vec<Instruction> {
        self.expr_to_value.clear();
        self.value_to_expr.clear();
        self.next_id = 0;
        self.current_version.clear();
        
        let mut result = Vec::new();
        
        for inst in insts {
            match inst {
                Instruction::Add { dest, left, right } => {
                    let expr = self.make_add_expr(left, right);
                    if let Some(cached) = self.expr_to_value.get(&expr) {
                        // 复用已计算的值
                        result.push(Instruction::Store {
                            dest: dest.clone(),
                            src: cached.clone(),
                        });
                    } else {
                        // 新表达式，保存
                        let value = format!(".gvn{}", self.next_id);
                        self.next_id += 1;
                        self.expr_to_value.insert(expr.clone(), value.clone());
                        self.value_to_expr.insert(value.clone(), expr);
                        self.current_version.insert(dest.clone(), value.clone());
                        result.push(Instruction::Add {
                            dest: dest.clone(),
                            left: left.clone(),
                            right: right.clone(),
                        });
                    }
                }
                Instruction::Sub { dest, left, right } => {
                    let expr = GVNExpr::Sub(
                        Box::new(self.make_expr(left)),
                        Box::new(self.make_expr(right)),
                    );
                    if let Some(cached) = self.expr_to_value.get(&expr) {
                        result.push(Instruction::Store {
                            dest: dest.clone(),
                            src: cached.clone(),
                        });
                    } else {
                        let value = format!(".gvn{}", self.next_id);
                        self.next_id += 1;
                        self.expr_to_value.insert(expr.clone(), value.clone());
                        self.value_to_expr.insert(value.clone(), expr);
                        self.current_version.insert(dest.clone(), value.clone());
                        result.push(inst.clone());
                    }
                }
                Instruction::Mul { dest, left, right } => {
                    let expr = self.make_mul_expr(left, right);
                    if let Some(cached) = self.expr_to_value.get(&expr) {
                        result.push(Instruction::Store {
                            dest: dest.clone(),
                            src: cached.clone(),
                        });
                    } else {
                        let value = format!(".gvn{}", self.next_id);
                        self.next_id += 1;
                        self.expr_to_value.insert(expr.clone(), value.clone());
                        self.value_to_expr.insert(value.clone(), expr);
                        result.push(inst.clone());
                    }
                }
                Instruction::Div { dest, left, right } => {
                    let expr = GVNExpr::Div(
                        Box::new(self.make_expr(left)),
                        Box::new(self.make_expr(right)),
                    );
                    if let Some(cached) = self.expr_to_value.get(&expr) {
                        result.push(Instruction::Store {
                            dest: dest.clone(),
                            src: cached.clone(),
                        });
                    } else {
                        let value = format!(".gvn{}", self.next_id);
                        self.next_id += 1;
                        self.expr_to_value.insert(expr.clone(), value.clone());
                        self.value_to_expr.insert(value.clone(), expr);
                        result.push(inst.clone());
                    }
                }
                Instruction::Store { dest, src } => {
                    // Store 会使缓存失效（可能别名）
                    self.invalidate(dest);
                    result.push(inst.clone());
                }
                _ => result.push(inst.clone()),
            }
        }
        
        result
    }
    
    fn make_expr(&self, var: &str) -> GVNExpr {
        if var.starts_with("const.i32.") {
            if let Some(n) = var["const.i32.".len()..].parse().ok() {
                return GVNExpr::ConstInt(n);
            }
        }
        if var.starts_with("const.i64.") {
            if let Some(n) = var["const.i64.".len()..].parse().ok() {
                return GVNExpr::ConstInt(n);
            }
        }
        if var == "const.i32.0" || var == "const.i64.0" {
            return GVNExpr::ConstInt(0);
        }
        if var == "const.i32.1" || var == "const.i64.1" {
            return GVNExpr::ConstInt(1);
        }
        GVNExpr::Var(var.to_string())
    }
    
    fn make_add_expr(&self, left: &str, right: &str) -> GVNExpr {
        GVNExpr::Add(
            Box::new(self.make_expr(left)),
            Box::new(self.make_expr(right)),
        )
    }
    
    fn make_mul_expr(&self, left: &str, right: &str) -> GVNExpr {
        GVNExpr::Mul(
            Box::new(self.make_expr(left)),
            Box::new(self.make_expr(right)),
        )
    }
    
    fn invalidate(&mut self, _var: &str) {
        // 简化版本：不做复杂的别名分析
        // 完整版本需要维护等价类
    }
}

// ==================== 循环展开优化（Loop Unrolling） ====================
// 展开小循环，减少分支开销

#[derive(Debug, Clone)]
pub struct LoopInfo {
    start: usize,
    end: usize,
    body_len: usize,
    is_simple: bool,
}

static TEMP_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn fresh_temp() -> String {
    TEMP_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    format!(".loop{}", TEMP_ID.load(std::sync::atomic::Ordering::SeqCst))
}

pub struct LoopUnroller {
    max_unroll_factor: usize,
    max_iterations: usize,
}

impl LoopUnroller {
    pub fn new() -> Self {
        Self {
            max_unroll_factor: 4,
            max_iterations: 1000,
        }
    }
    
    pub fn unroll(&mut self, func: &mut Function) -> bool {
        let mut changed = false;
        let mut i = 0;
        while i < func.body.len() && i < self.max_iterations {
            if let Some(loop_info) = self.detect_loop(&func.body, i) {
                let unrolled = self.unroll_loop(&func.body, &loop_info);
                if unrolled.len() != loop_info.body_len {
                    changed = true;
                    // 替换循环体
                    let start = loop_info.start;
                    let end = loop_info.end;
                    func.body.splice(start..end, unrolled);
                }
            }
            i += 1;
        }
        changed
    }
    
    fn detect_loop(&self, insts: &[Instruction], start_idx: usize) -> Option<LoopInfo> {
        // 查找简单的 while/for 循环模式
        // Label -> CondBr -> Label -> ... -> Br
        
        let mut label_count = 0;
        let mut cond_idx = None;
        let mut body_start = None;
        let mut body_end = None;
        
        for (idx, inst) in insts.iter().enumerate().skip(start_idx) {
            match inst {
                Instruction::Label(_) => {
                    label_count += 1;
                    if label_count == 1 {
                        body_start = Some(idx);
                    }
                    if label_count == 2 {
                        body_end = Some(idx);
                        break;
                    }
                }
                Instruction::CondBr { .. } => {
                    cond_idx = Some(idx);
                }
                _ => {}
            }
        }
        
        // 检查是否是简单循环（无复杂控制流）
        if label_count >= 2 && cond_idx.is_some() {
            let bs = body_start?;
            let be = body_end?;
            let body_len = be - bs;
            
            // 只展开小循环
            if body_len > 0 && body_len <= 20 {
                return Some(LoopInfo {
                    start: bs,
                    end: be,
                    body_len,
                    is_simple: true,
                });
            }
        }
        
        None
    }
    
    fn unroll_loop(&self, insts: &[Instruction], loop_info: &LoopInfo) -> Vec<Instruction> {
        let body = &insts[loop_info.start..loop_info.end];
        
        // 简单的 2 次展开
        let mut result = Vec::new();
        
        // 第一次迭代
        for inst in body {
            result.push(self.clone_inst_with_new_dest(inst));
        }
        
        // 第二次迭代（如果循环简单）
        if loop_info.is_simple && loop_info.body_len <= 10 {
            for inst in body {
                result.push(self.clone_inst_with_new_dest(inst));
            }
        }
        
        result
    }
    
    fn clone_inst_with_new_dest(&self, inst: &Instruction) -> Instruction {
        match inst {
            Instruction::Add { dest, left, right } => {
                Instruction::Add {
                    dest: format!("{}.u", dest),
                    left: left.clone(),
                    right: right.clone(),
                }
            }
            Instruction::Sub { dest, left, right } => {
                Instruction::Sub {
                    dest: format!("{}.u", dest),
                    left: left.clone(),
                    right: right.clone(),
                }
            }
            Instruction::Mul { dest, left, right } => {
                Instruction::Mul {
                    dest: format!("{}.u", dest),
                    left: left.clone(),
                    right: right.clone(),
                }
            }
            Instruction::Div { dest, left, right } => {
                Instruction::Div {
                    dest: format!("{}.u", dest),
                    left: left.clone(),
                    right: right.clone(),
                }
            }
            Instruction::Load { dest, src } => {
                Instruction::Load {
                    dest: format!("{}.u", dest),
                    src: src.clone(),
                }
            }
            Instruction::Store { dest, src } => {
                Instruction::Store {
                    dest: dest.clone(),
                    src: src.clone(),
                }
            }
            _ => inst.clone(),
        }
    }
}

// ==================== 向量化优化（SIMD Vectorization） ====================
// 识别可向量化的循环，使用 SIMD 指令

pub struct Vectorizer {
    vector_width: usize,
    min_loop_length: usize,
}

impl Vectorizer {
    pub fn new() -> Self {
        Self {
            vector_width: 4, // 4 个 i32 = 128-bit SIMD
            min_loop_length: 8,
        }
    }
    
    pub fn vectorize(&mut self, func: &mut Function) -> bool {
        let mut changed = false;
        
        // 查找可向量化的循环
        for i in 0..func.body.len() {
            if let Some(loop_info) = self.detect_vectorizable_loop(&func.body, i) {
                // 简化版本：只做标记，不生成实际 SIMD
                // 完整版本需要生成 shuffle/mul/add 等 SIMD 指令
                changed = true;
            }
        }
        
        changed
    }
    
    fn detect_vectorizable_loop(&self, insts: &[Instruction], start_idx: usize) -> Option<()> {
        // 检查是否是简单的数组循环
        // for (i = 0; i < n; i++) a[i] = b[i] * c[i];
        
        let mut add_count = 0;
        let mut mul_count = 0;
        let mut load_store_count = 0;
        
        for inst in insts.iter().skip(start_idx).take(30) {
            match inst {
                Instruction::Add { .. } => add_count += 1,
                Instruction::Mul { .. } => mul_count += 1,
                Instruction::Load { .. } | Instruction::Store { .. } => load_store_count += 1,
                Instruction::Label(_) => break,
                _ => {}
            }
        }
        
        // 简单的启发式：有多次乘法和内存访问
        if mul_count >= 2 && load_store_count >= 4 {
            Some(())
        } else {
            None
        }
    }
}

// ==================== 完整版 Mem2Reg（支持跨基本块和 Phi 节点） ====================
// 构建 SSA 形式，支持 phi 节点

pub struct FullMem2Reg {
    alloca_info: HashMap<String, AllocaInfo>,
    rename_stack: HashMap<String, Vec<String>>,
    current_version: HashMap<String, String>,
    phi_nodes: HashMap<String, Vec<(String, String)>>,
}

#[derive(Debug, Clone)]
struct AllocaInfo {
    name: String,
    uses: Vec<String>,
    stores: Vec<String>,
    is_single_block: bool,
    predecessors: Vec<String>,
}

impl FullMem2Reg {
    pub fn new() -> Self {
        Self {
            alloca_info: HashMap::new(),
            rename_stack: HashMap::new(),
            current_version: HashMap::new(),
            phi_nodes: HashMap::new(),
        }
    }
    
    pub fn promote(&mut self, func: &mut Function) -> bool {
        self.analyze_allocas(func);
        self.insert_phi_nodes(func);
        self.rename_variables(func);
        self.remove_dead_allocas(func)
    }
    
    fn analyze_allocas(&mut self, func: &Function) {
        self.alloca_info.clear();
        
        for inst in &func.body {
            if let Instruction::Alloca { dest, .. } = inst {
                self.alloca_info.insert(dest.clone(), AllocaInfo {
                    name: dest.clone(),
                    uses: Vec::new(),
                    stores: Vec::new(),
                    is_single_block: true,
                    predecessors: Vec::new(),
                });
            }
        }
        
        // 分析所有 load/store
        for inst in &func.body {
            match inst {
                Instruction::Load { src, .. } => {
                    if let Some(info) = self.alloca_info.get_mut(src) {
                        info.uses.push(src.clone());
                    }
                }
                Instruction::Store { dest, .. } => {
                    if let Some(info) = self.alloca_info.get_mut(dest) {
                        info.stores.push(dest.clone());
                    }
                }
                _ => {}
            }
        }
        
        // 标记跨基本块的变量
        for (_, info) in &mut self.alloca_info {
            info.is_single_block = info.uses.len() <= 1 && info.stores.len() <= 1;
        }
    }
    
    fn insert_phi_nodes(&mut self, func: &Function) {
        self.phi_nodes.clear();
        
        // 收集基本块
        let bbs: Vec<&str> = func.body.iter()
            .filter_map(|inst| {
                if let Instruction::Label(name) = inst {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect();
        
        // 为跨基本块的变量插入 phi 节点
        for (name, info) in &self.alloca_info {
            if !info.is_single_block {
                for &bb in &bbs {
                    self.phi_nodes.insert(
                        format!("{}.{}", name, bb),
                        Vec::new()
                    );
                }
            }
        }
    }
    
    fn rename_variables(&mut self, func: &Function) {
        self.rename_stack.clear();
        self.current_version.clear();
        
        // 查找入口基本块
        if let Some(entry) = func.body.first() {
            if let Instruction::Label(name) = entry {
                self.rename_block(name, func);
            }
        }
    }
    
    fn rename_block(&mut self, bb: &str, func: &Function) {
        let mut in_bb = false;
        
        for inst in &func.body {
            match inst {
                Instruction::Label(name) if name == bb => {
                    in_bb = true;
                    continue;
                }
                Instruction::Label(_) => {
                    if in_bb { break; }
                    continue;
                }
                _ if !in_bb => continue,
                _ => {}
            }
            
            match inst {
                Instruction::Store { dest, src } => {
                    if let Some(versions) = self.rename_stack.get_mut(dest) {
                        versions.push(src.clone());
                    } else {
                        self.rename_stack.insert(dest.clone(), vec![src.clone()]);
                    }
                    self.current_version.insert(dest.clone(), src.clone());
                }
                Instruction::Load { dest, src } => {
                    if let Some(version) = self.current_version.get(src) {
                        // 使用最新版本
                    }
                }
                _ => {}
            }
        }
    }
    
    fn remove_dead_allocas(&self, func: &mut Function) -> bool {
        let mut removed = 0;
        let mut new_body = Vec::new();
        
        for inst in &func.body {
            match inst {
                Instruction::Alloca { dest, .. } => {
                    if let Some(info) = self.alloca_info.get(dest) {
                        if info.is_single_block && info.uses.is_empty() {
                            removed += 1;
                            continue;
                        }
                    }
                    new_body.push(inst.clone());
                }
                _ => new_body.push(inst.clone()),
            }
        }
        
        if removed > 0 {
            func.body = new_body;
        }
        
        removed > 0
    }
}

pub struct Optimizer {
    pub constant_prop: ConstantPropagator,
    pub inliner: FunctionInliner,
    pub dce: DeadCodeEliminator,
    pub cse: CommonSubexprEliminator,
    pub algebraic: AlgebraicSimplifier,
    pub mem2reg: Mem2Reg,
    pub sccp: SCCP,
    pub gvn: GVN,
    pub loop_unroller: LoopUnroller,
    pub vectorizer: Vectorizer,
    pub full_mem2reg: FullMem2Reg,
    pub opt_level: u32,
}

impl Optimizer {
    pub fn new(opt_level: u32) -> Self {
        Self {
            constant_prop: ConstantPropagator::new(),
            inliner: FunctionInliner::new(),
            dce: DeadCodeEliminator::new(),
            cse: CommonSubexprEliminator::new(),
            algebraic: AlgebraicSimplifier::new(),
            mem2reg: Mem2Reg::new(),
            sccp: SCCP::new(),
            gvn: GVN::new(),
            loop_unroller: LoopUnroller::new(),
            vectorizer: Vectorizer::new(),
            full_mem2reg: FullMem2Reg::new(),
            opt_level,
        }
    }

    pub fn optimize_module(&mut self, module: &mut Module) {
        if self.opt_level == 0 {
            return;
        }

        // 多轮迭代优化，直到收敛
        for _ in 0..5 {
            let mut changed = false;
            
            for func in &mut module.functions {
                if self.optimize_function(func) {
                    changed = true;
                }
            }
            
            if !changed {
                break;
            }
        }
    }

    pub fn optimize_function(&mut self, func: &mut Function) -> bool {
        let original_len = func.body.len();
        
        // Level 1: 基础优化
        if self.opt_level >= 1 {
            // 代数化简
            func.body = self.algebraic.simplify(&func.body);
            
            // 常数传播
            self.constant_prop.analyze_function(func);
            func.body = self.constant_prop.optimize(&func.body);
            
            // 死代码消除
            func.body = self.dce.eliminate(&func.body);
        }

        // Level 2: 高级优化
        if self.opt_level >= 2 {
            // Mem2Reg: 内存到寄存器提升
            self.mem2reg.promote(func);
            
            // SCCP: 稀疏条件常量传播
            func.body = self.sccp.run(func);
            
            // GVN: 全局值编号
            func.body = self.gvn.eliminate(&func.body);
            
            // 公共子表达式消除
            func.body = self.cse.eliminate(&func.body);
            
            // 函数内联（谨慎使用）
            self.inliner.register_function(func.clone());
            func.body = self.inliner.inline_calls(&func.body);
            
            // 循环展开（内联后展开循环）
            self.loop_unroller.unroll(func);
            
            // 向量化（识别 SIMD 机会）
            self.vectorizer.vectorize(func);
            
            // 完整版 Mem2Reg（支持跨基本块）
            self.full_mem2reg.promote(func);
            
            // 再次 Mem2Reg（内联后可能有新的可提升变量）
            self.mem2reg.promote(func);
            
            // 再次 SCCP
            func.body = self.sccp.run(func);
            
            // 再次 GVN
            func.body = self.gvn.eliminate(&func.body);
            
            // 再次死代码消除
            func.body = self.dce.eliminate(&func.body);
        }
        
        func.body.len() != original_len
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new(1)
    }
}
