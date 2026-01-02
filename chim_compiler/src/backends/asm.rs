use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// x86-64汇编后端 - AT&T语法
/// 
/// 特点：
/// - 生成x86-64汇编代码（AT&T语法，GNU AS兼容）
/// - 支持System V ABI调用约定
/// - 寄存器分配策略
/// - 栈帧管理
pub struct AsmBackend {
    use_intel_syntax: bool,  // false=AT&T, true=Intel
}

impl AsmBackend {
    pub fn new() -> Self {
        Self {
            use_intel_syntax: false,  // 默认AT&T语法
        }
    }
    
    pub fn with_intel_syntax() -> Self {
        Self {
            use_intel_syntax: true,
        }
    }
    
    /// 获取类型大小（字节）
    fn type_size(&self, ty: &Type) -> usize {
        match ty {
            Type::Void => 0,
            Type::Bool => 1,
            Type::Int32 | Type::Float32 => 4,
            Type::Int64 | Type::Float64 => 8,
            Type::Ptr(_) | Type::Ref(_) | Type::MutRef(_) => 8,  // 64位指针
            Type::String => 8,
            Type::Array(inner, count) => self.type_size(inner) * count,
            _ => 8,
        }
    }
    
    /// 生成函数
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        let mut stack_offset = 0;
        let mut var_offsets = std::collections::HashMap::new();
        
        // 函数标签和全局声明
        output.push_str(&format!("    .globl {}\n", func.name));
        output.push_str(&format!("    .type {}, @function\n", func.name));
        output.push_str(&format!("{}:\n", func.name));
        
        // 函数序言（prologue）
        output.push_str("    pushq   %rbp\n");
        output.push_str("    movq    %rsp, %rbp\n");
        
        // 计算栈空间需求
        let mut max_stack = 0;
        for inst in &func.body {
            if let Instruction::Alloca { dest, ty } = inst {
                let size = self.type_size(ty);
                max_stack += ((size + 7) / 8) * 8;  // 8字节对齐
                var_offsets.insert(dest.clone(), max_stack);
            }
        }
        
        if max_stack > 0 {
            output.push_str(&format!("    subq    ${}, %rsp\n", max_stack));
        }
        
        // 保存参数到栈上
        // System V ABI: RDI, RSI, RDX, RCX, R8, R9
        let param_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
        for (i, (name, _ty)) in func.params.iter().enumerate() {
            if i < param_regs.len() {
                let offset = (i + 1) * 8;
                output.push_str(&format!("    movq    %{}, -{}(%rbp)  # param: {}\n", 
                    param_regs[i], offset, name));
            }
        }
        
        output.push_str("\n");
        
        // 生成指令
        for inst in &func.body {
            output.push_str(&self.generate_instruction(inst, &var_offsets));
        }
        
        // 函数尾声（epilogue） - 如果没有显式返回
        if !func.body.iter().any(|i| matches!(i, Instruction::Return(_) | Instruction::ReturnInPlace(_))) {
            output.push_str(&self.generate_epilogue());
        }
        
