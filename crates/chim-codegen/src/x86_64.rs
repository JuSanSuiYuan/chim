use crate::{CodegenTarget, CodeGenerator, GeneratedCode, CodegenError};
use chim_ir::{IRModule, IRFunction, IRInst, BinaryOp, UnaryOp, Terminator, ValueId, VarId, BlockId, TypeId, MemoryOrder, CaptureKind};
use chim_semantic::AnalyzedProgram;
use chim_span::Span;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum X86Register {
    RAX, RCX, RDX, RBX, RSP, RBP, RSI, RDI,
    R8, R9, R10, R11, R12, R13, R14, R15,
    XMM0, XMM1, XMM2, XMM3, XMM4, XMM5, XMM6, XMM7,
}

impl X86Register {
    pub fn is_callee_saved(&self) -> bool {
        matches!(self, X86Register::RBX | X86Register::RBP | X86Register::R12 | X86Register::R13 | X86Register::R14 | X86Register::R15)
    }

    pub fn is_arg(&self) -> bool {
        matches!(self, X86Register::RDI | X86Register::RSI | X86Register::RDX | X86Register::RCX | X86Register::R8 | X86Register::R9)
    }

    pub fn is_temp(&self) -> bool {
        matches!(self, X86Register::RAX | X86Register::R10 | X86Register::R11)
    }
}

#[derive(Debug, Clone)]
pub struct X86Instruction {
    pub mnemonic: String,
    pub operands: Vec<String>,
    pub comment: Option<String>,
}

impl X86Instruction {
    pub fn new(mnemonic: &str) -> Self {
        X86Instruction {
            mnemonic: mnemonic.to_string(),
            operands: Vec::new(),
            comment: None,
        }
    }

    pub fn with_operands(mnemonic: &str, operands: Vec<String>) -> Self {
        X86Instruction {
            mnemonic: mnemonic.to_string(),
            operands,
            comment: None,
        }
    }

    pub fn with_comment(mnemonic: &str, comment: &str) -> Self {
        X86Instruction {
            mnemonic: mnemonic.to_string(),
            operands: Vec::new(),
            comment: Some(comment.to_string()),
        }
    }

