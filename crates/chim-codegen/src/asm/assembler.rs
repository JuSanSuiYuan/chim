use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssemblySyntax {
    NASM,
    MASM,
    GAS,
    Intel,
    ATandT,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum X86_64Register {
    AL, AH, AX, EAX, RAX,
    BL, BH, BX, EBX, RBX,
    CL, CH, CX, ECX, RCX,
    DL, DH, DX, EDX, RDX,
    SPL, SP, ESP, RSP,
    BPL, BP, EBP, RBP,
    SIL, SI, ESI, RSI,
    DIL, DI, EDI, RDI,
    R8B, R8W, R8D, R8,
    R9B, R9W, R9D, R9,
    R10B, R10W, R10D, R10,
    R11B, R11W, R11D, R11,
    R12B, R12W, R12D, R12,
    R13B, R13W, R13D, R13,
    R14B, R14W, R14D, R14,
    R15B, R15W, R15D, R15,
    RIP,
    EFLAGS, RFLAGS,
    CS, DS, ES, FS, GS, SS,
    CR0, CR2, CR3, CR4,
    DR0, DR1, DR2, DR3, DR6, DR7,
    ST0, ST1, ST2, ST3, ST4, ST5, ST6, ST7,
    MM0, MM1, MM2, MM3, MM4, MM5, MM6, MM7,
    XMM0, XMM1, XMM2, XMM3, XMM4, XMM5, XMM6, XMM7,
    XMM8, XMM9, XMM10, XMM11, XMM12, XMM13, XMM14, XMM15,
    XMM16, XMM17, XMM18, XMM19, XMM20, XMM21, XMM22, XMM23,
    XMM24, XMM25, XMM26, XMM27, XMM28, XMM29, XMM30, XMM31,
    YMM0, YMM1, YMM2, YMM3, YMM4, YMM5, YMM6, YMM7,
    YMM8, YMM9, YMM10, YMM11, YMM12, YMM13, YMM14, YMM15,
    YMM16, YMM17, YMM18, YMM19, YMM20, YMM21, YMM22, YMM23,
    YMM24, YMM25, YMM26, YMM27, YMM28, YMM29, YMM30, YMM31,
    ZMM0, ZMM1, ZMM2, ZMM3, ZMM4, ZMM5, ZMM6, ZMM7,
    ZMM8, ZMM9, ZMM10, ZMM11, ZMM12, ZMM13, ZMM14, ZMM15,
    ZMM16, ZMM17, ZMM18, ZMM19, ZMM20, ZMM21, ZMM22, ZMM23,
    ZMM24, ZMM25, ZMM26, ZMM27, ZMM28, ZMM29, ZMM30, ZMM31,
    K0, K1, K2, K3, K4, K5, K6, K7,
    BND0, BND1, BND2, BND3,
    TMM0, TMM1, TMM2, TMM3, TMM4, TMM5, TMM6, TMM7,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AArch64Register {
    W0, W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12, W13, W14, W15,
    W16, W17, W18, W19, W20, W21, W22, W23, W24, W25, W26, W27, W28, W29, W30, WZR,
    X0, X1, X2, X3, X4, X5, X6, X7, X8, X9, X10, X11, X12, X13, X14, X15,
    X16, X17, X18, X19, X20, X21, X22, X23, X24, X25, X26, X27, X28, X29, X30, XZR,
    SP, WSP, SP,
    V0, V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11, V12, V13, V14, V15,
    V16, V17, V18, V19, V20, V21, V22, V23, V24, V25, V26, V27, V28, V29, V30, V31,
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15,
    B16, B17, B18, B19, B20, B21, B22, B23, B24, B25, B26, B27, B28, B29, B30, B31,
    H0, H1, H2, H3, H4, H5, H6, H7, H8, H9, H10, H11, H12, H13, H14, H15,
    H16, H17, H18, H19, H20, H21, H22, H23, H24, H25, H26, H27, H28, H29, H30, H31,
    S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15,
    S16, S17, S18, S19, S20, S21, S22, S23, S24, S25, S26, S27, S28, S29, S30, S31,
    D0, D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D11, D12, D13, D14, D15,
    D16, D17, D18, D19, D20, D21, D22, D23, D24, D25, D26, D27, D28, D29, D30, D31,
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15,
    Q16, Q17, Q18, Q19, Q20, Q21, Q22, Q23, Q24, Q25, Q26, Q27, Q28, Q29, Q30, Q31,
    NZCV, FPSR, FPCR,
    PC,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OperandSize {
    Byte,
    Word,
    DWord,
    QWord,
    XWord,
    YWord,
    ZWord,
}

#[derive(Debug, Clone)]
pub enum AssemblyOperand {
    Register(RegisterRef),
    Memory(MemoryOperand),
    Immediate(ImmediateValue),
    Label(String),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct RegisterRef {
    pub reg: String,
    pub size: Option<OperandSize>,
    pub high_byte: bool,
    pub low_byte: bool,
}

#[derive(Debug, Clone)]
pub struct MemoryOperand {
    pub base: Option<RegisterRef>,
    pub index: Option<RegisterRef>,
    pub scale: u8,
    pub displacement: Option<ImmediateValue>,
    pub segment: Option<String>,
    pub width: Option<OperandSize>,
}

#[derive(Debug, Clone)]
pub struct ImmediateValue {
    pub value: i128,
    pub size: Option<OperandSize>,
    pub is_signed: bool,
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub left: Box<AssemblyOperand>,
    pub operator: ExpressionOperator,
    pub right: Box<AssemblyOperand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpressionOperator {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Xor,
    Shl,
    Shr,
}

#[derive(Debug, Clone)]
pub struct X86_64Instruction {
    pub mnemonic: String,
    pub operands: Vec<AssemblyOperand>,
    pub prefixes: Vec<String>,
    pub size_hint: Option<OperandSize>,
    pub is_rex: bool,
    pub is_vex: bool,
    pub is_evex: bool,
    pub is_xop: bool,
    pub condition: Option<ConditionCode>,
    pub hint: Option<BranchHint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConditionCode {
    O, NO, B, NA, AE, NB, E, Z, NE, NZ, BE, G, LE, L, GE,
    parity, notparity, below, notbelow, above, notabove,
    equal, notequal, less, greater, lessequal, greaterequal,
    sign, notsign, overflow, notoverflow,
    Carry, NotCarry, Zero, NotZero, Less, LessEqual, Greater, GreaterEqual,
    Above, AboveEqual, Below, BelowEqual,
    Overflow, NotOverflow, Sign, NotSign,
    Parity, NotParity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BranchHint {
    Likely,
    Unlikely,
}

#[derive(Debug, Clone)]
pub struct AArch64Instruction {
    pub mnemonic: String,
    pub condition: Option<ConditionCode>,
    pub operands: Vec<AssemblyOperand>,
    pub modifiers: Vec<AArch64Modifier>,
    pub shift: Option<(ShiftType, u8)>,
    pub extend: Option<(ExtendType, Option<u8>)>,
    pub is_wide: bool,
    pub vector: Option<VectorOperand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AArch64Modifier {
    SF,
    W,
    X,
    S,
    D,
    Q,
    V,
    BarrelShifter,
    Prefetch,
    Hint(String),
    System(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShiftType {
    LSL,
    LSR,
    ASR,
    ROR,
    RRX,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtendType {
    UXTB,
    UXTH,
    UXTW,
    UXTX,
    SXTB,
    SXTH,
    SXTW,
    SXTX,
}

#[derive(Debug, Clone)]
pub struct VectorOperand {
    pub lanes: u8,
    pub element_size: u8,
    pub arrangement: VectorArrangement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VectorArrangement {
    _8B,
    _16B,
    _4H,
    _8H,
    _2S,
    _4S,
    _1D,
    _2D,
    _8B8,
    _16B8,
    _4H4,
    _8H4,
    _2S2,
    _4S2,
    _1D2,
    _2D2,
}

#[derive(Debug, Clone)]
pub struct AssemblyBlock {
    pub name: String,
    pub is_global: bool,
    pub is_extern: bool,
    pub instructions: Vec<AssemblyInstruction>,
    pub labels: Vec<AssemblyLabel>,
    pub align: Option<u32>,
    pub section: Option<SectionType>,
}

#[derive(Debug, Clone)]
pub enum AssemblyInstruction {
    X86_64(X86_64Instruction),
    AArch64(AArch64Instruction),
    Directive(AssemblyDirective),
}

#[derive(Debug, Clone)]
pub struct AssemblyLabel {
    pub name: String,
    pub is_local: bool,
    pub is_global: bool,
    pub position: usize,
}

#[derive(Debug, Clone)]
pub enum AssemblyDirective {
    Section(SectionType),
    Align(u32),
    Byte(Vec<ImmediateValue>),
    Word(Vec<ImmediateValue>),
    DWord(Vec<ImmediateValue>),
    QWord(Vec<ImmediateValue>),
    Asciz(String),
    Ascii(String),
    Zero(u32),
    Skip(u32),
    Reserve(u32),
    Float(Vec<f32>),
    Double(Vec<f64>),
    Include(String),
    Incbin(String, Option<u32>, Option<u32>),
    Equ(String, ImmediateValue),
    Set(String, ImmediateValue),
    Macro(String, Vec<String>, Vec<AssemblyInstruction>),
    EndMacro,
    Rept(u32),
    EndRept,
    If(ImmediateValue),
    ElseIf(ImmediateValue),
    Else,
    EndIf,
    Comment(String),
    LineInfo(String),
    Type(String, String),
    Size(String, ImmediateValue),
    Visibility(String),
    Weak(String),
    WeakAlias(String, String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectionType {
    Text,
    Data,
    Bss,
    Rodata,
    Relro,
    Init,
    Fini,
    Tls,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct InlineAssembly {
    pub code: String,
    pub syntax: AssemblySyntax,
    pub operands: Vec<InlineOperand>,
    pub clobbers: Vec<String>,
    pub side_effects: bool,
    pub is_intel: bool,
    pub align_stack: bool,
    pub dialect: AssemblyDialect,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssemblyDialect {
    ATT,
    Intel,
    NASM,
    MASM,
    GAS,
}

#[derive(Debug, Clone)]
pub struct InlineOperand {
    pub name: String,
    pub constraint: String,
    pub operand: AssemblyOperand,
    pub read_only: bool,
    pub write_only: bool,
    pub late_clobber: bool,
}

#[derive(Debug, Clone)]
pub struct AssemblyFunction {
    pub name: String,
    pub is_global: bool,
    pub is_extern: bool,
    pub is_weak: bool,
    pub calling_convention: Option<CallingConvention>,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<FunctionReturn>,
    pub body: Vec<AssemblyBlock>,
    pub align: u32,
    pub section: Option<SectionType>,
}

#[derive(Debug, Clone)]
pub struct FunctionParameter {
    pub name: String,
    pub reg: Option<String>,
    pub stack_offset: Option<u32>,
    pub size: u32,
    pub ty: ParameterType,
}

#[derive(Debug, Clone)]
pub struct FunctionReturn {
    pub reg: Option<String>,
    pub stack_offset: Option<u32>,
    pub size: u32,
    pub ty: ReturnType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParameterType {
    Integer,
    Float,
    Double,
    Vector,
    Pointer,
    Struct(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReturnType {
    Integer,
    Float,
    Double,
    Vector,
    Pointer,
    Struct(u32),
    Void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallingConvention {
    Cdecl,
    Stdcall,
    Thiscall,
    Fastcall,
    Vectorcall,
    SysV,
    Microsoft,
    AArch64AAPCS,
    AArch64AAPCS64,
    PNA,
    AMDM64,
    Target,
}

#[derive(Debug, Default)]
pub struct AssemblyParser {
    syntax: AssemblySyntax,
    current_line: usize,
    labels: HashMap<String, usize>,
    macros: HashMap<String, Vec<AssemblyInstruction>>,
    structs: HashMap<String, StructDef>,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
    pub size: u32,
    pub alignment: u32,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub offset: u32,
    pub size: u32,
    pub ty: String,
}

impl AssemblyParser {
    pub fn new(syntax: AssemblySyntax) -> Self {
        Self {
            syntax,
            current_line: 0,
            labels: HashMap::new(),
            macros: HashMap::new(),
            structs: HashMap::new(),
        }
    }

    pub fn parse(&mut self, code: &str) -> Result<Vec<AssemblyFunction>, AssemblyError> {
        let mut functions = Vec::new();
        self.current_line = 0;
        self.labels.clear();
        
        let lines: Vec<&str> = code.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                i += 1;
                continue;
            }
            
            if line.starts_with("global") || line.starts_with(".global") {
                i += 1;
                continue;
            }
            
            if line.starts_with("extern") || line.starts_with(".extern") {
                i += 1;
                continue;
            }
            
            if line.starts_with("section") || line.starts_with(".section") {
                i += 1;
                continue;
            }
            
            if line.ends_with(':') {
                let label_name = &line[..line.len()-1];
                self.labels.insert(label_name.to_string(), self.current_line);
                i += 1;
                continue;
            }
            
            if line.starts_with("align") || line.starts_with(".align") {
                i += 1;
                continue;
            }
            
            if line.starts_with("proc") || line.ends_with("proc") {
                let func = self.parse_function(&lines[i..])?;
                functions.push(func);
                i += 1;
                continue;
            }
            
            i += 1;
        }
        
        Ok(functions)
    }

    fn parse_function<'a>(&self, lines: &[&str]) -> Result<AssemblyFunction, AssemblyError> {
        let mut name = String::new();
        let mut is_global = false;
        let mut is_extern = false;
        let mut align = 16;
        
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            
            if line.is_empty() {
                i += 1;
                continue;
            }
            
            if line.starts_with("global") || line.starts_with(".global") {
                is_global = true;
            } else if line.starts_with("extern") || line.starts_with(".extern") {
                is_extern = true;
            } else if line.starts_with("align") || line.starts_with(".align") {
                if let Ok(align_val) = line.split_whitespace().nth(1).unwrap_or("16").parse() {
                    align = align_val;
                }
            } else if !name.is_empty() && (line.starts_with("proc") || line.starts_with(".proc")) {
                i += 1;
                break;
            } else if !line.ends_with(':') && !line.starts_with(';') && !line.starts_with('#') {
                name = line.split_whitespace().next().unwrap_or("").to_string();
            }
            
            i += 1;
        }
        
        Ok(AssemblyFunction {
            name,
            is_global,
            is_extern,
            is_weak: false,
            calling_convention: None,
            parameters: Vec::new(),
            return_type: None,
            body: Vec::new(),
            align,
            section: None,
        })
    }

    pub fn parse_instruction(&self, line: &str) -> Result<AssemblyInstruction, AssemblyError> {
        let line = line.trim();
        
        if line.starts_with('.') {
            self.parse_directive(line)
        } else {
            self.parse_instruction_mnemonic(line)
        }
    }

    fn parse_directive(&self, line: &str) -> Result<AssemblyInstruction, AssemblyError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        match parts[0] {
            ".byte" | "db" | "BYTE" => {
                let values: Vec<ImmediateValue> = parts[1..]
                    .iter()
                    .filter_map(|p| self.parse_immediate(p))
                    .collect();
                Ok(AssemblyDirective::Byte(values).into())
            }
            ".word" | "dw" | "WORD" => {
                let values: Vec<ImmediateValue> = parts[1..]
                    .iter()
                    .filter_map(|p| self.parse_immediate(p))
                    .collect();
                Ok(AssemblyDirective::Word(values).into())
            }
            ".long" | "dd" | "DWORD" | ".int" => {
                let values: Vec<ImmediateValue> = parts[1..]
                    .iter()
                    .filter_map(|p| self.parse_immediate(p))
                    .collect();
                Ok(AssemblyDirective::DWord(values).into())
            }
            ".quad" | "dq" | "QWORD" => {
                let values: Vec<ImmediateValue> = parts[1..]
                    .iter()
                    .filter_map(|p| self.parse_immediate(p))
                    .collect();
                Ok(AssemblyDirective::QWord(values).into())
            }
            ".ascii" => {
                let content = line[7..].trim();
                Ok(AssemblyDirective::Ascii(content.to_string()).into())
            }
            ".asciz" | ".string" => {
                let content = line[7..].trim();
                Ok(AssemblyDirective::Asciz(content.to_string()).into())
            }
            ".zero" | "resb" | "resw" | "resd" | "resq" => {
                let size = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                Ok(AssemblyDirective::Zero(size).into())
            }
            ".align" | "align" => {
                let align_val = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(4);
                Ok(AssemblyDirective::Align(align_val).into())
            }
            ".text" | "section .text" => {
                Ok(AssemblyDirective::Section(SectionType::Text).into())
            }
            ".data" | "section .data" => {
                Ok(AssemblyDirective::Section(SectionType::Data).into())
            }
            ".bss" | "section .bss" => {
                Ok(AssemblyDirective::Section(SectionType::Bss).into())
            }
            ".rodata" | "section .rodata" => {
                Ok(AssemblyDirective::Section(SectionType::Rodata).into())
            }
            ".equ" | "equ" => {
                if parts.len() >= 3 {
                    let name = parts[1].to_string();
                    if let Some(value) = self.parse_immediate(parts[2]) {
                        return Ok(AssemblyDirective::Equ(name, value).into());
                    }
                }
                Err(AssemblyError::InvalidDirective(line.to_string()))
            }
            _ => Err(AssemblyError::UnknownDirective(parts[0].to_string())),
        }
    }

    fn parse_instruction_mnemonic(&self, line: &str) -> Result<AssemblyInstruction, AssemblyError> {
        let parts: Vec<&str> = line.split(|c| c == ',' || c == ' ' || c == '\t')
            .filter(|s| !s.is_empty())
            .collect();
        
        if parts.is_empty() {
            return Err(AssemblyError::EmptyInstruction);
        }
        
        let mnemonic = parts[0].to_lowercase();
        let operands: Vec<AssemblyOperand> = parts[1..]
            .iter()
            .filter_map(|s| self.parse_operand(s))
            .collect();
        
        if self.is_x86_instruction(&mnemonic) {
            Ok(X86_64Instruction {
                mnemonic,
                operands,
                prefixes: Vec::new(),
                size_hint: None,
                is_rex: false,
                is_vex: false,
                is_evex: false,
                is_xop: false,
                condition: None,
                hint: None,
            }.into())
        } else if self.is_aarch64_instruction(&mnemonic) {
            Ok(AArch64Instruction {
                mnemonic,
                condition: None,
                operands,
                modifiers: Vec::new(),
                shift: None,
                extend: None,
                is_wide: false,
                vector: None,
            }.into())
        } else {
            Err(AssemblyError::UnknownInstruction(mnemonic))
        }
    }

    fn parse_operand(&self, s: &str) -> Option<AssemblyOperand> {
        let s = s.trim();
        
        if s.starts_with('[') && s.ends_with(']') {
            let inner = &s[1..s.len()-1];
            return Some(AssemblyOperand::Memory(self.parse_memory_operand(inner)));
        }
        
        if s.starts_with('$') {
            return Some(AssemblyOperand::Immediate(ImmediateValue {
                value: s[1..].parse().ok()?,
                size: None,
                is_signed: true,
            }));
        }
        
        if s.starts_with('%') {
            return Some(AssemblyOperand::Register(RegisterRef {
                reg: s[1..].to_string(),
                size: None,
                high_byte: false,
                low_byte: false,
            }));
        }
        
        if s.starts_with('%') {
            return Some(AssemblyOperand::Register(RegisterRef {
                reg: s[1..].to_string(),
                size: None,
                high_byte: false,
                low_byte: false,
            }));
        }
        
        if s.parse::<i128>().is_ok() {
            return Some(AssemblyOperand::Immediate(ImmediateValue {
                value: s.parse().ok()?,
                size: None,
                is_signed: true,
            }));
        }
        
        if s.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
            return Some(AssemblyOperand::Label(s.to_string()));
        }
        
        None
    }

    fn parse_memory_operand(&self, s: &str) -> MemoryOperand {
        let mut base = None;
        let mut index = None;
        let mut scale = 1;
        let mut displacement = None;
        
        let parts: Vec<&str> = s.split(|c| c == '+' || c == '-' || c == '*')
            .filter(|s| !s.is_empty())
            .collect();
        
        for part in parts {
            let part = part.trim();
            if part.ends_with("*2") {
                scale = 2;
                let reg_name = &part[..part.len()-2];
                index = Some(RegisterRef {
                    reg: reg_name.to_string(),
                    size: None,
                    high_byte: false,
                    low_byte: false,
                });
            } else if part.ends_with("*4") {
                scale = 4;
                let reg_name = &part[..part.len()-2];
                index = Some(RegisterRef {
                    reg: reg_name.to_string(),
                    size: None,
                    high_byte: false,
                    low_byte: false,
                });
            } else if part.ends_with("*8") {
                scale = 8;
                let reg_name = &part[..part.len()-2];
                index = Some(RegisterRef {
                    reg: reg_name.to_string(),
                    size: None,
                    high_byte: false,
                    low_byte: false,
                });
            } else if part.parse::<i128>().is_ok() {
                displacement = Some(ImmediateValue {
                    value: part.parse().ok()?,
                    size: None,
                    is_signed: true,
                });
            } else if part == "rip" {
                base = Some(RegisterRef {
                    reg: "rip".to_string(),
                    size: None,
                    high_byte: false,
                    low_byte: false,
                });
            } else {
                base = Some(RegisterRef {
                    reg: part.to_string(),
                    size: None,
                    high_byte: false,
                    low_byte: false,
                });
            }
        }
        
        MemoryOperand {
            base,
            index,
            scale,
            displacement,
            segment: None,
            width: None,
        }
    }

    fn parse_immediate(&self, s: &str) -> Option<ImmediateValue> {
        let s = s.trim();
        
        if s.starts_with('$') {
            return Some(ImmediateValue {
                value: s[1..].parse().ok()?,
                size: None,
                is_signed: true,
            });
        }
        
        if s.starts_with("0x") || s.starts_with("0X") {
            return Some(ImmediateValue {
                value: i128::from_str_radix(&s[2..], 16).ok()?,
                size: None,
                is_signed: true,
            });
        }
        
        s.parse::<i128>().ok().map(|v| ImmediateValue {
            value: v,
            size: None,
            is_signed: true,
        })
    }

    fn is_x86_instruction(&self, mnemonic: &str) -> bool {
        matches!(mnemonic,
            "mov" | "movzx" | "movsx" | "movsxd" | "movdqa" | "movdqu" | "movaps" | "movups" |
            "add" | "sub" | "inc" | "dec" | "neg" | "adc" | "sbb" |
            "and" | "or" | "xor" | "not" |
            "shl" | "shr" | "sal" | "sar" | "shld" | "shrd" |
            "mul" | "imul" | "div" | "idiv" |
            "cmp" | "test" |
            "push" | "pop" |
            "lea" | "xchg" | "cmpxchg" |
            "jo" | "jno" | "jb" | "jnae" | "jc" | "jnc" | "jz" | "je" | "jnz" | "jne" |
            "jbe" | "jna" | "ja" | "jnbe" | "jl" | "jnge" | "jge" | "jnl" | "jle" | "jng" |
            "jg" | "jnle" | "jp" | "jnp" | "jpe" | "jpo" | "js" | "jns" |
            "call" | "ret" | "jmp" | "loop" | "loope" | "loopne" |
            "movsb" | "movsw" | "movsd" | "movsq" | "cmpsb" | "cmpsw" | "cmpsd" | "cmpsq" |
            "scasb" | "scasw" | "scasd" | "scasq" | "lodsb" | "lodsw" | "lodsd" | "lodsq" |
            "stosb" | "stosw" | "stosd" | "stosq" | "rep" | "repe" | "repne" |
            "cwd" | "cdq" | "cqo" | "cbw" | "cwde" | "cdqe" |
            "fld" | "fst" | "fstp" | "fadd" | "fsub" | "fmul" | "fdiv" |
            "addss" | "subss" | "mulss" | "divss" | "sqrtss" | "maxss" | "minss" |
            "addsd" | "subsd" | "mulsd" | "divsd" | "sqrtsd" | "maxsd" | "minsd" |
            "movss" | "movsd" | "unpcklps" | "unpcklpd" | "unpckhps" | "unpckhpd" |
            "cmpltss" | "cmpltsd" | "cmpless" | "cmplesd" | "cmpneqss" | "cmpneqsd" |
            "cvttss2si" | "cvtsi2ss" | "cvttsd2si" | "cvtsi2sd" |
            "ucomiss" | "ucomisd" | "sqrtss" | "rsqrttss" | "rcppss" |
            "vbroadcastss" | "vbroadcastsd" | "vbroadcasti128" |
            "vextracti128" | "vinserti128" | "vpextrb" | "vpextrw" | "vpextrd" | "vpextrq" |
            "vpmovsxbw" | "vpmovsxbd" | "vpmovsxbq" | "vpmovsxwd" | "vpmovsxwq" | "vpmovsxdq" |
            "vpmovzxbw" | "vpmovzxbd" | "vpmovzxbq" | "vpmovzxwd" | "vpmovzxwq" | "vpmovzxdq"
        )
    }

    fn is_aarch64_instruction(&self, mnemonic: &str) -> bool {
        matches!(mnemonic,
            "add" | "adds" | "sub" | "subs" | "adr" | "adrp" |
            "mov" | "mvn" | "movk" | "movn" | "movz" |
            "and" | "orr" | "eor" | "bic" |
            "lsl" | "lsr" | "asr" | "ror" |
            "cmp" | "cmn" | "tst" |
            "ldrb" | "ldrh" | "ldr" | "ldrsb" | "ldrsh" | "ldrsw" | "ldp" | "ldp" |
            "strb" | "strh" | "str" | "stp" |
            "cbz" | "cbnz" | "tbz" | "tbnz" |
            "b" | "bl" | "blr" | "br" | "ret" |
            "beq" | "bne" | "bhs" | "bcs" | "blo" | "bcc" | "bmi" | "bpl" |
            "bvs" | "bvc" | "bhi" | "bls" | "bge" | "blt" | "bgt" | "ble" |
            "nop" | "yield" | "wfe" | "wfi" | "sev" | "sevl" |
            "dsb" | "dmb" | "isb" |
            "mrs" | "msr" | "mrs" |
            "fcvt" | "fcvtzs" | "fcvtzu" | "scvtf" | "ucvtf" |
            "fadd" | "fsub" | "fmul" | "fdiv" | "fmadd" | "fmsub" |
            "fabs" | "fneg" | "fsqrt" | "frsqrte" | "frecipe" |
            "fcmp" | "fccmp" | "fcsel" |
            "addp" | "addv" | "smax" | "smin" | "umax" | "umin" |
            "dup" | "ins" | "tbl" | "tbx" |
            "ld1" | "st1" | "ld1r" | "st1r" |
            "ext" | "zip1" | "zip2" | "uzp1" | "uzp2" | "trn1" | "trn2"
        )
    }
}

#[derive(Debug)]
pub struct AssemblyError {
    pub message: String,
    pub line: usize,
}

impl fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Assembly error at line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for AssemblyError {}

#[derive(Debug, Clone)]
pub enum AssemblyErrorType {
    UnknownDirective(String),
    UnknownInstruction(String),
    InvalidDirective(String),
    EmptyInstruction,
    InvalidOperand(String),
    InvalidImmediate(String),
    UndefinedLabel(String),
    RedefinedLabel(String),
    MacroRecursionLimit,
    InvalidMacroDefinition,
    UnterminatedMacro,
    InvalidSection,
    InvalidAlign,
    InvalidExpression,
    SyntaxError,
}

impl From<AssemblyInstruction> for X86_64Instruction {
    fn from(instr: AssemblyInstruction) -> Self {
        match instr {
            AssemblyInstruction::X86_64(i) => i,
            _ => panic!("Expected x86_64 instruction"),
        }
    }
}

impl From<AssemblyInstruction> for AArch64Instruction {
    fn from(instr: AssemblyInstruction) -> Self {
        match instr {
            AssemblyInstruction::AArch64(i) => i,
            _ => panic!("Expected AArch64 instruction"),
        }
    }
}

impl From<AssemblyDirective> for AssemblyInstruction {
    fn from(directive: AssemblyDirective) -> Self {
        AssemblyInstruction::Directive(directive)
    }
}

impl From<AssemblyErrorType> for AssemblyError {
    fn from(error_type: AssemblyErrorType) -> Self {
        AssemblyError {
            message: format!("{:?}", error_type),
            line: 0,
        }
    }
}

pub struct InlineAssemblyGenerator {
    pub options: InlineAssemblyOptions,
}

#[derive(Debug, Clone, Default)]
pub struct InlineAssemblyOptions {
    pub syntax: AssemblySyntax,
    pub dialect: AssemblyDialect,
    pub volatile: bool,
    pub inline: bool,
    pub align_stack: bool,
    pub preserve_flags: bool,
    pub preserve_all: bool,
}

impl InlineAssemblyGenerator {
    pub fn new() -> Self {
        Self {
            options: InlineAssemblyOptions::default(),
        }
    }

    pub fn generate_inline_asm(&self, asm: &InlineAssembly, chim_expr: &str) -> String {
        match asm.syntax {
            AssemblySyntax::NASM => self.generate_nasm_inline(asm, chim_expr),
            AssemblySyntax::GAS => self.generate_gas_inline(asm, chim_expr),
            AssemblySyntax::MASM => self.generate_masm_inline(asm, chim_expr),
            _ => self.generate_gas_inline(asm, chim_expr),
        }
    }

    fn generate_nasm_inline(&self, asm: &InlineAssembly, chim_expr: &str) -> String {
        let mut output = String::new();
        
        output.push_str("asm {");
        
        for (i, line) in asm.code.lines().enumerate() {
            if i > 0 {
                output.push_str("; ");
            }
            output.push_str(line.trim());
        }
        
        output.push_str("}");
        
        output
    }

    fn generate_gas_inline(&self, asm: &InlineAssembly, chim_expr: &str) -> String {
        let mut output = String::new();
        
        output.push_str("asm(");
        
        if asm.is_intel {
            output.push_str("\".intel_syntax noprefix\\n\"");
        } else {
            output.push_str("\".syntax att\\n\"");
        }
        
        for line in asm.code.lines() {
            output.push_str(&format!("\"{} \\n\\\"\\n\"", line.trim()));
        }
        
        output.push_str("\".att_syntax\\n\"");
        
        if !asm.operands.is_empty() {
            output.push_str(", ");
            let operands: Vec<String> = asm.operands.iter()
                .map(|o| format!("\"{}\"", o.name))
                .collect();
            output.push_str(&operands.join(", "));
        }
        
        if !asm.clobbers.is_empty() {
            output.push_str(", ");
            let clobbers: Vec<String> = asm.clobbers.iter()
                .map(|c| format!("\"~{{%}}\"", c))
                .collect();
            output.push_str(&clobbers.join(", "));
        }
        
        output.push_str(")");
        
        output
    }

    fn generate_masm_inline(&self, asm: &InlineAssembly, chim_expr: &str) -> String {
        let mut output = String::new();
        
        output.push_str("__asm {");
        
        for line in asm.code.lines() {
            output.push_str(" ");
            output.push_str(line.trim());
        }
        
        output.push_str(" }");
        
        output
    }
}

pub fn assemble_x86_64(
    code: &str,
    syntax: AssemblySyntax,
    _output_format: OutputFormat,
) -> Result<Vec<u8>, AssemblyError> {
    let mut parser = AssemblyParser::new(syntax);
    let _functions = parser.parse(code)?;
    
    let mut output = Vec::new();
    
    for line in code.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        
        if let Ok(instr) = parser.parse_instruction(line) {
            match instr {
                AssemblyInstruction::X86_64(i) => {
                    let bytes = encode_x86_instruction(&i);
                    output.extend_from_slice(&bytes);
                }
                _ => {}
            }
        }
    }
    
    Ok(output)
}

fn encode_x86_instruction(instr: &X86_64Instruction) -> Vec<u8> {
    let mut bytes = Vec::new();
    
    match instr.mnemonic.as_str() {
        "mov" => {
            if let Some(dst) = instr.operands.get(0) {
                if let Some(src) = instr.operands.get(1) {
                    match (dst, src) {
                        (AssemblyOperand::Register(r), AssemblyOperand::Immediate(imm)) => {
                            bytes.push(0x48);
                            bytes.push(0xC7);
                            bytes.push(0xC0 | r.reg.chars().next().map(|c| (c as u8 - b'a') % 16).unwrap_or(0));
                            bytes.extend_from_slice(&(imm.value as i32).to_le_bytes());
                        }
                        (AssemblyOperand::Register(r1), AssemblyOperand::Register(r2)) => {
                            bytes.push(0x48);
                            bytes.push(0x89);
                            bytes.push(0xC0 | (r1.reg.chars().next().map(|c| (c as u8 - b'a') % 8).unwrap_or(0)) |
                                       ((r2.reg.chars().next().map(|c| (c as u8 - b'a') % 8).unwrap_or(0)) << 3));
                        }
                        _ => {}
                    }
                }
            }
        }
        "push" => {
            if let Some(operand) = instr.operands.get(0) {
                if let AssemblyOperand::Register(r) = operand {
                    let reg_num = match r.reg.as_str() {
                        "rax" => 0, "rcx" => 1, "rdx" => 2, "rbx" => 3,
                        "rsp" => 4, "rbp" => 5, "rsi" => 6, "rdi" => 7,
                        "r8" => 0, "r9" => 1, "r10" => 2, "r11" => 3,
                        "r12" => 4, "r13" => 5, "r14" => 6, "r15" => 7,
                        _ => 0,
                    };
                    bytes.push(0x50 | reg_num as u8);
                }
            }
        }
        "pop" => {
            if let Some(operand) = instr.operands.get(0) {
                if let AssemblyOperand::Register(r) = operand {
                    let reg_num = match r.reg.as_str() {
                        "rax" => 0, "rcx" => 1, "rdx" => 2, "rbx" => 3,
                        "rsp" => 4, "rbp" => 5, "rsi" => 6, "rdi" => 7,
                        "r8" => 0, "r9" => 1, "r10" => 2, "r11" => 3,
                        "r12" => 4, "r13" => 5, "r14" => 6, "r15" => 7,
                        _ => 0,
                    };
                    bytes.push(0x58 | reg_num as u8);
                }
            }
        }
        "ret" => {
            bytes.push(0xC3);
        }
        "nop" => {
            bytes.push(0x90);
        }
        "add" => {
            bytes.push(0x48);
            bytes.push(0x01);
            bytes.push(0xC0);
        }
        "sub" => {
            bytes.push(0x48);
            bytes.push(0x29);
            bytes.push(0xC0);
        }
        "xor" => {
            bytes.push(0x48);
            bytes.push(0x31);
            bytes.push(0xC0);
        }
        "call" => {
            bytes.push(0xE8);
            bytes.extend_from_slice(&0i32.to_le_bytes());
        }
        "jmp" => {
            bytes.push(0xE9);
            bytes.extend_from_slice(&0i32.to_le_bytes());
        }
        "inc" => {
            if let Some(operand) = instr.operands.get(0) {
                if let AssemblyOperand::Register(r) = operand {
                    bytes.push(0x48);
                    bytes.push(0xFF);
                    let reg_num = match r.reg.as_str() {
                        "rax" => 0, "rcx" => 1, "rdx" => 2, "rbx" => 3,
                        "rsp" => 4, "rbp" => 5, "rsi" => 6, "rdi" => 7,
                        _ => 0,
                    };
                    bytes.push(0xC0 | reg_num);
                }
            }
        }
        _ => {
            bytes.push(0x90);
        }
    }
    
    bytes
}

pub enum OutputFormat {
    ELF64,
    MachO64,
    PE32Plus,
    Raw,
}

pub fn assemble_aarch64(
    code: &str,
    _syntax: AssemblySyntax,
) -> Result<Vec<u8>, AssemblyError> {
    let mut parser = AssemblyParser::new(AssemblySyntax::GAS);
    let _functions = parser.parse(code)?;
    
    let mut output = Vec::new();
    
    for line in code.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        
        if let Ok(instr) = parser.parse_instruction(line) {
            match instr {
                AssemblyInstruction::AArch64(i) => {
                    let bytes = encode_aarch64_instruction(&i);
                    output.extend_from_slice(&bytes);
                }
                _ => {}
            }
        }
    }
    
    Ok(output)
}

fn encode_aarch64_instruction(instr: &AArch64Instruction) -> Vec<u8> {
    let mut bytes = Vec::new();
    
    match instr.mnemonic.as_str() {
        "ret" => {
            bytes.push(0xC0);
            bytes.push(0x03);
            bytes.push(0x5F);
            bytes.push(0xD6);
        }
        "nop" => {
            bytes.push(0x1F);
            bytes.push(0x20);
            bytes.push(0x03);
            bytes.push(0xD5);
        }
        "mov" => {
            bytes.push(0x20);
            bytes.push(0x00);
            bytes.push(0x00);
            bytes.push(0xAA);
        }
        "add" => {
            bytes.push(0x00);
            bytes.push(0x00);
            bytes.push(0x00);
            bytes.push(0x91);
        }
        "sub" => {
            bytes.push(0x00);
            bytes.push(0x00);
            bytes.push(0x00);
            bytes.push(0xD1);
        }
        "b" => {
            bytes.push(0x00);
            bytes.push(0x00);
            bytes.push(0x00);
            bytes.push(0x14);
        }
        "bl" => {
            bytes.push(0x00);
            bytes.push(0x00);
            bytes.push(0x00);
            bytes.push(0x94);
        }
        "ldr" => {
            bytes.push(0x00);
            bytes.push(0x00);
            bytes.push(0x00);
            bytes.push(0x58);
        }
        "str" => {
            bytes.push(0x00);
            bytes.push(0x00);
            bytes.push(0x00);
            bytes.push(0x39);
        }
        _ => {
            bytes.push(0x1F);
            bytes.push(0x20);
            bytes.push(0x03);
            bytes.push(0xD5);
        }
    }
    
    bytes
}

pub fn map_chim_type_to_asm(chim_type: &crate::Type) -> AssemblyOperand {
    match chim_type {
        crate::Type::CVoid => AssemblyOperand::Immediate(ImmediateValue {
            value: 0,
            size: Some(OperandSize::QWord),
            is_signed: false,
        }),
        crate::Type::CInt | crate::Type::CLong | crate::Type::CLongLong => {
            AssemblyOperand::Immediate(ImmediateValue {
                value: 0,
                size: Some(OperandSize::QWord),
                is_signed: true,
            })
        }
        crate::Type::CFloat | crate::Type::CDouble => {
            AssemblyOperand::Immediate(ImmediateValue {
                value: 0,
                size: Some(OperandSize::QWord),
                is_signed: true,
            })
        }
        _ => AssemblyOperand::Immediate(ImmediateValue {
            value: 0,
            size: Some(OperandSize::QWord),
            is_signed: false,
        }),
    }
}
