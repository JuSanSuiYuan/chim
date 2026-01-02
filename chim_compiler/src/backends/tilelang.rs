use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// TileLang后端
/// 生成TileLang代码，国产AI编程语言
/// 由北京大学计算机学院杨智团队主导开发
/// 支持CUDA和国产算力芯片（如昇腾、寒武纪等）
/// 已应用于DeepSeek v3.2等前沿AI模型
pub struct TileLangBackend {
    pub tile_size: usize,
    pub use_flash_attention: bool,
    pub vectorize: bool,
    pub target_device: String, // "cuda" 或 "ascend" 或 "cambricon"
}

impl TileLangBackend {
    pub fn new() -> Self {
        Self {
            tile_size: 32,
            use_flash_attention: true,
            vectorize: true,
            target_device: "cuda".to_string(), // 默认CUDA，也支持国产芯片
        }
    }
    
    fn tile_type(&self, ty: &Type) -> String {
        match ty {
            Type::Int32 | Type::Int64 => "i32".to_string(),
            Type::Float32 => "f32".to_string(),
            Type::Float64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Void => "()".to_string(),
            Type::String => "str".to_string(),
            Type::Array(elem_ty, size) => {
                format!("Tile[{}, {}]", self.tile_type(elem_ty), size)
            },
            _ => "f32".to_string(),
        }
    }
    