    pub fn to_asm(&self) -> String {
        let mut result = format!("    {}", self.mnemonic);
        if !self.operands.is_empty() {
            result.push_str(&format!(" {}", self.operands.join(", ")));
        }
        if let Some(comment) = &self.comment {
            result.push_str(&format!(" ; {}", comment));
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub local_offset: i32,
    pub local_size: i32,
    pub arg_offsets: HashMap<VarId, i32>,
}

impl StackFrame {
    pub fn new() -> Self {
        StackFrame {
            local_offset: 0,
            local_size: 0,
            arg_offsets: HashMap::new(),
        }
    }

    pub fn allocate_local(&mut self, size: i32) -> i32 {
        self.local_offset -= size;
        self.local_offset = (self.local_offset + 15) & !15;
        self.local_size = self.local_size.max(-self.local_offset);
        self.local_offset
    }

    pub fn allocate_arg(&mut self, var: VarId, size: i32) {
        let offset = 8 + self.arg_offsets.len() as i32 * 8;
        self.arg_offsets.insert(var, offset);
        offset
    }
}

#[derive(Debug, Clone)]
pub struct RegisterAllocator {
    pub free_regs: HashSet<X86Register>,
    pub used_regs: HashSet<X86Register>,
    pub var_to_reg: HashMap<VarId, X86Register>,
    pub reg_to_var: HashMap<X86Register, VarId>,
}

impl RegisterAllocator {
    pub fn new() -> Self {
        let mut free_regs = HashSet::new();
        free_regs.insert(X86Register::RAX);
        free_regs.insert(X86Register::RCX);
        free_regs.insert(X86Register::RDX);
        free_regs.insert(X86Register::RBX);
        free_regs.insert(X86Register::RBP);
        free_regs.insert(X86Register::RSI);
        free_regs.insert(X86Register::RDI);
        free_regs.insert(X86Register::R8);
        free_regs.insert(X86Register::R9);
        free_regs.insert(X86Register::R10);
        free_regs.insert(X86Register::R11);
        free_regs.insert(X86Register::R12);
        free_regs.insert(X86Register::R13);
        free_regs.insert(X86Register::R14);
        free_regs.insert(X86Register::R15);

        RegisterAllocator {
            free_regs,
            used_regs: HashSet::new(),
            var_to_reg: HashMap::new(),
            reg_to_var: HashMap::new(),
        }
    }

    pub fn allocate(&mut self, var: VarId) -> Option<X86Register> {
        for reg in &self.free_regs {
            if reg.is_temp() && !self.used_regs.contains(reg) {
                self.free_regs.remove(reg);
                self.used_regs.insert(*reg);
                self.var_to_reg.insert(var, *reg);
                self.reg_to_var.insert(*reg, var);
                return Some(*reg);
            }
        }
        None
    }

    pub fn allocate_callee_saved(&mut self) -> Option<X86Register> {
        for reg in &self.free_regs {
            if reg.is_callee_saved() && !self.used_regs.contains(reg) {
                self.free_regs.remove(reg);
                self.used_regs.insert(*reg);
                return Some(*reg);
            }
        }
        None
    }

    pub fn free(&mut self, reg: X86Register) {
        if let Some(var) = self.reg_to_var.remove(&reg) {
            self.var_to_reg.remove(&var);
        }
        self.used_regs.remove(&reg);
        self.free_regs.insert(reg);
    }

    pub fn get_reg(&self, var: VarId) -> Option<X86Register> {
        self.var_to_reg.get(&var).copied()
    }
}

#[derive(Debug, Clone)]
pub struct X86CodeGenerator {
    pub reg_allocator: RegisterAllocator,
    pub stack_frame: StackFrame,
    pub instructions: Vec<X86Instruction>,
    pub labels: HashMap<BlockId, String>,
    pub label_counter: usize,
    pub current_function: Option<String>,
}

impl X86CodeGenerator {
    pub fn new() -> Self {
        X86CodeGenerator {
            reg_allocator: RegisterAllocator::new(),
            stack_frame: StackFrame::new(),
            instructions: Vec::new(),
            labels: HashMap::new(),
            label_counter: 0,
            current_function: None,
        }
    }

    pub fn generate_label(&mut self) -> String {
        let label = format!(".L{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    pub fn emit(&mut self, inst: X86Instruction) {
        self.instructions.push(inst);
    }

    pub fn emit_comment(&mut self, comment: &str) {
        self.emit(X86Instruction::with_comment("", comment));
    }

    pub fn emit_label(&mut self, label: &str) {
        self.emit(X86Instruction {
            mnemonic: label.to_string(),
            operands: Vec::new(),
            comment: None,
        });
    }

    pub fn emit_mov(&mut self, dest: &str, src: &str) {
        self.emit(X86Instruction::with_operands("mov", vec![dest.to_string(), src.to_string()]));
    }

    pub fn emit_mov_reg_to_reg(&mut self, dest: X86Register, src: X86Register) {
        self.emit_mov(&format!("{}", Self::reg_to_str(dest)), &format!("{}", Self::reg_to_str(src)));
    }

    pub fn emit_mov_reg_to_mem(&mut self, dest: &str, src: X86Register) {
        self.emit_mov(dest, &format!("[{}]", Self::reg_to_str(src)));
    }

    pub fn emit_mov_mem_to_reg(&mut self, dest: X86Register, src: &str) {
        self.emit_mov(&format!("{}", Self::reg_to_str(dest)), src);
    }

    pub fn emit_mov_imm_to_reg(&mut self, dest: X86Register, imm: i64) {
        self.emit_mov(&format!("{}", Self::reg_to_str(dest)), &format!("{}", imm));
    }

    pub fn emit_add(&mut self, dest: &str, src: &str) {
        self.emit(X86Instruction::with_operands("add", vec![dest.to_string(), src.to_string()]));
    }

    pub fn emit_sub(&mut self, dest: &str, src: &str) {
        self.emit(X86Instruction::with_operands("sub", vec![dest.to_string(), src.to_string()]));
    }

    pub fn emit_mul(&mut self, dest: &str, src: &str) {
        self.emit(X86Instruction::with_operands("imul", vec![dest.to_string(), src.to_string()]));
    }

    pub fn emit_div(&mut self, dest: &str, src: &str) {
        self.emit(X86Instruction::with_operands("idiv", vec![src.to_string()]));
    }

    pub fn emit_cmp(&mut self, op1: &str, op2: &str) {
        self.emit(X86Instruction::with_operands("cmp", vec![op1.to_string(), op2.to_string()]));
    }

    pub fn emit_jmp(&mut self, label: &str) {
        self.emit(X86Instruction::with_operands("jmp", vec![label.to_string()]));
    }

    pub fn emit_je(&mut self, label: &str) {
        self.emit(X86Instruction::with_operands("je", vec![label.to_string()]));
    }

    pub fn emit_jne(&mut self, label: &str) {
        self.emit(X86Instruction::with_operands("jne", vec![label.to_string()]));
    }

    pub fn emit_jg(&mut self, label: &str) {
        self.emit(X86Instruction::with_operands("jg", vec![label.to_string()]));
    }

    pub fn emit_jl(&mut self, label: &str) {
        self.emit(X86Instruction::with_operands("jl", vec![label.to_string()]));
    }

    pub fn emit_jge(&mut self, label: &str) {
        self.emit(X86Instruction::with_operands("jge", vec![label.to_string()]));
    }

    pub fn emit_jle(&mut self, label: &str) {
        self.emit(X86Instruction::with_operands("jle", vec![label.to_string()]));
    }

    pub fn emit_call(&mut self, label: &str) {
        self.emit(X86Instruction::with_operands("call", vec![label.to_string()]));
    }

    pub fn emit_ret(&mut self) {
        self.emit(X86Instruction::new("ret"));
    }

    pub fn emit_push(&mut self, reg: X86Register) {
        self.emit(X86Instruction::with_operands("push", vec![Self::reg_to_str(reg)]));
    }

    pub fn emit_pop(&mut self, reg: X86Register) {
        self.emit(X86Instruction::with_operands("pop", vec![Self::reg_to_str(reg)]));
    }

    pub fn emit_lea(&mut self, dest: X86Register, src: &str) {
        self.emit(X86Instruction::with_operands("lea", vec![Self::reg_to_str(dest), src]));
    }

    pub fn emit_test(&mut self, reg: X86Register) {
        self.emit(X86Instruction::with_operands("test", vec![Self::reg_to_str(reg), Self::reg_to_str(reg)]));
    }

    pub fn emit_and(&mut self, dest: &str, src: &str) {
        self.emit(X86Instruction::with_operands("and", vec![dest.to_string(), src.to_string()]));
    }

    pub fn emit_or(&mut self, dest: &str, src: &str) {
        self.emit(X86Instruction::with_operands("or", vec![dest.to_string(), src.to_string()]));
    }

    pub fn emit_xor(&mut self, dest: &str, src: &str) {
        self.emit(X86Instruction::with_operands("xor", vec![dest.to_string(), src.to_string()]));
    }

    pub fn emit_not(&mut self, dest: &str) {
        self.emit(X86Instruction::with_operands("not", vec![dest.to_string()]));
    }

    pub fn emit_neg(&mut self, dest: &str) {
        self.emit(X86Instruction::with_operands("neg", vec![dest.to_string()]));
    }

    pub fn emit_shl(&mut self, dest: &str, src: &str) {
        self.emit(X86Instruction::with_operands("shl", vec![dest.to_string(), src.to_string()]));
    }

    pub fn emit_shr(&mut self, dest: &str, src: &str) {
        self.emit(X86Instruction::with_operands("shr", vec![dest.to_string(), src.to_string()]));
    }

    pub fn emit_lock(&mut self, mnemonic: &str, operands: Vec<String>) {
        let mut full_operands = vec!["lock".to_string()];
        full_operands.extend(operands);
        self.emit(X86Instruction::with_operands(mnemonic, full_operands));
    }

    pub fn emit_lock_cmpxchg(&mut self, dest: &str, src: &str) {
        self.emit_lock("cmpxchg", vec![dest.to_string(), src.to_string()]);
    }

    pub fn emit_lock_add(&mut self, dest: &str, src: &str) {
        self.emit_lock("add", vec![dest.to_string(), src.to_string()]);
    }

    pub fn emit_lock_sub(&mut self, dest: &str, src: &str) {
        self.emit_lock("sub", vec![dest.to_string(), src.to_string()]);
    }

    pub fn emit_lock_and(&mut self, dest: &str, src: &str) {
        self.emit_lock("and", vec![dest.to_string(), src.to_string()]);
    }

    pub fn emit_lock_or(&mut self, dest: &str, src: &str) {
        self.emit_lock("or", vec![dest.to_string(), src.to_string()]);
    }

    pub fn emit_lock_xor(&mut self, dest: &str, src: &str) {
        self.emit_lock("xor", vec![dest.to_string(), src.to_string()]);
    }

    pub fn emit_lock_inc(&mut self, dest: &str) {
        self.emit_lock("inc", vec![dest.to_string()]);
    }

    pub fn emit_lock_dec(&mut self, dest: &str) {
        self.emit_lock("dec", vec![dest.to_string()]);
    }

    pub fn emit_mfence(&mut self) {
        self.emit(X86Instruction::new("mfence"));
    }

    pub fn emit_sfence(&mut self) {
        self.emit(X86Instruction::new("sfence"));
    }

    pub fn emit_lfence(&mut self) {
        self.emit(X86Instruction::new("lfence"));
    }

    pub fn reg_to_str(reg: X86Register) -> &'static str {
        match reg {
            X86Register::RAX => "rax",
            X86Register::RCX => "rcx",
            X86Register::RDX => "rdx",
            X86Register::RBX => "rbx",
            X86Register::RSP => "rsp",
            X86Register::RBP => "rbp",
            X86Register::RSI => "rsi",
            X86Register::RDI => "rdi",
            X86Register::R8 => "r8",
            X86Register::R9 => "r9",
            X86Register::R10 => "r10",
            X86Register::R11 => "r11",
            X86Register::R12 => "r12",
            X86Register::R13 => "r13",
            X86Register::R14 => "r14",
            X86Register::R15 => "r15",
            X86Register::XMM0 => "xmm0",
            X86Register::XMM1 => "xmm1",
            X86Register::XMM2 => "xmm2",
            X86Register::XMM3 => "xmm3",
            X86Register::XMM4 => "xmm4",
            X86Register::XMM5 => "xmm5",
            X86Register::XMM6 => "xmm6",
            X86Register::XMM7 => "xmm7",
        }
    }

    pub fn memory_order_to_fence(&mut self, order: &MemoryOrder) {
        match order {
            MemoryOrder::Relaxed => {},
            MemoryOrder::Consume => self.emit_lfence(),
            MemoryOrder::Acquire => self.emit_lfence(),
            MemoryOrder::Release => self.emit_sfence(),
            MemoryOrder::AcqRel => { self.emit_lfence(); self.emit_sfence(); }
            MemoryOrder::SeqCst => self.emit_mfence(),
            MemoryOrder::HappensBefore => self.emit_mfence(),
            MemoryOrder::Volatile => self.emit_mfence(),
            MemoryOrder::MemoryBarrier => self.emit_mfence(),
            _ => {}
        }
    }

    pub fn generate_function(&mut self, func: &IRFunction) {
        self.current_function = Some(func.name.clone());
        self.emit_comment(&format!("Function: {}", func.name));
        
        self.emit_label(&format!("_{}", func.name));
        
        self.emit_push(X86Register::RBP);
        self.emit_mov_reg_to_reg(X86Register::RBP, X86Register::RSP);
        
        self.stack_frame = StackFrame::new();
        
        for (i, param) in func.params.iter().enumerate() {
            let offset = self.stack_frame.allocate_arg(param.id, 8);
            self.emit_comment(&format!("Parameter {}: offset = {}", i, offset));
        }
        
        if self.stack_frame.local_size > 0 {
            self.emit_sub(&format!("{}", Self::reg_to_str(X86Register::RSP)), &format!("{}", self.stack_frame.local_size));
        }
        
        self.reg_allocator = RegisterAllocator::new();
        
        for block in &func.body {
            self.generate_block(block);
        }
        
        if self.stack_frame.local_size > 0 {
            self.emit_add(&format!("{}", Self::reg_to_str(X86Register::RSP)), &format!("{}", self.stack_frame.local_size));
        }
        
        self.emit_pop(X86Register::RBP);
        self.emit_ret();
        
        self.current_function = None;
    }

    pub fn generate_block(&mut self, block: &BlockId) {
        let label = self.labels.get(block).cloned().unwrap_or_else(|| {
            let label = self.generate_label();
            self.labels.insert(*block, label.clone());
            label
        });
        
        self.emit_label(&label);
    }

    pub fn generate_binary_op(&mut self, op: &BinaryOp, left: ValueId, right: ValueId, dest: VarId) {
        let left_reg = self.reg_allocator.get_reg(left).unwrap_or(X86Register::RAX);
        let right_reg = self.reg_allocator.get_reg(right).unwrap_or(X86Register::RCX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RDX);
        
        match op {
            BinaryOp::Add => self.emit_add(&format!("{}", Self::reg_to_str(dest_reg)), &format!("{}", Self::reg_to_str(right_reg))),
            BinaryOp::Sub => self.emit_sub(&format!("{}", Self::reg_to_str(dest_reg)), &format!("{}", Self::reg_to_str(right_reg))),
            BinaryOp::Mul => self.emit_mul(&format!("{}", Self::reg_to_str(dest_reg)), &format!("{}", Self::reg_to_str(right_reg))),
            BinaryOp::Div => {
                self.emit_mov_reg_to_reg(X86Register::RAX, left_reg);
                self.emit_div(&format!("{}", Self::reg_to_str(right_reg)));
                self.emit_mov_reg_to_reg(dest_reg, X86Register::RAX);
            }
            BinaryOp::And => self.emit_and(&format!("{}", Self::reg_to_str(dest_reg)), &format!("{}", Self::reg_to_str(right_reg))),
            BinaryOp::Or => self.emit_or(&format!("{}", Self::reg_to_str(dest_reg)), &format!("{}", Self::reg_to_str(right_reg))),
            BinaryOp::Xor => self.emit_xor(&format!("{}", Self::reg_to_str(dest_reg)), &format!("{}", Self::reg_to_str(right_reg))),
            BinaryOp::Shl => self.emit_shl(&format!("{}", Self::reg_to_str(dest_reg)), &format!("{}", Self::reg_to_str(right_reg))),
            BinaryOp::Shr => self.emit_shr(&format!("{}", Self::reg_to_str(dest_reg)), &format!("{}", Self::reg_to_str(right_reg))),
            _ => {}
        }
    }

    pub fn generate_unary_op(&mut self, op: &UnaryOp, operand: ValueId, dest: VarId) {
        let operand_reg = self.reg_allocator.get_reg(operand).unwrap_or(X86Register::RAX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RCX);
        
        match op {
            UnaryOp::Neg => self.emit_neg(&format!("{}", Self::reg_to_str(dest_reg))),
            UnaryOp::Not => self.emit_not(&format!("{}", Self::reg_to_str(dest_reg))),
            _ => {}
        }
        
        self.emit_mov_reg_to_reg(dest_reg, operand_reg);
    }

    pub fn generate_atomic_load(&mut self, dest: VarId, src: ValueId, order: &MemoryOrder) {
        let src_reg = self.reg_allocator.get_reg(src).unwrap_or(X86Register::RAX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RCX);
        
        self.memory_order_to_fence(order);
        self.emit_mov_reg_to_reg(dest_reg, src_reg);
    }

    pub fn generate_atomic_store(&mut self, dest: ValueId, src: ValueId, order: &MemoryOrder) {
        let dest_reg = self.reg_allocator.get_reg(dest).unwrap_or(X86Register::RAX);
        let src_reg = self.reg_allocator.get_reg(src).unwrap_or(X86Register::RCX);
        
        self.memory_order_to_fence(order);
        self.emit_lock_mov(&format!("[{}]", Self::reg_to_str(dest_reg)), &format!("{}", Self::reg_to_str(src_reg)));
    }

    pub fn generate_atomic_fetch_add(&mut self, dest: VarId, src: ValueId, value: ValueId, order: &MemoryOrder) {
        let src_reg = self.reg_allocator.get_reg(src).unwrap_or(X86Register::RAX);
        let value_reg = self.reg_allocator.get_reg(value).unwrap_or(X86Register::RCX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RDX);
        
        self.memory_order_to_fence(order);
        self.emit_lock_add(&format!("[{}]", Self::reg_to_str(src_reg)), &format!("{}", Self::reg_to_str(value_reg)));
        self.emit_mov_reg_to_reg(dest_reg, src_reg);
    }

    pub fn generate_atomic_fetch_sub(&mut self, dest: VarId, src: ValueId, value: ValueId, order: &MemoryOrder) {
        let src_reg = self.reg_allocator.get_reg(src).unwrap_or(X86Register::RAX);
        let value_reg = self.reg_allocator.get_reg(value).unwrap_or(X86Register::RCX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RDX);
        
        self.memory_order_to_fence(order);
        self.emit_lock_sub(&format!("[{}]", Self::reg_to_str(src_reg)), &format!("{}", Self::reg_to_str(value_reg)));
        self.emit_mov_reg_to_reg(dest_reg, src_reg);
    }

    pub fn generate_atomic_fetch_and(&mut self, dest: VarId, src: ValueId, value: ValueId, order: &MemoryOrder) {
        let src_reg = self.reg_allocator.get_reg(src).unwrap_or(X86Register::RAX);
        let value_reg = self.reg_allocator.get_reg(value).unwrap_or(X86Register::RCX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RDX);
        
        self.memory_order_to_fence(order);
        self.emit_lock_and(&format!("[{}]", Self::reg_to_str(src_reg)), &format!("{}", Self::reg_to_str(value_reg)));
        self.emit_mov_reg_to_reg(dest_reg, src_reg);
    }

    pub fn generate_atomic_fetch_or(&mut self, dest: VarId, src: ValueId, value: ValueId, order: &MemoryOrder) {
        let src_reg = self.reg_allocator.get_reg(src).unwrap_or(X86Register::RAX);
        let value_reg = self.reg_allocator.get_reg(value).unwrap_or(X86Register::RCX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RDX);
        
        self.memory_order_to_fence(order);
        self.emit_lock_or(&format!("[{}]", Self::reg_to_str(src_reg)), &format!("{}", Self::reg_to_str(value_reg)));
        self.emit_mov_reg_to_reg(dest_reg, src_reg);
    }

    pub fn generate_atomic_fetch_xor(&mut self, dest: VarId, src: ValueId, value: ValueId, order: &MemoryOrder) {
        let src_reg = self.reg_allocator.get_reg(src).unwrap_or(X86Register::RAX);
        let value_reg = self.reg_allocator.get_reg(value).unwrap_or(X86Register::RCX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RDX);
        
        self.memory_order_to_fence(order);
        self.emit_lock_xor(&format!("[{}]", Self::reg_to_str(src_reg)), &format!("{}", Self::reg_to_str(value_reg)));
        self.emit_mov_reg_to_reg(dest_reg, src_reg);
    }

    pub fn generate_atomic_compare_exchange(&mut self, dest: VarId, src: ValueId, expected: ValueId, desired: ValueId, success_order: &MemoryOrder, failure_order: &MemoryOrder) {
        let src_reg = self.reg_allocator.get_reg(src).unwrap_or(X86Register::RAX);
        let expected_reg = self.reg_allocator.get_reg(expected).unwrap_or(X86Register::RCX);
        let desired_reg = self.reg_allocator.get_reg(desired).unwrap_or(X86Register::RDX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::R8);
        
        self.memory_order_to_fence(success_order);
        self.emit_lock_cmpxchg(&format!("[{}]", Self::reg_to_str(src_reg)), &format!("{}", Self::reg_to_str(desired_reg)));
        self.emit_mov_reg_to_reg(dest_reg, src_reg);
    }

    pub fn generate_atomic_exchange(&mut self, dest: VarId, src: ValueId, value: ValueId, order: &MemoryOrder) {
        let src_reg = self.reg_allocator.get_reg(src).unwrap_or(X86Register::RAX);
        let value_reg = self.reg_allocator.get_reg(value).unwrap_or(X86Register::RCX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RDX);
        
        self.memory_order_to_fence(order);
        self.emit_lock_mov(&format!("[{}]", Self::reg_to_str(src_reg)), &format!("{}", Self::reg_to_str(value_reg)));
        self.emit_mov_reg_to_reg(dest_reg, src_reg);
    }

    pub fn generate_atomic_fence(&mut self, order: &MemoryOrder) {
        self.memory_order_to_fence(order);
    }

    pub fn generate_wait(&mut self, atomic: ValueId, timeout: Option<ValueId>) {
        let atomic_reg = self.reg_allocator.get_reg(atomic).unwrap_or(X86Register::RAX);
        
        self.emit_comment("Wait operation");
        self.emit_test(atomic_reg);
        self.emit_jne(&format!("_wait_{}", atomic_reg.0));
        
        if let Some(timeout) = timeout {
            let timeout_reg = self.reg_allocator.get_reg(timeout).unwrap_or(X86Register::RCX);
            self.emit_comment("Wait with timeout");
        }
    }

    pub fn generate_notify(&mut self, atomic: ValueId) {
        let atomic_reg = self.reg_allocator.get_reg(atomic).unwrap_or(X86Register::RAX);
        
        self.emit_comment("Notify operation");
        self.emit_lock_inc(&format!("[{}]", Self::reg_to_str(atomic_reg)));
    }

    pub fn generate_notify_all(&mut self, atomic: ValueId) {
        let atomic_reg = self.reg_allocator.get_reg(atomic).unwrap_or(X86Register::RAX);
        
        self.emit_comment("Notify all operation");
        self.emit_lock_inc(&format!("[{}]", Self::reg_to_str(atomic_reg)));
    }

    pub fn generate_iterator_next(&mut self, dest: VarId, iterator: ValueId) {
        let iterator_reg = self.reg_allocator.get_reg(iterator).unwrap_or(X86Register::RAX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RCX);
        
        self.emit_comment("Iterator next");
        self.emit_call("iterator_next");
        self.emit_mov_reg_to_reg(dest_reg, X86Register::RAX);
    }

    pub fn generate_iterator_collect(&mut self, dest: VarId, iterator: ValueId) {
        let iterator_reg = self.reg_allocator.get_reg(iterator).unwrap_or(X86Register::RAX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RCX);
        
        self.emit_comment("Iterator collect");
        self.emit_call("iterator_collect");
        self.emit_mov_reg_to_reg(dest_reg, X86Register::RAX);
    }

    pub fn generate_iterator_chain(&mut self, dest: VarId, iterator1: ValueId, iterator2: ValueId) {
        let iterator1_reg = self.reg_allocator.get_reg(iterator1).unwrap_or(X86Register::RAX);
        let iterator2_reg = self.reg_allocator.get_reg(iterator2).unwrap_or(X86Register::RCX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RDX);
        
        self.emit_comment("Iterator chain");
        self.emit_call("iterator_chain");
        self.emit_mov_reg_to_reg(dest_reg, X86Register::RAX);
    }

    pub fn generate_iterator_filter(&mut self, dest: VarId, iterator: ValueId, predicate: ValueId) {
        let iterator_reg = self.reg_allocator.get_reg(iterator).unwrap_or(X86Register::RAX);
        let predicate_reg = self.reg_allocator.get_reg(predicate).unwrap_or(X86Register::RCX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RDX);
        
        self.emit_comment("Iterator filter");
        self.emit_call("iterator_filter");
        self.emit_mov_reg_to_reg(dest_reg, X86Register::RAX);
    }

    pub fn generate_iterator_fold(&mut self, dest: VarId, iterator: ValueId, init: ValueId, accumulator: VarId, body: BlockId) {
        let iterator_reg = self.reg_allocator.get_reg(iterator).unwrap_or(X86Register::RAX);
        let init_reg = self.reg_allocator.get_reg(init).unwrap_or(X86Register::RCX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RDX);
        
        self.emit_comment("Iterator fold");
        self.emit_call("iterator_fold");
        self.emit_mov_reg_to_reg(dest_reg, X86Register::RAX);
    }

    pub fn generate_iterator_map(&mut self, dest: VarId, iterator: ValueId, mapper: ValueId) {
        let iterator_reg = self.reg_allocator.get_reg(iterator).unwrap_or(X86Register::RAX);
        let mapper_reg = self.reg_allocator.get_reg(mapper).unwrap_or(X86Register::RCX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RDX);
        
        self.emit_comment("Iterator map");
        self.emit_call("iterator_map");
        self.emit_mov_reg_to_reg(dest_reg, X86Register::RAX);
    }

    pub fn generate_result_ok(&mut self, dest: VarId, value: ValueId, ok_type: TypeId, err_type: TypeId) {
        let value_reg = self.reg_allocator.get_reg(value).unwrap_or(X86Register::RAX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RCX);
        
        self.emit_comment("Result ok");
        self.emit_mov_reg_to_reg(dest_reg, value_reg);
    }

    pub fn generate_result_err(&mut self, dest: VarId, error: ValueId, ok_type: TypeId, err_type: TypeId) {
        let error_reg = self.reg_allocator.get_reg(error).unwrap_or(X86Register::RAX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RCX);
        
        self.emit_comment("Result err");
        self.emit_mov_reg_to_reg(dest_reg, error_reg);
    }

    pub fn generate_try_catch(&mut self, dest: VarId, try_expr: ValueId, catch_block: BlockId, error_var: VarId) {
        let try_reg = self.reg_allocator.get_reg(try_expr).unwrap_or(X86Register::RAX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RCX);
        
        self.emit_comment("Try catch");
        self.emit_mov_reg_to_reg(dest_reg, try_reg);
    }

    pub fn generate_throw(&mut self, error: ValueId) {
        let error_reg = self.reg_allocator.get_reg(error).unwrap_or(X86Register::RAX);
        
        self.emit_comment("Throw error");
        self.emit_mov_reg_to_reg(X86Register::RDI, error_reg);
        self.emit_call("panic");
    }

    pub fn generate_future_await(&mut self, dest: VarId, future: ValueId) {
        let future_reg = self.reg_allocator.get_reg(future).unwrap_or(X86Register::RAX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RCX);
        
        self.emit_comment("Future await");
        self.emit_call("future_await");
        self.emit_mov_reg_to_reg(dest_reg, X86Register::RAX);
    }

    pub fn generate_yield(&mut self, value: Option<ValueId>) {
        if let Some(value_reg) = value.and_then(|v| self.reg_allocator.get_reg(v)) {
            self.emit_comment("Yield value");
            self.emit_mov_reg_to_reg(X86Register::RAX, value_reg);
        } else {
            self.emit_comment("Yield unit");
            self.emit_xor(&format!("{}", Self::reg_to_str(X86Register::RAX)), &format!("{}", Self::reg_to_str(X86Register::RAX)));
        }
    }

    pub fn generate_stream_yield(&mut self, value: Option<ValueId>) {
        if let Some(value_reg) = value.and_then(|v| self.reg_allocator.get_reg(v)) {
            self.emit_comment("Stream yield value");
            self.emit_mov_reg_to_reg(X86Register::RAX, value_reg);
        } else {
            self.emit_comment("Stream yield unit");
            self.emit_xor(&format!("{}", Self::reg_to_str(X86Register::RAX)), &format!("{}", Self::reg_to_str(X86Register::RAX)));
        }
    }

    pub fn generate_alloc(&mut self, dest: VarId, ty: TypeId, size: Option<ValueId>) {
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RAX);
        
        self.emit_comment("Alloc memory");
        self.emit_call("alloc");
        self.emit_mov_reg_to_reg(dest_reg, X86Register::RAX);
    }

    pub fn generate_free(&mut self, ptr: ValueId) {
        let ptr_reg = self.reg_allocator.get_reg(ptr).unwrap_or(X86Register::RAX);
        
        self.emit_comment("Free memory");
        self.emit_mov_reg_to_reg(X86Register::RDI, ptr_reg);
        self.emit_call("free");
    }

    pub fn generate_ptr_add(&mut self, dest: VarId, ptr: ValueId, offset: ValueId) {
        let ptr_reg = self.reg_allocator.get_reg(ptr).unwrap_or(X86Register::RAX);
        let offset_reg = self.reg_allocator.get_reg(offset).unwrap_or(X86Register::RCX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RDX);
        
        self.emit_comment("Pointer add");
        self.emit_add(&format!("{}", Self::reg_to_str(dest_reg)), &format!("[{} + {}]", Self::reg_to_str(ptr_reg), Self::reg_to_str(offset_reg)));
    }

    pub fn generate_ptr_sub(&mut self, dest: VarId, ptr1: ValueId, ptr2: ValueId) {
        let ptr1_reg = self.reg_allocator.get_reg(ptr1).unwrap_or(X86Register::RAX);
        let ptr2_reg = self.reg_allocator.get_reg(ptr2).unwrap_or(X86Register::RCX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RDX);
        
        self.emit_comment("Pointer sub");
        self.emit_sub(&format!("{}", Self::reg_to_str(dest_reg)), &format!("[{} - {}]", Self::reg_to_str(ptr1_reg), Self::reg_to_str(ptr2_reg)));
    }

    pub fn generate_ptr_load(&mut self, dest: VarId, ptr: ValueId, ty: TypeId) {
        let ptr_reg = self.reg_allocator.get_reg(ptr).unwrap_or(X86Register::RAX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RCX);
        
        self.emit_comment("Pointer load");
        self.emit_mov_mem_to_reg(dest_reg, &format!("[{}]", Self::reg_to_str(ptr_reg)));
    }

    pub fn generate_ptr_store(&mut self, ptr: ValueId, value: ValueId) {
        let ptr_reg = self.reg_allocator.get_reg(ptr).unwrap_or(X86Register::RAX);
        let value_reg = self.reg_allocator.get_reg(value).unwrap_or(X86Register::RCX);
        
        self.emit_comment("Pointer store");
        self.emit_mov(&format!("[{}]", Self::reg_to_str(ptr_reg)), &format!("{}", Self::reg_to_str(value_reg)));
    }

    pub fn generate_ptr_cast(&mut self, dest: VarId, ptr: ValueId, target_ty: TypeId) {
        let ptr_reg = self.reg_allocator.get_reg(ptr).unwrap_or(X86Register::RAX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RCX);
        
        self.emit_comment("Pointer cast");
        self.emit_mov_reg_to_reg(dest_reg, ptr_reg);
    }

    pub fn generate_ptr_offset_of(&mut self, dest: VarId, ptr: ValueId, field: &str) {
        let ptr_reg = self.reg_allocator.get_reg(ptr).unwrap_or(X86Register::RAX);
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RCX);
        
        self.emit_comment("Pointer offset of");
        self.emit_lea(dest_reg, &format!("[{} + {}]", Self::reg_to_str(ptr_reg), field));
    }

