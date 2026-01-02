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

pub struct Optimizer {
    pub constant_prop: ConstantPropagator,
    pub inliner: FunctionInliner,
    pub dce: DeadCodeEliminator,
    pub cse: CommonSubexprEliminator,
    pub algebraic: AlgebraicSimplifier,
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
            opt_level,
        }
    }

    pub fn optimize_module(&mut self, module: &mut Module) {
        if self.opt_level == 0 {
            return;
        }

        // 多轮迭代优化，直到收敛
        for _ in 0..3 {
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
            // 公共子表达式消除
            func.body = self.cse.eliminate(&func.body);
            
            // 函数内联（谨慎使用）
            self.inliner.register_function(func.clone());
            func.body = self.inliner.inline_calls(&func.body);
            
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