        output.push_str(&format!("    .size {}, .-{}\n\n", func.name, func.name));
        output
    }
    
    /// 生成指令
    fn generate_instruction(&self, inst: &Instruction, var_offsets: &std::collections::HashMap<String, usize>) -> String {
        match inst {
            Instruction::Add { dest, left, right } => {
                format!(
                    "    movl    {}(%rbp), %eax  # load {}\n\
                         addl    {}(%rbp), %eax  # add {}\n\
                         movl    %eax, {}(%rbp)  # store {}\n",
                    self.get_var_offset(left, var_offsets), left,
                    self.get_var_offset(right, var_offsets), right,
                    self.get_var_offset(dest, var_offsets), dest
                )
            },
            Instruction::Sub { dest, left, right } => {
                format!(
                    "    movl    {}(%rbp), %eax  # load {}\n\
                         subl    {}(%rbp), %eax  # sub {}\n\
                         movl    %eax, {}(%rbp)  # store {}\n",
                    self.get_var_offset(left, var_offsets), left,
                    self.get_var_offset(right, var_offsets), right,
                    self.get_var_offset(dest, var_offsets), dest
                )
            },
            Instruction::Mul { dest, left, right } => {
                format!(
                    "    movl    {}(%rbp), %eax  # load {}\n\
                         imull   {}(%rbp), %eax  # mul {}\n\
                         movl    %eax, {}(%rbp)  # store {}\n",
                    self.get_var_offset(left, var_offsets), left,
                    self.get_var_offset(right, var_offsets), right,
                    self.get_var_offset(dest, var_offsets), dest
                )
            },
            Instruction::Div { dest, left, right } => {
                format!(
                    "    movl    {}(%rbp), %eax  # load {}\n\
                         cltd                     # sign extend\n\
                         idivl   {}(%rbp)         # div {}\n\
                         movl    %eax, {}(%rbp)  # store {}\n",
                    self.get_var_offset(left, var_offsets), left,
                    self.get_var_offset(right, var_offsets), right,
                    self.get_var_offset(dest, var_offsets), dest
                )
            },
            Instruction::Store { dest, src } => {
                format!(
                    "    movl    {}(%rbp), %eax  # load {}\n\
                         movl    %eax, {}(%rbp)  # store to {}\n",
                    self.get_var_offset(src, var_offsets), src,
                    self.get_var_offset(dest, var_offsets), dest
                )
            },
            Instruction::Load { dest, src } => {
                format!(
                    "    movl    {}(%rbp), %eax  # load from {}\n\
                         movl    %eax, {}(%rbp)  # store to {}\n",
                    self.get_var_offset(src, var_offsets), src,
                    self.get_var_offset(dest, var_offsets), dest
                )
            },
            Instruction::Alloca { dest, ty } => {
                format!("    # alloca {} (type: {:?})\n", dest, ty)
            },
            Instruction::Return(Some(value)) => {
                format!(
                    "    movl    {}(%rbp), %eax  # return value\n{}",
                    self.get_var_offset(value, var_offsets),
                    self.generate_epilogue()
                )
            },
            Instruction::Return(None) => {
                format!(
                    "    xorl    %eax, %eax      # return 0\n{}",
                    self.generate_epilogue()
                )
            },
            Instruction::ReturnInPlace(value) => {
                format!(
                    "    movl    {}(%rbp), %eax  # RVO return\n{}",
                    self.get_var_offset(value, var_offsets),
                    self.generate_epilogue()
                )
            },
            Instruction::Call { dest, func, args } => {
                let mut code = String::new();
                
                // 参数传递（System V ABI）
                let param_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                for (i, arg) in args.iter().enumerate() {
                    if i < param_regs.len() {
                        code.push_str(&format!(
                            "    movl    {}(%rbp), %{}  # arg {}\n",
                            self.get_var_offset(arg, var_offsets),
                            param_regs[i].chars().skip(1).collect::<String>(),
                            i
                        ));
                    }
                }
                
                code.push_str(&format!("    call    {}\n", func));
                
                if let Some(d) = dest {
                    code.push_str(&format!(
                        "    movl    %eax, {}(%rbp)  # save return\n",
                        self.get_var_offset(d, var_offsets)
                    ));
                }
                
                code
            },
            Instruction::Lt { dest, left, right } => {
                format!(
                    "    movl    {}(%rbp), %eax  # load {}\n\
                         cmpl    {}(%rbp), %eax  # compare {}\n\
                         setl    %al              # set if less\n\
                         movzbl  %al, %eax\n\
                         movl    %eax, {}(%rbp)  # store {}\n",
                    self.get_var_offset(left, var_offsets), left,
                    self.get_var_offset(right, var_offsets), right,
                    self.get_var_offset(dest, var_offsets), dest
                )
            },
            Instruction::CondBr { cond, true_bb, false_bb } => {
                format!(
                    "    cmpl    $0, {}(%rbp)    # test {}\n\
                         jne     .L{}            # jump if true\n\
                         jmp     .L{}            # jump if false\n",
                    self.get_var_offset(cond, var_offsets), cond,
                    true_bb, false_bb
                )
            },
            Instruction::Br(label) => {
                format!("    jmp     .L{}\n", label)
            },
            Instruction::Label(name) => {
                format!(".L{}:\n", name)
            },
            _ => format!("    # {:?}\n", inst),
        }
    }
    
    /// 获取变量在栈上的偏移
    fn get_var_offset(&self, var: &str, var_offsets: &std::collections::HashMap<String, usize>) -> String {
        if let Some(offset) = var_offsets.get(var) {
            format!("-{}", offset)
        } else {
            // 可能是立即数或临时变量
            format!("-8")  // 默认偏移
        }
    }
    
    /// 生成函数尾声
    fn generate_epilogue(&self) -> String {
        format!(
            "    movq    %rbp, %rsp\n\
                 popq    %rbp\n\
                 ret\n"
        )
    }
}

impl CodegenBackend for AsmBackend {
    fn name(&self) -> &str {
        if self.use_intel_syntax {
            "x86-64 Assembly (Intel)"
        } else {
            "x86-64 Assembly (AT&T)"
        }
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // 汇编头部
        output.push_str("# Generated by Chim Compiler - x86-64 Assembly Backend\n");
        output.push_str("# Target: x86_64-linux-gnu\n");
        output.push_str("# ABI: System V AMD64\n");
        output.push_str("# Syntax: AT&T\n");
        output.push_str("#\n\n");
        
        output.push_str("    .text\n");
        output.push_str("    .file   \"chim_module.s\"\n\n");
        
        // 生成所有函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        // 数据段（如果有全局变量）
        if !module.globals.is_empty() {
            output.push_str("    .data\n");
            for global in &module.globals {
                output.push_str(&format!(
                    "    .globl {}\n\
                     {}:\n\
                         .zero   {}  # {:?}\n",
                    global.name,
                    global.name,
                    self.type_size(&global.ty),
                    global.ty
                ));
            }
        }
        
        // 结束标记
        output.push_str("\n    .section .note.GNU-stack,\"\",@progbits\n");
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "s"
    }
    
    fn supports_optimization(&self) -> bool {
        false  // 汇编代码已经是最底层
    }
}