    pub fn generate_ptr_size_of(&mut self, dest: VarId, ty: TypeId) {
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RAX);
        
        self.emit_comment("Pointer size of");
        self.emit_mov_imm_to_reg(dest_reg, 8);
    }

    pub fn generate_align_of(&mut self, dest: VarId, ty: TypeId) {
        let dest_reg = self.reg_allocator.allocate(dest).unwrap_or(X86Register::RAX);
        
        self.emit_comment("Align of");
        self.emit_mov_imm_to_reg(dest_reg, 8);
    }

    pub fn generate_unsafe(&mut self, body: &BlockId) {
        self.emit_comment("Unsafe block");
        self.generate_block(body);
    }

    pub fn generate_proof(&mut self, proposition: ValueId, proof: ValueId) {
        let prop_reg = self.reg_allocator.get_reg(proposition).unwrap_or(X86Register::RAX);
        let proof_reg = self.reg_allocator.get_reg(proof).unwrap_or(X86Register::RCX);
        
        self.emit_comment("Proof");
        self.emit_mov_reg_to_reg(proof_reg, prop_reg);
    }

    pub fn generate_theorem(&mut self, name: &str, params: Vec<VarId>, proposition: ValueId, proof: ValueId) {
        self.emit_comment(&format!("Theorem: {}", name));
        self.generate_proof(proposition, proof);
    }

    pub fn generate_lemma(&mut self, name: &str, params: Vec<VarId>, proposition: ValueId, proof: ValueId) {
        self.emit_comment(&format!("Lemma: {}", name));
        self.generate_proof(proposition, proof);
    }

    pub fn generate_induction(&mut self, variable: VarId, base_case: ValueId, inductive_step: ValueId) {
        let var_reg = self.reg_allocator.get_reg(variable).unwrap_or(X86Register::RAX);
        let base_reg = self.reg_allocator.get_reg(base_case).unwrap_or(X86Register::RCX);
        let step_reg = self.reg_allocator.get_reg(inductive_step).unwrap_or(X86Register::RDX);
        
        self.emit_comment("Induction");
        self.emit_cmp(&format!("{}", Self::reg_to_str(var_reg)), &format!("{}", Self::reg_to_str(base_reg)));
        self.emit_je(&format!("_induction_base_{}", var_reg.0));
        self.emit_jmp(&format!("_induction_step_{}", var_reg.0));
        
        self.emit_label(&format!("_induction_base_{}", var_reg.0));
        self.emit_comment("Induction base case");
        self.emit_jmp(&format!("_induction_end_{}", var_reg.0));
        
        self.emit_label(&format!("_induction_step_{}", var_reg.0));
        self.emit_comment("Induction step");
        self.emit_jmp(&format!("_induction_end_{}", var_reg.0));
        
        self.emit_label(&format!("_induction_end_{}", var_reg.0));
    }

    pub fn generate_case(&mut self, value: ValueId, cases: Vec<crate::chim_ast::MatchCase>) {
        let value_reg = self.reg_allocator.get_reg(value).unwrap_or(X86Register::RAX);
        
        self.emit_comment("Case analysis");
        for (i, case) in cases.iter().enumerate() {
            if i > 0 {
                self.emit_jmp(&format!("_case_end_{}", value_reg.0));
            }
            self.emit_label(&format!("_case_{}", i));
        }
        self.emit_label(&format!("_case_end_{}", value_reg.0));
    }

    pub fn generate_refl(&mut self, ty: TypeId) {
        self.emit_comment("Reflexivity");
        self.emit_xor(&format!("{}", Self::reg_to_str(X86Register::RAX)), &format!("{}", Self::reg_to_str(X86Register::RAX)));
    }

    pub fn generate_cong(&mut self, ty: TypeId, expr1: ValueId, expr2: ValueId) {
        let expr1_reg = self.reg_allocator.get_reg(expr1).unwrap_or(X86Register::RAX);
        let expr2_reg = self.reg_allocator.get_reg(expr2).unwrap_or(X86Register::RCX);
        
        self.emit_comment("Congruence");
        self.emit_cmp(&format!("{}", Self::reg_to_str(expr1_reg)), &format!("{}", Self::reg_to_str(expr2_reg)));
    }

    pub fn generate_sym(&mut self, ty: TypeId, expr: ValueId) {
        let expr_reg = self.reg_allocator.get_reg(expr).unwrap_or(X86Register::RAX);
        
        self.emit_comment("Symmetry");
        self.emit_mov_reg_to_reg(X86Register::RCX, expr_reg);
    }

    pub fn generate_trans(&mut self, ty: TypeId, expr1: ValueId, expr2: ValueId, expr3: ValueId) {
        let expr1_reg = self.reg_allocator.get_reg(expr1).unwrap_or(X86Register::RAX);
        let expr2_reg = self.reg_allocator.get_reg(expr2).unwrap_or(X86Register::RCX);
        let expr3_reg = self.reg_allocator.get_reg(expr3).unwrap_or(X86Register::RDX);
        
        self.emit_comment("Transitivity");
        self.emit_cmp(&format!("{}", Self::reg_to_str(expr1_reg)), &format!("{}", Self::reg_to_str(expr2_reg)));
    }

    pub fn generate_rec(&mut self, ty: TypeId, body: &BlockId) {
        self.emit_comment("Recursion");
        self.generate_block(body);
    }

    pub fn generate_fix(&mut self, ty: TypeId, body: &BlockId) {
        self.emit_comment("Fixpoint");
        self.generate_block(body);
    }

    pub fn generate_class(&mut self, name: &str, params: Vec<VarId>, methods: Vec<crate::chim_ast::Function>) {
        self.emit_comment(&format!("Class: {}", name));
        
        for method in methods {
            self.emit_comment(&format!("Method: {}", method.name));
        }
    }

    pub fn generate_instance(&mut self, class_name: &str, ty: TypeId, methods: Vec<crate::chim_ast::Function>) {
        self.emit_comment(&format!("Instance of {}", class_name));
        
        for method in methods {
            self.emit_comment(&format!("Method: {}", method.name));
        }
    }

    pub fn generate_where(&mut self, expr: ValueId, constraints: Vec<ValueId>) {
        let expr_reg = self.reg_allocator.get_reg(expr).unwrap_or(X86Register::RAX);
        
        self.emit_comment("Where constraints");
        for constraint in constraints {
            self.emit_comment(&format!("Constraint: {:?}", constraint));
        }
    }

    pub fn generate_eq_prop(&mut self, ty: TypeId, left: ValueId, right: ValueId) {
        let left_reg = self.reg_allocator.get_reg(left).unwrap_or(X86Register::RAX);
        let right_reg = self.reg_allocator.get_reg(right).unwrap_or(X86Register::RCX);
        
        self.emit_comment("Equality proposition");
        self.emit_cmp(&format!("{}", Self::reg_to_str(left_reg)), &format!("{}", Self::reg_to_str(right_reg)));
    }

    pub fn generate_refl_prop(&mut self, ty: TypeId, expr: ValueId) {
        let expr_reg = self.reg_allocator.get_reg(expr).unwrap_or(X86Register::RAX);
        
        self.emit_comment("Reflexivity proposition");
        self.emit_mov_reg_to_reg(X86Register::RCX, expr_reg);
    }

    pub fn generate_jmeq(&mut self, ty: TypeId, expr1: ValueId, expr2: ValueId) {
        let expr1_reg = self.reg_allocator.get_reg(expr1).unwrap_or(X86Register::RAX);
        let expr2_reg = self.reg_allocator.get_reg(expr2).unwrap_or(X86Register::RCX);
        
        self.emit_comment("Judgmental equality");
        self.emit_cmp(&format!("{}", Self::reg_to_str(expr1_reg)), &format!("{}", Self::reg_to_str(expr2_reg)));
    }

    pub fn generate_rewrite(&mut self, ty: TypeId, expr: ValueId, rule: ValueId) {
        let expr_reg = self.reg_allocator.get_reg(expr).unwrap_or(X86Register::RAX);
        let rule_reg = self.reg_allocator.get_reg(rule).unwrap_or(X86Register::RCX);
        
        self.emit_comment("Rewrite rule");
        self.emit_mov_reg_to_reg(X86Register::RCX, expr_reg);
    }

    pub fn generate_with(&mut self, expr: ValueId, bindings: Vec<(VarId, ValueId)>) {
        let expr_reg = self.reg_allocator.get_reg(expr).unwrap_or(X86Register::RAX);
        
        self.emit_comment("With bindings");
        for (var, value) in bindings {
            let value_reg = self.reg_allocator.get_reg(value).unwrap_or(X86Register::RCX);
            self.emit_mov_reg_to_reg(X86Register::RDX, value_reg);
        }
    }
}

