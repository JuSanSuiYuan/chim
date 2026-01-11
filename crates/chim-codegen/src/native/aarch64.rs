use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AArch64Register {
    X0, X1, X2, X3, X4, X5, X6, X7, X8, X9, X10, X11, X12, X13, X14, X15,
    X16, X17, X18, X19, X20, X21, X22, X23, X24, X25, X26, X27, X28, X29, X30, SP,
    W0, W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12, W13, W14, W15,
    W16, W17, W18, W19, W20, W21, W22, W23, W24, W25, W26, W27, W28, W29, W30, WSP,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AArch64Instruction {
    MOV { dst: AArch64Operand, src: AArch64Operand },
    ADD { dst: AArch64Operand, src1: AArch64Operand, src2: AArch64Operand },
    SUB { dst: AArch64Operand, src1: AArch64Operand, src2: AArch64Operand },
    MUL { dst: AArch64Operand, src1: AArch64Operand, src2: AArch64Operand },
    SDIV { dst: AArch64Operand, src1: AArch64Operand, src2: AArch64Operand },
    AND { dst: AArch64Operand, src1: AArch64Operand, src2: AArch64Operand },
    ORR { dst: AArch64Operand, src1: AArch64Operand, src2: AArch64Operand },
    EOR { dst: AArch64Operand, src1: AArch64Operand, src2: AArch64Operand },
    NOT { dst: AArch64Operand, src: AArch64Operand },
    NEG { dst: AArch64Operand, src: AArch64Operand },
    LSL { dst: AArch64Operand, src: AArch64Operand, shift: u8 },
    LSR { dst: AArch64Operand, src: AArch64Operand, shift: u8 },
    ASR { dst: AArch64Operand, src: AArch64Operand, shift: u8 },
    SXTW { dst: AArch64Operand, src: AArch64Operand },
    UXTW { dst: AArch64Operand, src: AArch64Operand },
    LDR { dst: AArch64Operand, src: AArch64Operand },
    STR { src: AArch64Operand, dst: AArch64Operand },
    ADR { dst: AArch64Operand, label: String },
    ADRP { dst: AArch64Operand, label: String },
    B { target: String },
    BL { target: String },
    BLR { target: AArch64Register },
    BR { target: AArch64Register },
    RET { target: Option<AArch64Register> },
    CBZ { src: AArch64Operand, target: String },
    CBNZ { src: AArch64Operand, target: String },
    TBZ { src: AArch64Operand, bit: u8, target: String },
    TBNZ { src: AArch64Operand, bit: u8, target: String },
    CMP { src1: AArch64Operand, src2: AArch64Operand },
    CCMPI { src1: AArch64Operand, src2: AArch64Operand, nzcv: u8 },
    CSEL { dst: AArch64Operand, cond: String, src1: AArch64Operand, src2: AArch64Operand },
    CSET { dst: AArch64Operand, cond: String },
    CSINC { dst: AArch64Operand, cond: String, src1: AArch64Operand, src2: AArch64Operand },
    FADD { dst: AArch64Operand, src1: AArch64Operand, src2: AArch64Operand },
    FSUB { dst: AArch64Operand, src1: AArch64Operand, src2: AArch64Operand },
    FMUL { dst: AArch64Operand, src1: AArch64Operand, src2: AArch64Operand },
    FDIV { dst: AArch64Operand, src1: AArch64Operand, src2: AArch64Operand },
    FMOV { dst: AArch64Operand, src: AArch64Operand },
    FCVT { dst: AArch64Operand, src: AArch64Operand },
    SCVTF { dst: AArch64Operand, src: AArch64Operand },
    FCVTS { dst: AArch64Operand, src: AArch64Operand },
    NOP,
    LABEL(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AArch64Operand {
    Register(AArch64Register),
    Memory(MemoryOperand),
    Immediate(i64),
    Immediate64(u64),
    Label(String),
    SP,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryOperand {
    pub base: Option<AArch64Register>,
    pub offset: i32,
    pub pre_index: bool,
    pub post_index: bool,
}

#[derive(Debug)]
pub struct AArch64CodeGenerator {
    instructions: Vec<AArch64Instruction>,
    current_function: Option<String>,
    label_count: u32,
    pool_labels: HashMap<String, u64>,
    code_position: u64,
}

impl AArch64CodeGenerator {
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
        self.instructions.push(AArch64Instruction::LABEL(name.to_string()));
    }

    pub fn end_function(&mut self) {
        self.current_function = None;
    }

    pub fn generate_prologue(&mut self, frame_size: u32) {
        self.stp(AArch64Register::X29, AArch64Register::X30, AArch64Operand::Memory(MemoryOperand {
            base: Some(AArch64Register::SP),
            offset: -16,
            pre_index: true,
            post_index: false,
        }));
        self.mov_reg_reg(AArch64Register::X29, AArch64Register::SP);
        if frame_size > 16 {
            self.sub_imm_reg(frame_size as i64, AArch64Register::SP);
        }
    }

    pub fn generate_epilogue(&mut self) {
        self.add_imm_reg(16, AArch64Register::SP);
        self.ldp(AArch64Register::X29, AArch64Register::X30, AArch64Operand::Memory(MemoryOperand {
            base: Some(AArch64Register::SP),
            offset: 16,
            pre_index: false,
            post_index: true,
        }));
        self.ret(Some(AArch64Register::X30));
    }

    pub fn mov_reg_reg(&mut self, dst: AArch64Register, src: AArch64Register) {
        self.instructions.push(AArch64Instruction::MOV {
            dst: AArch64Operand::Register(dst),
            src: AArch64Operand::Register(src),
        });
    }

    pub fn mov_imm_reg(&mut self, imm: i64, dst: AArch64Register) {
        self.instructions.push(AArch64Instruction::MOV {
            dst: AArch64Operand::Register(dst),
            src: AArch64Operand::Immediate(imm),
        });
    }

    pub fn add_reg_reg_reg(&mut self, dst: AArch64Register, src1: AArch64Register, src2: AArch64Register) {
        self.instructions.push(AArch64Instruction::ADD {
            dst: AArch64Operand::Register(dst),
            src1: AArch64Operand::Register(src1),
            src2: AArch64Operand::Register(src2),
        });
    }

    pub fn add_imm_reg(&mut self, imm: i64, dst: AArch64Register) {
        self.instructions.push(AArch64Instruction::ADD {
            dst: AArch64Operand::Register(dst),
            src1: AArch64Operand::Register(dst),
            src2: AArch64Operand::Immediate(imm),
        });
    }

    pub fn sub_imm_reg(&mut self, imm: i64, dst: AArch64Register) {
        self.instructions.push(AArch64Instruction::SUB {
            dst: AArch64Operand::Register(dst),
            src1: AArch64Operand::Register(dst),
            src2: AArch64Operand::Immediate(imm),
        });
    }

    pub fn sub_reg_reg_reg(&mut self, dst: AArch64Register, src1: AArch64Register, src2: AArch64Register) {
        self.instructions.push(AArch64Instruction::SUB {
            dst: AArch64Operand::Register(dst),
            src1: AArch64Operand::Register(src1),
            src2: AArch64Operand::Register(src2),
        });
    }

    pub fn mul_reg_reg_reg(&mut self, dst: AArch64Register, src1: AArch64Register, src2: AArch64Register) {
        self.instructions.push(AArch64Instruction::MUL {
            dst: AArch64Operand::Register(dst),
            src1: AArch64Operand::Register(src1),
            src2: AArch64Operand::Register(src2),
        });
    }

    pub fn sdiv_reg_reg_reg(&mut self, dst: AArch64Register, src1: AArch64Register, src2: AArch64Register) {
        self.instructions.push(AArch64Instruction::SDIV {
            dst: AArch64Operand::Register(dst),
            src1: AArch64Operand::Register(src1),
            src2: AArch64Operand::Register(src2),
        });
    }

    pub fn and_reg_reg_reg(&mut self, dst: AArch64Register, src1: AArch64Register, src2: AArch64Register) {
        self.instructions.push(AArch64Instruction::AND {
            dst: AArch64Operand::Register(dst),
            src1: AArch64Operand::Register(src1),
            src2: AArch64Operand::Register(src2),
        });
    }

    pub fn orr_reg_reg_reg(&mut self, dst: AArch64Register, src1: AArch64Register, src2: AArch64Register) {
        self.instructions.push(AArch64Instruction::ORR {
            dst: AArch64Operand::Register(dst),
            src1: AArch64Operand::Register(src1),
            src2: AArch64Operand::Register(src2),
        });
    }

    pub fn eor_reg_reg_reg(&mut self, dst: AArch64Register, src1: AArch64Register, src2: AArch64Register) {
        self.instructions.push(AArch64Instruction::EOR {
            dst: AArch64Operand::Register(dst),
            src1: AArch64Operand::Register(src1),
            src2: AArch64Operand::Register(src2),
        });
    }

    pub fn stp(&mut self, rt1: AArch64Register, rt2: AArch64Register, dst: AArch64Operand) {
        self.instructions.push(AArch64Instruction::STR {
            src: AArch64Operand::Register(rt1),
            dst,
        });
        self.instructions.push(AArch64Instruction::STR {
            src: AArch64Operand::Register(rt2),
            dst: match dst {
                AArch64Operand::Memory(mut mem) => {
                    mem.offset += 8;
                    AArch64Operand::Memory(mem)
                }
                _ => dst,
            },
        });
    }

    pub fn ldp(&mut self, rt1: AArch64Register, rt2: AArch64Register, src: AArch64Operand) {
        self.instructions.push(AArch64Instruction::LDR {
            dst: AArch64Operand::Register(rt1),
            src,
        });
        self.instructions.push(AArch64Instruction::LDR {
            dst: AArch64Operand::Register(rt2),
            src: match src {
                AArch64Operand::Memory(mut mem) => {
                    mem.offset += 8;
                    AArch64Operand::Memory(mem)
                }
                _ => src,
            },
        });
    }

    pub fn cmp_reg_reg(&mut self, src1: AArch64Register, src2: AArch64Register) {
        self.instructions.push(AArch64Instruction::CMP {
            src1: AArch64Operand::Register(src1),
            src2: AArch64Operand::Register(src2),
        });
    }

    pub fn cmp_reg_imm(&mut self, src: AArch64Register, imm: i64) {
        self.instructions.push(AArch64Instruction::CMP {
            src1: AArch64Operand::Register(src),
            src2: AArch64Operand::Immediate(imm),
        });
    }

    pub fn b(&mut self, target: &str) {
        self.instructions.push(AArch64Instruction::B {
            target: target.to_string(),
        });
    }

    pub fn bl(&mut self, target: &str) {
        self.instructions.push(AArch64Instruction::BL {
            target: target.to_string(),
        });
    }

    pub fn blr(&mut self, target: AArch64Register) {
        self.instructions.push(AArch64Instruction::BLR { target });
    }

    pub fn ret(&mut self, target: Option<AArch64Register>) {
        self.instructions.push(AArch64Instruction::RET { target });
    }

    pub fn cbz(&mut self, src: AArch64Register, target: &str) {
        self.instructions.push(AArch64Instruction::CBZ {
            src: AArch64Operand::Register(src),
            target: target.to_string(),
        });
    }

    pub fn cbnz(&mut self, src: AArch64Register, target: &str) {
        self.instructions.push(AArch64Instruction::CBNZ {
            src: AArch64Operand::Register(src),
            target: target.to_string(),
        });
    }

    pub fn cset(&mut self, dst: AArch64Register, cond: &str) {
        self.instructions.push(AArch64Instruction::CSET {
            dst: AArch64Operand::Register(dst),
            cond: cond.to_string(),
        });
    }

    pub fn emit_label(&mut self, label: &str) {
        self.instructions.push(AArch64Instruction::LABEL(label.to_string()));
    }

    pub fn new_label(&mut self, prefix: &str) -> String {
        self.label_count += 1;
        format!("{}_{}", prefix, self.label_count)
    }

    pub fn nop(&mut self) {
        self.instructions.push(AArch64Instruction::NOP);
    }

    pub fn memory_operand(&self, base: AArch64Register, offset: i32) -> AArch64Operand {
        AArch64Operand::Memory(MemoryOperand {
            base: Some(base),
            offset,
            pre_index: false,
            post_index: false,
        })
    }

    pub fn emit(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for instr in &self.instructions {
            self.encode_instruction(instr, &mut bytes);
        }
        bytes
    }

    fn encode_instruction(&self, instr: &AArch64Instruction, bytes: &mut Vec<u8>) {
        match instr {
            AArch64Instruction::MOV { dst, src } => {
                self.encode_mov(dst, src, bytes);
            }
            AArch64Instruction::ADD { dst, src1, src2 } => {
                self.encode_add(dst, src1, src2, bytes);
            }
            AArch64Instruction::RET { .. } => {
                bytes.push(0xC0);
                bytes.push(0x03);
                bytes.push(0x5F);
                bytes.push(0xD6);
            }
            AArch64Instruction::NOP => {
                bytes.push(0x1F);
                bytes.push(0x20);
                bytes.push(0x03);
                bytes.push(0xD5);
            }
            AArch64Instruction::LABEL(name) => {
                self.pool_labels.insert(name.clone(), self.code_position);
            }
            _ => {}
        }
        self.code_position += bytes.len() as u64;
    }

    fn encode_mov(&self, _dst: &AArch64Operand, _src: &AArch64Operand, _bytes: &mut Vec<u8>) {
    }

    fn encode_add(&self, _dst: &AArch64Operand, _src1: &AArch64Operand, _src2: &AArch64Operand, _bytes: &mut Vec<u8>) {
    }
}

pub fn generate_aarch64_code(
    ir: &crate::IRModule,
    target: crate::Target,
) -> Vec<u8> {
    let mut codegen = AArch64CodeGenerator::new();
    
    for func in &ir.functions {
        codegen.start_function(&func.name);
        codegen.generate_prologue(0);
        codegen.generate_epilogue();
        codegen.end_function();
    }
    
    codegen.emit()
}

pub const AARCH64_CALLER_SAVED: &[AArch64Register] = &[
    AArch64Register::X0, AArch64Register::X1, AArch64Register::X2, AArch64Register::X3,
    AArch64Register::X4, AArch64Register::X5, AArch64Register::X6, AArch64Register::X7,
    AArch64Register::X8, AArch64Register::X9, AArch64Register::X10, AArch64Register::X11,
];

pub const AARCH64_CALLEE_SAVED: &[AArch64Register] = &[
    AArch64Register::X19, AArch64Register::X20, AArch64Register::X21, AArch64Register::X22,
    AArch64Register::X23, AArch64Register::X24, AArch64Register::X25, AArch64Register::X26,
    AArch64Register::X27, AArch64Register::X28, AArch64Register::X29, AArch64Register::X30,
];
