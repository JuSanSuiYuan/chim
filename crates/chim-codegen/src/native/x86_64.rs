use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X86_64Register {
    RAX, RBX, RCX, RDX, RSI, RDI, RBP, RSP,
    R8, R9, R10, R11, R12, R13, R14, R15,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum X86_64Instruction {
    MOV { dst: Operand, src: Operand },
    ADD { dst: Operand, src: Operand },
    SUB { dst: Operand, src: Operand },
    IMUL { dst: Operand, src: Operand },
    IDIV { operand: Operand },
    AND { dst: Operand, src: Operand },
    OR { dst: Operand, src: Operand },
    XOR { dst: Operand, src: Operand },
    NOT { operand: Operand },
    NEG { operand: Operand },
    SHL { dst: Operand, src: Operand },
    SHR { dst: Operand, src: Operand },
    SAR { dst: Operand, src: Operand },
    MOVSX { dst: Operand, src: Operand },
    MOVZX { dst: Operand, src: Operand },
    LEA { dst: Operand, src: Operand },
    PUSH { operand: Operand },
    POP { operand: Operand },
    CALL { target: Operand },
    RET,
    JMP { target: Operand },
    JE { target: Operand },
    JNE { target: Operand },
    JL { target: Operand },
    JLE { target: Operand },
    JG { target: Operand },
    JGE { target: Operand },
    JA { target: Operand },
    JAE { target: Operand },
    JB { target: Operand },
    JBE { target: Operand },
    JO { target: Operand },
    JNO { target: Operand },
    CMP { dst: Operand, src: Operand },
    TEST { dst: Operand, src: Operand },
    SETE { dst: Operand },
    SETNE { dst: Operand },
    SETL { dst: Operand },
    SETLE { dst: Operand },
    SETG { dst: Operand },
    SETGE { dst: Operand },
    SETA { dst: Operand },
    SETAE { dst: Operand },
    SETB { dst: Operand },
    SETBE { dst: Operand },
    NOP,
    LABEL(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    Register(X86_64Register),
    Memory(MemoryOperand),
    Immediate(i64),
    Immediate64(u64),
    Label(String),
    RIPRelative(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryOperand {
    pub base: Option<X86_64Register>,
    pub index: Option<X86_64Register>,
    pub scale: u8,
    pub displacement: i32,
}

#[derive(Debug)]
pub struct X86_64CodeGenerator {
    instructions: Vec<X86_64Instruction>,
    current_function: Option<String>,
    label_count: u32,
    pool_labels: HashMap<String, u64>,
    code_position: u64,
}

impl X86_64CodeGenerator {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            current_function: None,
            label_count: 0,
            pool_labels: HashMap::new(),
            code_position: 0,
        }
    }

    pub fn start_function(&mut self, name: &str) {
        self.current_function = Some(name.to_string());
        self.add_instr(X86_64Instruction::LABEL(name.to_string()));
    }

    pub fn end_function(&mut self) {
        self.current_function = None;
    }

    pub fn generate_prologue(&mut self, frame_size: u32) {
        self.push(X86_64Register::RBP);
        self.mov_reg_to_reg(X86_64Register::RSP, X86_64Register::RBP);
        if frame_size > 0 {
            self.sub_imm_from_reg(frame_size as i64, X86_64Register::RSP);
        }
    }

    pub fn generate_epilogue(&mut self) {
        self.mov_reg_to_reg(X86_64Register::RBP, X86_64Register::RSP);
        self.pop(X86_64Register::RBP);
        self.ret();
    }

    pub fn mov_reg_to_reg(&mut self, dst: X86_64Register, src: X86_64Register) {
        self.instructions.push(X86_64Instruction::MOV {
            dst: Operand::Register(dst),
            src: Operand::Register(src),
        });
    }

    pub fn mov_imm_to_reg(&mut self, imm: i64, dst: X86_64Register) {
        self.instructions.push(X86_64Instruction::MOV {
            dst: Operand::Register(dst),
            src: Operand::Immediate(imm),
        });
    }

    pub fn mov_imm_to_mem(&mut self, imm: i64, mem: MemoryOperand) {
        self.instructions.push(X86_64Instruction::MOV {
            dst: Operand::Memory(mem),
            src: Operand::Immediate(imm),
        });
    }

    pub fn mov_reg_to_mem(&mut self, src: X86_64Register, mem: MemoryOperand) {
        self.instructions.push(X86_64Instruction::MOV {
            dst: Operand::Memory(mem),
            src: Operand::Register(src),
        });
    }

    pub fn mov_mem_to_reg(&mut self, mem: MemoryOperand, dst: X86_64Register) {
        self.instructions.push(X86_64Instruction::MOV {
            dst: Operand::Register(dst),
            src: Operand::Memory(mem),
        });
    }

    pub fn add_reg_to_reg(&mut self, dst: X86_64Register, src: X86_64Register) {
        self.add(Operand::Register(dst), Operand::Register(src));
    }

    pub fn add_imm_to_reg(&mut self, imm: i64, dst: X86_64Register) {
        self.add(Operand::Register(dst), Operand::Immediate(imm));
    }

    pub fn add_imm_from_reg(&mut self, src: X86_64Register, imm: i64, dst: X86_64Register) {
        self.mov_reg_to_reg(dst, src);
        self.add_imm_to_reg(imm, dst);
    }

    pub fn add(&mut self, dst: Operand, src: Operand) {
        self.instructions.push(X86_64Instruction::ADD { dst, src });
    }

    pub fn sub_reg_from_reg(&mut self, dst: X86_64Register, src: X86_64Register) {
        self.sub(Operand::Register(dst), Operand::Register(src));
    }

    pub fn sub_imm_from_reg(&mut self, imm: i64, dst: X86_64Register) {
        self.sub(Operand::Register(dst), Operand::Immediate(imm));
    }

    pub fn sub(&mut self, dst: Operand, src: Operand) {
        self.instructions.push(X86_64Instruction::SUB { dst, src });
    }

    pub fn imul_reg_by_reg(&mut self, dst: X86_64Register, src: X86_64Register) {
        self.instructions.push(X86_64Instruction::IMUL {
            dst: Operand::Register(dst),
            src: Operand::Register(src),
        });
    }

    pub fn imul_reg_by_imm(&mut self, dst: X86_64Register, imm: i32) {
        self.instructions.push(X86_64Instruction::IMUL {
            dst: Operand::Register(dst),
            src: Operand::Immediate(imm as i64),
        });
    }

    pub fn idiv_by_reg(&mut self, divisor: X86_64Register) {
        self.instructions.push(X86_64Instruction::IDIV {
            operand: Operand::Register(divisor),
        });
    }

    pub fn and_reg_with_reg(&mut self, dst: X86_64Register, src: X86_64Register) {
        self.and(Operand::Register(dst), Operand::Register(src));
    }

    pub fn and(&mut self, dst: Operand, src: Operand) {
        self.instructions.push(X86_64Instruction::AND { dst, src });
    }

    pub fn or_reg_with_reg(&mut self, dst: X86_64Register, src: X86_64Register) {
        self.or(Operand::Register(dst), Operand::Register(src));
    }

    pub fn or(&mut self, dst: Operand, src: Operand) {
        self.instructions.push(X86_64Instruction::OR { dst, src });
    }

    pub fn xor_reg_with_reg(&mut self, dst: X86_64Register, src: X86_64Register) {
        self.xor(Operand::Register(dst), Operand::Register(src));
    }

    pub fn xor(&mut self, dst: Operand, src: Operand) {
        self.instructions.push(X86_64Instruction::XOR { dst, src });
    }

    pub fn not_reg(&mut self, reg: X86_64Register) {
        self.not(Operand::Register(reg));
    }

    pub fn not(&mut self, operand: Operand) {
        self.instructions.push(X86_64Instruction::NOT { operand });
    }

    pub fn neg_reg(&mut self, reg: X86_64Register) {
        self.neg(Operand::Register(reg));
    }

    pub fn neg(&mut self, operand: Operand) {
        self.instructions.push(X86_64Instruction::NEG { operand });
    }

    pub fn shl_reg_by_imm(&mut self, dst: X86_64Register, imm: u8) {
        self.shl(Operand::Register(dst), Operand::Immediate(imm as i64));
    }

    pub fn shl(&mut self, dst: Operand, src: Operand) {
        self.instructions.push(X86_64Instruction::SHL { dst, src });
    }

    pub fn shr_reg_by_imm(&mut self, dst: X86_64Register, imm: u8) {
        self.shr(Operand::Register(dst), Operand::Immediate(imm as i64));
    }

    pub fn shr(&mut self, dst: Operand, src: Operand) {
        self.instructions.push(X86_64Instruction::SHR { dst, src });
    }

    pub fn sar_reg_by_imm(&mut self, dst: X86_64Register, imm: u8) {
        self.sar(Operand::Register(dst), Operand::Immediate(imm as i64));
    }

    pub fn sar(&mut self, dst: Operand, src: Operand) {
        self.instructions.push(X86_64Instruction::SAR { dst, src });
    }

    pub fn push(&mut self, operand: Operand) {
        self.instructions.push(X86_64Instruction::PUSH { operand });
    }

    pub fn pop(&mut self, operand: Operand) {
        self.instructions.push(X86_64Instruction::POP { operand });
    }

    pub fn call(&mut self, target: Operand) {
        self.instructions.push(X86_64Instruction::CALL { target });
    }

    pub fn call_label(&mut self, label: &str) {
        self.call(Operand::Label(label.to_string()));
    }

    pub fn ret(&mut self) {
        self.instructions.push(X86_64Instruction::RET);
    }

    pub fn jmp(&mut self, target: Operand) {
        self.instructions.push(X86_64Instruction::JMP { target });
    }

    pub fn jmp_label(&mut self, label: &str) {
        self.jmp(Operand::Label(label.to_string()));
    }

    pub fn je(&mut self, target: Operand) {
        self.instructions.push(X86_64Instruction::JE { target });
    }

    pub fn jne(&mut self, target: Operand) {
        self.instructions.push(X86_64Instruction::JNE { target });
    }

    pub fn jl(&mut self, target: Operand) {
        self.instructions.push(X86_64Instruction::JL { target });
    }

    pub fn jle(&mut self, target: Operand) {
        self.instructions.push(X86_64Instruction::JLE { target });
    }

    pub fn jg(&mut self, target: Operand) {
        self.instructions.push(X86_64Instruction::JG { target });
    }

    pub fn jge(&mut self, target: Operand) {
        self.instructions.push(X86_64Instruction::JGE { target });
    }

    pub fn cmp_reg_reg(&mut self, dst: X86_64Register, src: X86_64Register) {
        self.cmp(Operand::Register(dst), Operand::Register(src));
    }

    pub fn cmp_reg_imm(&mut self, dst: X86_64Register, imm: i32) {
        self.cmp(Operand::Register(dst), Operand::Immediate(imm as i64));
    }

    pub fn cmp(&mut self, dst: Operand, src: Operand) {
        self.instructions.push(X86_64Instruction::CMP { dst, src });
    }

    pub fn test_reg_reg(&mut self, dst: X86_64Register, src: X86_64Register) {
        self.test(Operand::Register(dst), Operand::Register(src));
    }

    pub fn test(&mut self, dst: Operand, src: Operand) {
        self.instructions.push(X86_64Instruction::TEST { dst, src });
    }

    pub fn sete(&mut self, dst: Operand) {
        self.instructions.push(X86_64Instruction::SETE { dst });
    }

    pub fn setne(&mut self, dst: Operand) {
        self.instructions.push(X86_64Instruction::SETNE { dst });
    }

    pub fn setl(&mut self, dst: Operand) {
        self.instructions.push(X86_64Instruction::SETL { dst });
    }

    pub fn setg(&mut self, dst: Operand) {
        self.instructions.push(X86_64Instruction::SETG { dst });
    }

    pub fn seta(&mut self, dst: Operand) {
        self.instructions.push(X86_64Instruction::SETA { dst });
    }

    pub fn setb(&mut self, dst: Operand) {
        self.instructions.push(X86_64Instruction::SETB { dst });
    }

    pub fn emit_label(&mut self, label: &str) {
        self.instructions.push(X86_64Instruction::LABEL(label.to_string()));
    }

    pub fn new_label(&mut self, prefix: &str) -> String {
        self.label_count += 1;
        format!("{}_{}", prefix, self.label_count)
    }

    pub fn nop(&mut self) {
        self.instructions.push(X86_64Instruction::NOP);
    }

    pub fn memory_operand(&self, base: Option<X86_64Register>, disp: i32) -> MemoryOperand {
        MemoryOperand {
            base,
            index: None,
            scale: 1,
            displacement: disp,
        }
    }

    pub fn memory_indexed(&self, base: Option<X86_64Register>, index: X86_64Register, scale: u8, disp: i32) -> MemoryOperand {
        MemoryOperand {
            base,
            index: Some(index),
            scale,
            displacement: disp,
        }
    }

    pub fn emit(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for instr in &self.instructions {
            self.encode_instruction(instr, &mut bytes);
        }
        bytes
    }

    fn encode_instruction(&self, instr: &X86_64Instruction, bytes: &mut Vec<u8>) {
        match instr {
            X86_64Instruction::MOV { dst, src } => {
                self.encode_mov(dst, src, bytes);
            }
            X86_64Instruction::ADD { dst, src } => {
                self.encode_add(dst, src, bytes);
            }
            X86_64Instruction::SUB { dst, src } => {
                self.encode_sub(dst, src, bytes);
            }
            X86_64Instruction::RET => {
                bytes.push(0xC3);
            }
            X86_64Instruction::NOP => {
                bytes.push(0x90);
            }
            X86_64Instruction::LABEL(name) => {
                self.pool_labels.insert(name.clone(), self.code_position);
            }
            _ => {}
        }
        self.code_position += bytes.len() as u64;
    }

    fn encode_mov(&self, dst: &Operand, src: &Operand, bytes: &mut Vec<u8>) {
        match (dst, src) {
            (Operand::Register(r1), Operand::Register(r2)) => {
                bytes.push(0x48);
                bytes.push(0x89);
                bytes.push(self.encode_modrm(3, *r1 as u8, *r2 as u8));
            }
            (Operand::Register(r), Operand::Immediate(imm)) => {
                bytes.push(0x48);
                bytes.push(0xC7);
                bytes.push(self.encode_modrm(3, 0, *r as u8));
                bytes.extend_from_slice(&(*imm as i32).to_le_bytes());
            }
            (Operand::Memory(mem), Operand::Immediate(imm)) => {
                bytes.push(0x48);
                bytes.push(0xC7);
                let rm = self.encode_memory_modrm(0, mem);
                bytes.push(rm);
                bytes.extend_from_slice(&(*imm as i32).to_le_bytes());
            }
            _ => {}
        }
    }

    fn encode_add(&self, dst: &Operand, src: &Operand, bytes: &mut Vec<u8>) {
        match (dst, src) {
            (Operand::Register(r1), Operand::Register(r2)) => {
                bytes.push(0x48);
                bytes.push(0x01);
                bytes.push(self.encode_modrm(3, *r1 as u8, *r2 as u8));
            }
            (Operand::Register(r), Operand::Immediate(imm)) => {
                bytes.push(0x48);
                bytes.push(0x81);
                bytes.push(self.encode_modrm(3, 0, *r as u8));
                bytes.extend_from_slice(&(*imm as i32).to_le_bytes());
            }
            _ => {}
        }
    }

    fn encode_sub(&self, dst: &Operand, src: &Operand, bytes: &mut Vec<u8>) {
        match (dst, src) {
            (Operand::Register(r1), Operand::Register(r2)) => {
                bytes.push(0x48);
                bytes.push(0x29);
                bytes.push(self.encode_modrm(3, *r1 as u8, *r2 as u8));
            }
            (Operand::Register(r), Operand::Immediate(imm)) => {
                bytes.push(0x48);
                bytes.push(0x81);
                bytes.push(self.encode_modrm(3, 5, *r as u8));
                bytes.extend_from_slice(&(*imm as i32).to_le_bytes());
            }
            _ => {}
        }
    }

    fn encode_modrm(&self, mod_: u8, reg: u8, rm: u8) -> u8 {
        ((mod_ & 0x3) << 6) | ((reg & 0x7) << 3) | (rm & 0x7)
    }

    fn encode_memory_modrm(&self, reg: u8, mem: &MemoryOperand) -> u8 {
        let mut modrm = self.encode_modrm(0, reg, mem.base.unwrap_or(X86_64Register::RAX) as u8);
        if let Some(index) = mem.index {
            modrm |= 0x04;
            modrm |= (index as u8 & 0x7) << 3;
        }
        modrm
    }

    fn add_instr(&mut self, instr: X86_64Instruction) {
        self.instructions.push(instr);
    }
}

pub fn generate_x86_64_code(
    ir: &crate::IRModule,
    target: crate::Target,
) -> Vec<u8> {
    let mut codegen = X86_64CodeGenerator::new();
    
    for func in &ir.functions {
        codegen.start_function(&func.name);
        
        codegen.generate_prologue(0);
        
        for instr in &func.instructions {
            codegen.add_instr(instr.clone().into());
        }
        
        codegen.generate_epilogue();
        codegen.end_function();
    }
    
    codegen.emit()
}

impl From<crate::IRInstruction> for X86_64Instruction {
    fn from(instr: crate::IRInstruction) -> Self {
        match instr {
            crate::IRInstruction::Mov { dst, src } => X86_64Instruction::MOV {
                dst: dst.into(),
                src: src.into(),
            },
            crate::IRInstruction::Add { dst, src } => X86_64Instruction::ADD {
                dst: dst.into(),
                src: src.into(),
            },
            crate::IRInstruction::Sub { dst, src } => X86_64Instruction::SUB {
                dst: dst.into(),
                src: src.into(),
            },
            crate::IRInstruction::Ret => X86_64Instruction::RET,
            _ => X86_64Instruction::NOP,
        }
    }
}

impl From<crate::IROperand> for Operand {
    fn from(op: crate::IROperand) -> Self {
        match op {
            crate::IROperand::Register(r) => Operand::Register(match r.id {
                0 => X86_64Register::RAX,
                1 => X86_64Register::RBX,
                2 => X86_64Register::RCX,
                3 => X86_64Register::RDX,
                _ => X86_64Register::RAX,
            }),
            crate::IROperand::Immediate(i) => Operand::Immediate(i),
            crate::IROperand::Label(l) => Operand::Label(l),
        }
    }
}