pub struct X86CodeGeneratorWrapper;

impl X86CodeGeneratorWrapper {
    pub fn new() -> Self {
        X86CodeGeneratorWrapper
    }
}

impl CodeGenerator for X86CodeGeneratorWrapper {
    fn generate(&self, module: &IRModule, _program: &AnalyzedProgram) -> Result<GeneratedCode, CodegenError> {
        let mut generator = X86CodeGenerator::new();
        
        let mut output = String::new();
        output.push_str("; x86_64 assembly generated by Chim Compiler\n");
        output.push_str("; Target: x86_64\n");
        output.push_str("; Syntax: AT&T\n");
        output.push_str("\n");
        
        output.push_str(".section .text\n");
        output.push_str(".global main\n");
        output.push_str("\n");
        
        for func in &module.functions {
            generator.generate_function(func);
        }
        
        for inst in &generator.instructions {
            output.push_str(&inst.to_asm());
            output.push('\n');
        }
        
        Ok(GeneratedCode {
            source: output,
            extension: String::from("asm"),
            language: String::from("Assembly (x86_64)"),
            is_executable: false,
        })
    }

    fn name(&self) -> &str {
        "x86_64"
    }

    fn file_extension(&self) -> &str {
        "asm"
    }

    fn target(&self) -> CodegenTarget {
        CodegenTarget::X86_64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x86_code_generator_creation() {
        let generator = X86CodeGenerator::new();
        assert_eq!(generator.instructions.len(), 0);
    }

    #[test]
    fn test_register_allocator() {
        let mut allocator = RegisterAllocator::new();
        let var = VarId(0);
        let reg = allocator.allocate(var).unwrap();
        assert!(allocator.used_regs.contains(&reg));
    }

    #[test]
    fn test_stack_frame() {
        let mut frame = StackFrame::new();
        let offset = frame.allocate_local(8);
        assert!(offset < 0);
        assert!(frame.local_size >= 8);
    }

    #[test]
    fn test_x86_instruction() {
        let inst = X86Instruction::new("mov");
        assert_eq!(inst.mnemonic, "mov");
        assert!(inst.operands.is_empty());
    }

    #[test]
    fn test_x86_instruction_with_operands() {
        let inst = X86Instruction::with_operands("add", vec!["rax".to_string(), "rcx".to_string()]);
        assert_eq!(inst.mnemonic, "add");
        assert_eq!(inst.operands.len(), 2);
    }

    #[test]
    fn test_x86_instruction_with_comment() {
        let inst = X86Instruction::with_comment("nop", "Do nothing");
        assert_eq!(inst.mnemonic, "nop");
        assert_eq!(inst.comment, Some("Do nothing".to_string()));
    }

    #[test]
    fn test_x86_instruction_to_asm() {
        let inst = X86Instruction::with_operands("mov", vec!["rax".to_string(), "rbx".to_string()]);
        let asm = inst.to_asm();
        assert!(asm.contains("mov rax, rbx"));
    }

    #[test]
    fn test_reg_to_str() {
        assert_eq!(X86CodeGenerator::reg_to_str(X86Register::RAX), "rax");
        assert_eq!(X86CodeGenerator::reg_to_str(X86Register::RCX), "rcx");
        assert_eq!(X86CodeGenerator::reg_to_str(X86Register::XMM0), "xmm0");
    }

    #[test]
    fn test_x86_register_is_callee_saved() {
        assert!(X86Register::RBX.is_callee_saved());
        assert!(X86Register::RBP.is_callee_saved());
        assert!(X86Register::R12.is_callee_saved());
        assert!(!X86Register::RAX.is_callee_saved());
    }

    #[test]
    fn test_x86_register_is_arg() {
        assert!(X86Register::RDI.is_arg());
        assert!(X86Register::RSI.is_arg());
        assert!(!X86Register::RAX.is_arg());
    }

    #[test]
    fn test_x86_register_is_temp() {
        assert!(X86Register::RAX.is_temp());
        assert!(X86Register::R10.is_temp());
        assert!(!X86Register::RDI.is_temp());
    }

    #[test]
    fn test_register_allocator_allocate() {
        let mut allocator = RegisterAllocator::new();
        let var = VarId(0);
        let reg = allocator.allocate(var).unwrap();
        assert!(allocator.used_regs.contains(&reg));
        assert!(allocator.var_to_reg.get(&var).is_some());
        assert!(allocator.reg_to_var.get(&reg).is_some());
    }

    #[test]
    fn test_register_allocator_free() {
        let mut allocator = RegisterAllocator::new();
        let var = VarId(0);
        let reg = allocator.allocate(var).unwrap();
        allocator.free(reg);
        assert!(!allocator.used_regs.contains(&reg));
        assert!(allocator.var_to_reg.get(&var).is_none());
        assert!(allocator.reg_to_var.get(&reg).is_none());
    }

    #[test]
    fn test_stack_frame_allocate_local() {
        let mut frame = StackFrame::new();
        let offset1 = frame.allocate_local(8);
        let offset2 = frame.allocate_local(16);
        assert!(offset1 < offset2);
        assert!(frame.local_size >= 24);
    }

    #[test]
    fn test_stack_frame_allocate_arg() {
        let mut frame = StackFrame::new();
        let var1 = VarId(0);
        let var2 = VarId(1);
        let offset1 = frame.allocate_arg(var1, 8);
        let offset2 = frame.allocate_arg(var2, 8);
        assert_eq!(offset1, 8);
        assert_eq!(offset2, 16);
    }

    #[test]
    fn test_x86_code_generator_emit_mov() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_mov_reg_to_reg(X86Register::RAX, X86Register::RCX);
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "mov");
    }

    #[test]
    fn test_x86_code_generator_emit_add() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_add("rax", "rcx");
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "add");
    }

    #[test]
    fn test_x86_code_generator_emit_cmp() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_cmp("rax", "rcx");
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "cmp");
    }

    #[test]
    fn test_x86_code_generator_emit_jmp() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_jmp(".L1");
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "jmp");
    }

    #[test]
    fn test_x86_code_generator_emit_je() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_je(".L1");
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "je");
    }

    #[test]
    fn test_x86_code_generator_emit_call() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_call("main");
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "call");
    }

    #[test]
    fn test_x86_code_generator_emit_ret() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_ret();
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "ret");
    }

    #[test]
    fn test_x86_code_generator_emit_push() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_push(X86Register::RAX);
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "push");
    }

    #[test]
    fn test_x86_code_generator_emit_pop() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_pop(X86Register::RAX);
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "pop");
    }

    #[test]
    fn test_x86_code_generator_emit_test() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_test(X86Register::RAX);
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "test");
    }

    #[test]
    fn test_x86_code_generator_emit_lock() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_lock("add", vec!["[rax]".to_string(), "1".to_string()]);
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "lock");
    }

    #[test]
    fn test_x86_code_generator_emit_lock_cmpxchg() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_lock_cmpxchg("[rax]", "rcx");
        assert_eq!(generator.instructions.len(), 1);
        assert!(generator.instructions[0].operands[0], "lock");
    }

    #[test]
    fn test_x86_code_generator_emit_mfence() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_mfence();
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "mfence");
    }

    #[test]
    fn test_x86_code_generator_emit_sfence() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_sfence();
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "sfence");
    }

    #[test]
    fn test_x86_code_generator_emit_lfence() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_lfence();
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "lfence");
    }

    #[test]
    fn test_x86_code_generator_emit_lea() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_lea(X86Register::RAX, "[rbx + rcx]");
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "lea");
    }

    #[test]
    fn test_x86_code_generator_emit_and() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_and("rax", "rcx");
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "and");
    }

    #[test]
    fn test_x86_code_generator_emit_or() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_or("rax", "rcx");
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "or");
    }

    #[test]
    fn test_x86_code_generator_emit_xor() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_xor("rax", "rcx");
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "xor");
    }

    #[test]
    fn test_x86_code_generator_emit_not() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_not("rax");
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "not");
    }

    #[test]
    fn test_x86_code_generator_emit_neg() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_neg("rax");
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "neg");
    }

    #[test]
    fn test_x86_code_generator_emit_shl() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_shl("rax", "cl");
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "shl");
    }

    #[test]
    fn test_x86_code_generator_emit_shr() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_shr("rax", "cl");
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "shr");
    }

    #[test]
    fn test_x86_code_generator_generate_label() {
        let mut generator = X86CodeGenerator::new();
        let label1 = generator.generate_label();
        let label2 = generator.generate_label();
        assert_ne!(label1, label2);
        assert!(label1.starts_with(".L"));
    }

    #[test]
    fn test_x86_code_generator_emit_comment() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_comment("Test comment");
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].comment, Some("Test comment".to_string()));
    }

    #[test]
    fn test_x86_code_generator_emit_label_direct() {
        let mut generator = X86CodeGenerator::new();
        generator.emit_label(".L1");
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, ".L1");
    }

    #[test]
    fn test_x86_code_generator_memory_order_to_fence() {
        let mut generator = X86CodeGenerator::new();
        generator.memory_order_to_fence(&MemoryOrder::Acquire);
        assert_eq!(generator.instructions.len(), 1);
        assert_eq!(generator.instructions[0].mnemonic, "lfence");
    }

    #[test]
    fn test_x86_code_generator_wrapper() {
        let wrapper = X86CodeGeneratorWrapper::new();
        assert_eq!(wrapper.name(), "x86_64");
        assert_eq!(wrapper.file_extension(), "asm");
        assert_eq!(wrapper.target(), CodegenTarget::X86_64);
    }
}
