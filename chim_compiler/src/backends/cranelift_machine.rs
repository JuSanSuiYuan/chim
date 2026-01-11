use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type as IRType};
use crate::ast;
use std::error::Error;
use std::sync::Arc;

pub struct CraneliftMachineCodeBackend {
    isa: Arc<dyn cranelift_codegen::isa::TargetIsa>,
}

impl CraneliftMachineCodeBackend {
    pub fn new() -> Self {
        let flags = cranelift_codegen::settings::Flags::new(cranelift_codegen::settings::Config::new());
        
        let builder = cranelift_codegen::isa::TargetIsaBuilder::new(
            cranelift_native::get_native_arch().expect("Failed to get native architecture"),
            flags,
        );
        
        let isa = builder.finish().expect("Failed to create TargetISA");
        
        Self { isa: Arc::new(isa) }
    }
    
    fn convert_type(&self, ty: &IRType) -> cranelift_codegen::type::Type {
        match ty {
            IRType::Void => cranelift_codegen::type::INVALID,
            IRType::Int32 => cranelift_codegen::type::I32,
            IRType::Int64 => cranelift_codegen::type::I64,
            IRType::Float32 => cranelift_codegen::type::F32,
            IRType::Float64 => cranelift_codegen::type::F64,
            IRType::Bool => cranelift_codegen::type::I8,
            IRType::String => cranelift_codegen::type::I64,
            IRType::Ptr(_) => cranelift_codegen::type::I64,
            IRType::Ref(_) => cranelift_codegen::type::I64,
            IRType::MutRef(_) => cranelift_codegen::type::I64,
            IRType::Array(_, _) => cranelift_codegen::type::I64,
            IRType::Struct(_) => cranelift_codegen::type::I64,
        }
    }
    
    fn translate_function(&self, func: &Function) -> Result<cranelift_codegen::ir::Function, Box<dyn Error>> {
        let mut clif_func = cranelift_codegen::ir::Function::new();
        clif_func.name = cranelift_codegen::ir::ExternalName::user(0, 0);
        
        let mut trans = FunctionTranslator {
            clif_func: &mut clif_func,
            value_counter: 0,
            block_map: std::collections::HashMap::new(),
        };
        
        for (idx, (name, ty)) in func.params.iter().enumerate() {
            let arg_value = clif_func.append_block_arg(cranelift_codegen::ir::Block::new(), self.convert_type(ty));
            clif_func.params[idx] = arg_value;
        }
        
        for inst in &func.body {
            trans.translate_instruction(inst)?;
        }
        
        Ok(clif_func)
    }
    
    fn compile_module(&self, module: &Module) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut clif_module = cranelift_codegen::module::Module::new(self.isa.clone());
        
        let mut funcs = Vec::new();
        
        for (idx, func) in module.functions.iter().enumerate() {
            let clif_func = self.translate_function(func)?;
            let func_id = clif_module.declare_function(
                &func.name,
                cranelift_codegen::linkage::Linkage::Export,
                &clif_func.signature,
            ).map_err(|e| Box::new(e) as Box<dyn Error>)?;
            
            funcs.push((func_id, clif_func));
        }
        
        for (func_id, clif_func) in &funcs {
            clif_module.define_function(*func_id, clif_func.clone())?;
        }
        
        clif_module.finalize_definitions();
        
        let mut ctx = cranelift_codegen::Context::for_functions(funcs.into_iter().map(|(_, f)| f).collect());
        
        let code = ctx.emit_to_memory(self.isa.as_ref());
        
        Ok(code)
    }
}

struct FunctionTranslator<'a> {
    clif_func: &'a mut cranelift_codegen::ir::Function,
    value_counter: usize,
    block_map: std::collections::HashMap<String, cranelift_codegen::ir::Block>,
}

impl<'a> FunctionTranslator<'a> {
    fn fresh_value(&mut self, ty: cranelift_codegen::type::Type) -> cranelift_codegen::ir::Value {
        self.value_counter += 1;
        self.clif_func.append_result(cranelift_codegen::ir::Block::new(), ty)
    }
    
    fn get_block(&mut self, label: &str) -> cranelift_codegen::ir::Block {
        if let Some(block) = self.block_map.get(label) {
            return *block;
        }
        let block = self.clif_func.create_block();
        self.block_map.insert(label.to_string(), block);
        block
    }
    
