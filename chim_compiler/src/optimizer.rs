use crate::ir::{Instruction, Function, Module};
use std::collections::HashMap;
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
}

impl FunctionInliner {
    pub fn new() -> Self {
        Self {
            inlined_functions: HashMap::new(),
            max_inline_size: 10,
        }
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
        func.body.len() <= self.max_inline_size && !func.is_kernel
    }

    pub fn inline_calls(&self, insts: &[Instruction]) -> Vec<Instruction> {
        let mut inlined = Vec::new();
        for inst in insts {
            match inst {
                Instruction::Call { dest, func, args } => {
                    if let Some(target) = self.inlined_functions.get(func) {
                        let call_insts = self.inline_function(target, args, dest.as_ref());
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

pub struct Optimizer {
    pub constant_prop: ConstantPropagator,
    pub inliner: FunctionInliner,
    pub opt_level: u32,
}

impl Optimizer {
    pub fn new(opt_level: u32) -> Self {
        Self {
            constant_prop: ConstantPropagator::new(),
            inliner: FunctionInliner::new(),
            opt_level,
        }
    }

    pub fn optimize_module(&mut self, module: &mut Module) {
        if self.opt_level == 0 {
            return;
        }

        for func in &mut module.functions {
            self.optimize_function(func);
        }
    }

    pub fn optimize_function(&mut self, func: &mut Function) {
        if self.opt_level >= 1 {
            self.constant_prop.analyze_function(func);
            func.body = self.constant_prop.optimize(&func.body);
        }

        if self.opt_level >= 2 {
            self.inliner.register_function(func.clone());
            for inst in &mut func.body {
                if let Instruction::Call { func: name, .. } = inst {
                    if self.inliner.inlined_functions.contains_key(name) {
                        *inst = Instruction::Nop;
                    }
                }
            }
            func.body.retain(|i| !matches!(i, Instruction::Nop));
        }
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new(1)
    }
}