    fn generate_kernel(&self, func: &Function) -> String {
        let mut code = String::new();
        
        // TileLang语法：使用@tile装饰器
        code.push_str("@tile\n");
        if self.vectorize {
            code.push_str("@vectorize\n");
        }
        code.push_str(&format!("@tile_size({})\n", self.tile_size));
        
        // 函数定义 - TileLang使用def关键字（类似Python）
        code.push_str("def ");
        code.push_str(&func.name);
        code.push_str("(");
        
        // 参数 - 使用Tile类型（TileLang核心概念）
        let params: Vec<String> = func.params.iter()
            .map(|(name, _ty)| {
                // TileLang的Tile类型语法
                format!("{}: Tile[{}, {}]", name, self.tile_size, self.tile_size)
            })
            .collect();
        code.push_str(&params.join(", "));
        
        code.push_str(&format!(") -> Tile[{}, {}]:\n", self.tile_size, self.tile_size));
        
        // 注释：TileLang自动优化特性
        code.push_str(&format!("    \"\"\"\n"));
        code.push_str(&format!("    TileLang自动优化：\n"));
        code.push_str(&format!("    - 线程绑定优化\n"));
        code.push_str(&format!("    - 内存布局优化\n"));
        code.push_str(&format!("    - 流水线并行\n"));
        code.push_str(&format!("    目标设备: {}\n", self.target_device));
        code.push_str(&format!("    \"\"\"\n"));
        
        // 初始化结果Tile
        code.push_str(&format!("    result = Tile.zeros({}, {})\n\n", 
            self.tile_size, self.tile_size));
        
        // Tile分块循环（TileLang核心）
        code.push_str(&format!("    # Tile分块计算（{}x{}）\n", self.tile_size, self.tile_size));
        code.push_str(&format!("    for i in tile_range(0, {}):\n", self.tile_size));
        code.push_str(&format!("        for j in tile_range(0, {}):\n", self.tile_size));
        
        // 函数体
        for inst in &func.body {
            code.push_str("            ");
            code.push_str(&self.generate_instruction(inst));
            code.push_str("\n");
        }
        
        code.push_str("\n    return result\n\n");
        code
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Alloca { dest, ty } => {
                format!("var {}: {}", dest, self.tile_type(ty))
            },
            Instruction::Store { dest, src } => {
                format!("{} = {}", dest, src)
            },
            Instruction::Load { dest, src } => {
                format!("let {} = {}", dest, src)
            },
            Instruction::Add { dest, left, right } => {
                // Tile addition (element-wise)
                format!("let {} = {} + {}", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("let {} = {} - {}", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                // Tile multiplication (matrix multiply for 2D tiles)
                format!("let {} = tile_matmul({}, {})", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("let {} = {} / {}", dest, left, right)
            },
            Instruction::Call { dest, func, args } => {
                // TileLang built-in operations
                let tile_func = match func.as_str() {
                    "matmul" => "tile_matmul",
                    "transpose" => "tile_transpose",
                    "softmax" => "tile_softmax",
                    "relu" => "tile_relu",
                    "gelu" => "tile_gelu",
                    _ => func.as_str(),
                };
                
                if let Some(d) = dest {
                    format!("let {} = {}({})", d, tile_func, args.join(", "))
                } else {
                    format!("{}({})", tile_func, args.join(", "))
                }
            },
            Instruction::Return(val) => {
                if let Some(v) = val {
                    format!("return {}", v)
                } else {
                    "return".to_string()
                }
            },
            _ => "# Unsupported instruction".to_string(),
        }
    }
    
    fn generate_flash_attention(&self) -> String {
        let mut code = String::new();
        
        code.push_str("# FlashAttention - TileLang高效实现\n");
        code.push_str("# 已应用于DeepSeek v3.2 MLA注意力机制\n");
        code.push_str("@tile\n");
        code.push_str("@flash_attention  # TileLang内置优化\n");
        code.push_str(&format!("@tile_size({})\n", self.tile_size));
        code.push_str("def flash_attention(\n");
        code.push_str(&format!("    Q: Tile[{0}, {0}],\n", self.tile_size));
        code.push_str(&format!("    K: Tile[{0}, {0}],\n", self.tile_size));
        code.push_str(&format!("    V: Tile[{0}, {0}]\n", self.tile_size));
        code.push_str(&format!(") -> Tile[{}, {}]:\n", self.tile_size, self.tile_size));
        code.push_str("    \"\"\"\n");
        code.push_str("    FlashAttention优化实现\n");
        code.push_str("    - 在线softmax降低内存占用\n");
        code.push_str("    - Tile分块提升缓存命中率\n");
        code.push_str("    - TileLang自动流水线并行\n");
        code.push_str("    \"\"\"\n");
        code.push_str("    O = Tile.zeros()\n");
        code.push_str("    m = Tile.full(-float('inf'))\n");
        code.push_str("    l = Tile.zeros()\n\n");
        
        code.push_str("    # Tile分块注意力计算\n");
        code.push_str(&format!("    for i in tile_range(0, {}):\n", self.tile_size));
        code.push_str("        Q_tile = Q.load_tile(i)\n\n");
        
        code.push_str(&format!("        for j in tile_range(0, {}):\n", self.tile_size));
        code.push_str("            K_tile = K.load_tile(j)\n");
        code.push_str("            V_tile = V.load_tile(j)\n\n");
        
        code.push_str("            # 计算注意力分数（TileLang自动优化matmul）\n");
        code.push_str("            S = tile_matmul(Q_tile, K_tile.T)\n");
        code.push_str("            S_scaled = S / sqrt(d_k)\n\n");
        
        code.push_str("            # 在线softmax更新（内存高效）\n");
        code.push_str("            m_new = tile_max(m, tile_max(S_scaled))\n");
        code.push_str("            l_new = exp(m - m_new) * l + tile_sum(exp(S_scaled - m_new))\n");
        code.push_str("            O = exp(m - m_new) * O + tile_matmul(exp(S_scaled - m_new), V_tile)\n");
        code.push_str("            m = m_new\n");
        code.push_str("            l = l_new\n\n");
        
        code.push_str("    return O / l\n\n");
        code
    }
    
    fn generate_builtins(&self) -> String {
        let mut code = String::new();
        
        code.push_str("# TileLang内置算子（Built-in Operators）\n");
        code.push_str("# 这些算子由TileLang编译器自动优化\n\n");
        
        // 矩阵乘法
        code.push_str("@tile\n");
        code.push_str("def tile_matmul(A: Tile, B: Tile) -> Tile:\n");
        code.push_str("    \"\"\"\n");
        code.push_str("    优化的Tile矩阵乘法\n");
        code.push_str("    TileLang自动处理：\n");
        code.push_str("    - 内存合并访问\n");
        code.push_str("    - 共享内存利用\n");
        code.push_str("    - 寄存器分块\n");
        code.push_str("    \"\"\"\n");
        code.push_str("    C = Tile.zeros()\n");
        code.push_str("    for k in tile_range(tile_size):\n");
        code.push_str("        C += A[:, k] @ B[k, :]  # @运算符表示矩阵乘法\n");
        code.push_str("    return C\n\n");
        
        // 转置
        code.push_str("@tile\n");
        code.push_str("def tile_transpose(A: Tile) -> Tile:\n");
        code.push_str("    \"\"\"Tile转置（无需手动优化）\"\"\"\n");
        code.push_str("    return A.T\n\n");
        
        // Softmax
        code.push_str("@tile\n");
        code.push_str("def tile_softmax(A: Tile) -> Tile:\n");
        code.push_str("    \"\"\"\n");
        code.push_str("    数值稳定的Softmax\n");
        code.push_str("    TileLang自动向量化\n");
        code.push_str("    \"\"\"\n");
        code.push_str("    max_val = tile_max(A)\n");
        code.push_str("    exp_vals = exp(A - max_val)\n");
        code.push_str("    return exp_vals / tile_sum(exp_vals)\n\n");
        
        // ReLU
        code.push_str("@tile\n");
        code.push_str("def tile_relu(A: Tile) -> Tile:\n");
        code.push_str("    \"\"\"ReLU激活函数\"\"\"\n");
        code.push_str("    return maximum(A, 0)\n\n");
        
        // GELU
        code.push_str("@tile\n");
        code.push_str("def tile_gelu(A: Tile) -> Tile:\n");
        code.push_str("    \"\"\"GELU激活函数（用于Transformer）\"\"\"\n");
        code.push_str("    return 0.5 * A * (1 + tanh(sqrt(2/pi) * (A + 0.044715 * A**3)))\n\n");
        
        code
    }
}

impl CodegenBackend for TileLangBackend {
    fn name(&self) -> &str {
        "tilelang"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = String::new();
        
        // Header
        code.push_str("# Generated by Chim Compiler - TileLang Backend\n");
        code.push_str("# TileLang: 国产AI编程语言\n");
        code.push_str("# 由北京大学计算机学院杨智团队主导开发\n");
        code.push_str("# GitHub: https://github.com/tilelang/tilelang\n");
        code.push_str(&format!("# Tile大小: {}x{}\n", self.tile_size, self.tile_size));
        code.push_str(&format!("# 目标设备: {}\n", self.target_device));
        code.push_str("# \n");
        code.push_str("# 特性：\n");
        code.push_str("# - 支持CUDA和国产算力芯片（昇腾、寒武纪等）\n");
        code.push_str("# - 自动线程绑定和内存布局优化\n");
        code.push_str("# - 自动流水线并行\n");
        code.push_str("# - 已应用于DeepSeek v3.2等前沿AI模型\n\n");
        
        // Imports
        code.push_str("from tilelang import Tile, tile_range\n");
        code.push_str("from tilelang.math import sqrt, exp, log, tanh, maximum\n");
        code.push_str("from tilelang.ops import tile_matmul, tile_max, tile_sum\n");
        code.push_str("import math\n\n");
        
        // 常量定义
        code.push_str("# 常量\n");
        code.push_str(&format!("tile_size = {}\n", self.tile_size));
        code.push_str("pi = math.pi\n\n");
        
        // Built-in operations
        code.push_str(&self.generate_builtins());
        
        // FlashAttention if enabled
        if self.use_flash_attention {
            code.push_str(&self.generate_flash_attention());
        }
        
        // User kernels
        code.push_str("# 用户定义的Kernel\n");
        for func in &module.functions {
            code.push_str(&self.generate_kernel(func));
        }
        
        // Main execution
        code.push_str("# 主执行函数\n");
        code.push_str("if __name__ == '__main__':\n");
        code.push_str("    print(\"TileLang - 国产AI编程语言\")\n");
        code.push_str("    print(\"北京大学计算机学院杨智团队开发\")\n");
        code.push_str(&format!("    print(\"Tile大小: {}x{}\")\n", self.tile_size, self.tile_size));
        code.push_str(&format!("    print(\"向量化: {}\")\n", self.vectorize));
        code.push_str(&format!("    print(\"FlashAttention: {}\")\n", self.use_flash_attention));
        code.push_str(&format!("    print(\"目标设备: {}\")\n\n", self.target_device));
        
        for func in &module.functions {
            code.push_str(&format!("    # 执行 {}\n", func.name));
            code.push_str(&format!("    # result = {}(tiles...)\n", func.name));
            code.push_str("    # print(result)\n\n");
        }
        
        Ok(code)
    }
    
    fn file_extension(&self) -> &str {
        ".tile"
    }
    
    fn supports_optimization(&self) -> bool {
        true
    }
}