    fn translate_instruction(&mut self, inst: &Instruction) -> Result<(), Box<dyn Error>> {
        match inst {
            Instruction::Alloca { dest, ty } => {
                let clif_ty = match ty {
                    IRType::Int32 => cranelift_codegen::type::I32,
                    IRType::Int64 => cranelift_codegen::type::I64,
                    IRType::Float32 => cranelift_codegen::type::F32,
                    IRType::Float64 => cranelift_codegen::type::F64,
                    _ => cranelift_codegen::type::I64,
                };
                let dest_val = self.fresh_value(cranelift_codegen::type::I64);
                self.clif_func.append_result(cranelift_codegen::ir::Block::new(), clif_ty);
                Ok(())
            }
            
            Instruction::Add { dest, left, right } => {
                let clif_ty = cranelift_codegen::type::I32;
                let lhs = self.fresh_value(clif_ty);
                let rhs = self.fresh_value(clif_ty);
                let result = self.fresh_value(clif_ty);
                
                let mut inst = cranelift_codegen::ir::Instruction::iadd(lhs, rhs);
                inst.set_result(0, result);
                self.clif_func.insert_block_inst(cranelift_codegen::ir::Block::new(), inst);
                Ok(())
            }
            
            Instruction::Sub { dest, left, right } => {
                let clif_ty = cranelift_codegen::type::I32;
                let result = self.fresh_value(clif_ty);
                let mut inst = cranelift_codegen::ir::Instruction::isub(result, result);
                self.clif_func.insert_block_inst(cranelift_codegen::ir::Block::new(), inst);
                Ok(())
            }
            
            Instruction::Mul { dest, left, right } => {
                let clif_ty = cranelift_codegen::type::I32;
                let result = self.fresh_value(clif_ty);
                let mut inst = cranelift_codegen::ir::Instruction::imul(result, result);
                self.clif_func.insert_block_inst(cranelift_codegen::ir::Block::new(), inst);
                Ok(())
            }
            
            Instruction::Div { dest, left, right } => {
                let clif_ty = cranelift_codegen::type::I32;
                let result = self.fresh_value(clif_ty);
                let mut inst = cranelift_codegen::ir::Instruction::sdiv(result, result);
                self.clif_func.insert_block_inst(cranelift_codegen::ir::Block::new(), inst);
                Ok(())
            }
            
            Instruction::Load { dest, src } => {
                let clif_ty = cranelift_codegen::type::I32;
                let result = self.fresh_value(clif_ty);
                let mut inst = cranelift_codegen::ir::Instruction::load(
                    cranelift_codegen::ir::MemFlags::new(),
                    cranelift_codegen::ir::AbiParam::new(clif_ty),
                );
                self.clif_func.insert_block_inst(cranelift_codegen::ir::Block::new(), inst);
                Ok(())
            }
            
            Instruction::Store { dest, src } => {
                let mut inst = cranelift_codegen::ir::Instruction::store(
                    cranelift_codegen::ir::MemFlags::new(),
                    self.fresh_value(cranelift_codegen::type::I32),
                    self.fresh_value(cranelift_codegen::type::I64),
                    cranelift_codegen::ir::AbiParam::new(cranelift_codegen::type::I32),
                );
                self.clif_func.insert_block_inst(cranelift_codegen::ir::Block::new(), inst);
                Ok(())
            }
            
            Instruction::Call { dest, func, args } => {
                let mut inst = cranelift_codegen::ir::Instruction::call(
                    cranelift_codegen::ir::ExternalName::user(0, 0),
                    vec![],
                );
                if let Some(d) = dest {
                    let result = self.fresh_value(cranelift_codegen::type::I32);
                    inst.set_result(0, result);
                }
                self.clif_func.insert_block_inst(cranelift_codegen::ir::Block::new(), inst);
                Ok(())
            }
            
            Instruction::Return(Some(value)) => {
                let mut inst = cranelift_codegen::ir::Instruction::return_(
                    vec![self.fresh_value(cranelift_codegen::type::I32)],
                );
                self.clif_func.insert_block_inst(cranelift_codegen::ir::Block::new(), inst);
                Ok(())
            }
            
            Instruction::Return(None) => {
                let mut inst = cranelift_codegen::ir::Instruction::return_(vec![]);
                self.clif_func.insert_block_inst(cranelift_codegen::ir::Block::new(), inst);
                Ok(())
            }
            
            Instruction::Br(label) => {
                let block = self.get_block(label);
                let mut inst = cranelift_codegen::ir::Instruction::jump(block, vec![]);
                self.clif_func.insert_block_inst(cranelift_codegen::ir::Block::new(), inst);
                Ok(())
            }
            
            Instruction::CondBr { cond, true_bb, false_bb } => {
                let true_block = self.get_block(true_bb);
                let false_block = self.get_block(false_bb);
                let cond_val = self.fresh_value(cranelift_codegen::type::I8);
                let mut inst = cranelift_codegen::ir::Instruction::br_icmp(
                    cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                    cond_val,
                    self.fresh_value(cranelift_codegen::type::I8),
                    true_block,
                    vec![],
                    false_block,
                    vec![],
                );
                self.clif_func.insert_block_inst(cranelift_codegen::ir::Block::new(), inst);
                Ok(())
            }
            
            Instruction::Label(name) => {
                let block = self.get_block(name);
                self.clif_func.layout.append_block(block);
                Ok(())
            }
            
            _ => Ok(())
        }
    }
}

impl CodegenBackend for CraneliftMachineCodeBackend {
    fn name(&self) -> &str {
        "Cranelift Machine Code"
    }
    
    fn generate(&self, module: &Module) -> Result<Vec<u8>, Box<dyn Error>> {
        self.compile_module(module)
    }
    
    fn file_extension(&self) -> &str {
        "o"
    }
}

impl Default for CraneliftMachineCodeBackend {
    fn default() -> Self {
        Self::new()
    }
}
