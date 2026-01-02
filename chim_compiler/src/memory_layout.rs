use std::collections::HashMap;
use crate::ast::StructField;

/// 结构体内存布局信息
#[derive(Debug, Clone)]
pub struct StructLayout {
    pub size: usize,
    pub alignment: usize,
    pub field_offsets: Vec<usize>,
    pub original_order: Vec<String>,
    pub optimized_order: Vec<String>,
    pub padding_bytes: usize,  // 填充字节数
    pub cache_aligned: bool,   // 是否缓存对齐
}

/// 内存布局分析器
/// 
/// 负责分析结构体的内存布局，并应用优化：
/// - 字段重排（按对齐要求从大到小排序）
/// - 填充消除（减少内存浪费）
/// - SIMD对齐（为向量化做准备）
/// - 缓存行对齐（提高缓存效率）
pub struct MemoryLayoutAnalyzer {
    layouts: HashMap<String, StructLayout>,
    value_types: HashMap<String, bool>,  // 标记值类型
    simd_alignment: usize,  // SIMD对齐要求
}

impl MemoryLayoutAnalyzer {
    pub fn new() -> Self {
        Self {
            layouts: HashMap::new(),
            value_types: HashMap::new(),
            simd_alignment: 16,  // 默认16字节对齐（SSE）
        }
    }
    
    /// 设置SIMD对齐要求
    pub fn set_simd_alignment(&mut self, align: usize) {
        self.simd_alignment = align;
    }
    
    /// 标记为值类型
    pub fn mark_value_type(&mut self, name: &str) {
        self.value_types.insert(name.to_string(), true);
    }
    
    /// 检查是否是值类型
    pub fn is_value_type(&self, name: &str) -> bool {
        self.value_types.get(name).copied().unwrap_or(false)
    }
    
    /// 分析并优化结构体布局
    pub fn analyze_struct(&mut self, name: &str, fields: &[StructField]) -> StructLayout {
        // 1. 计算每个字段的大小和对齐
        let mut field_info: Vec<(String, usize, usize)> = fields
            .iter()
            .map(|f| {
                let (size, align) = self.get_type_info(&f.ty);
                (f.name.clone(), size, align)
            })
            .collect();
        
        // 2. 按对齐要求从大到小排序（字段重排优化）
        field_info.sort_by(|a, b| b.2.cmp(&a.2).then(b.1.cmp(&a.1)));
        
        // 3. 计算偏移量
        let mut offset = 0;
        let mut max_align = 1;
        let mut offsets = Vec::new();
        
        for (_, size, align) in &field_info {
            // 对齐到字段要求
            offset = align_up(offset, *align);
            offsets.push(offset);
            offset += size;
            max_align = max_align.max(*align);
        }
        
        // 4. 结构体总大小需要对齐到最大对齐要求
        let total_size = align_up(offset, max_align);
        
        // 5. 计算填充字节
        let padding_bytes = total_size - offset;
        
        // 6. 检查是否需要缓存行对齐（64字节）
        let cache_aligned = total_size >= 64 || self.is_value_type(name);
        
        let layout = StructLayout {
            size: total_size,
            alignment: max_align,
            field_offsets: offsets,
            original_order: fields.iter().map(|f| f.name.clone()).collect(),
            optimized_order: field_info.iter().map(|f| f.0.clone()).collect(),
            padding_bytes,
            cache_aligned,
        };
        
        self.layouts.insert(name.to_string(), layout.clone());
        layout
    }
    
    /// 获取类型的大小和对齐
    fn get_type_info(&self, ty: &str) -> (usize, usize) {
        match ty {
            "int" | "i32" => (4, 4),
            "i64" => (8, 8),
            "float" | "f32" => (4, 4),
            "f64" => (8, 8),
            "bool" => (1, 1),
            "string" => (16, 8), // 假设是指针+长度
            "char" => (1, 1),
            "i8" => (1, 1),
            "i16" => (2, 2),
            "u8" => (1, 1),
            "u16" => (2, 2),
            "u32" => (4, 4),
            "u64" => (8, 8),
            _ => {
                // 查找已定义的结构体
                if let Some(layout) = self.layouts.get(ty) {
                    (layout.size, layout.alignment)
                } else if ty.starts_with("[") {
                    // 数组类型
                    (16, 8)  // 假设是动态数组
                } else {
                    // 默认指针大小
                    (8, 8)
                }
            }
        }
    }
    
    /// 计算填充消除后的节省
    pub fn calculate_savings(&self, name: &str, original_fields: &[StructField]) -> usize {
        // 计算未优化的大小
        let mut naive_size = 0;
        let mut max_align = 1;
        
        for field in original_fields {
            let (size, align) = self.get_type_info(&field.ty);
            naive_size = align_up(naive_size, align);
            naive_size += size;
            max_align = max_align.max(align);
        }
        naive_size = align_up(naive_size, max_align);
        
        // 对比优化后的大小
        let optimized = self.layouts.get(name).unwrap();
        naive_size.saturating_sub(optimized.size)
    }
    
    /// 获取已分析的布局
    pub fn get_layout(&self, name: &str) -> Option<&StructLayout> {
        self.layouts.get(name)
    }
    
    /// 应用SIMD对齐优化
    pub fn apply_simd_alignment(&mut self, name: &str) {
        if let Some(layout) = self.layouts.get_mut(name) {
            if layout.size < self.simd_alignment {
                layout.size = self.simd_alignment;
                layout.alignment = self.simd_alignment;
            }
        }
    }
    
    /// 获取优化报告
    pub fn get_optimization_report(&self, name: &str) -> Option<String> {
        if let Some(layout) = self.layouts.get(name) {
            let report = format!(
                "结构体 '{}' 内存布局优化:\n  \
                大小: {} 字节\n  \
                对齐: {} 字节\n  \
                填充: {} 字节\n  \
                缓存对齐: {}\n  \
                字段顺序优化: {} -> {}",
                name,
                layout.size,
                layout.alignment,
                layout.padding_bytes,
                if layout.cache_aligned { "是" } else { "否" },
                layout.original_order.join(", "),
                layout.optimized_order.join(", ")
            );
            Some(report)
        } else {
            None
        }
    }
}

impl Default for MemoryLayoutAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// 向上对齐辅助函数
fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) / align * align
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_struct_layout_optimization() {
        let mut analyzer = MemoryLayoutAnalyzer::new();
        
        // 定义一个结构体：bool(1字节) + int(4字节) + bool(1字节)
        // 未优化：1 + 3填充 + 4 + 1 + 3填充 = 12字节
        // 优化后：int(4) + bool(1) + bool(1) + 2填充 = 8字节
        let fields = vec![
            StructField { name: "flag1".to_string(), ty: "bool".to_string() },
            StructField { name: "value".to_string(), ty: "int".to_string() },
            StructField { name: "flag2".to_string(), ty: "bool".to_string() },
        ];
        
        let layout = analyzer.analyze_struct("TestStruct", &fields);
        
        assert_eq!(layout.size, 8);
        assert_eq!(layout.alignment, 4);
        
        let savings = analyzer.calculate_savings("TestStruct", &fields);
        assert_eq!(savings, 4); // 节省了4字节
    }
    
    #[test]
    fn test_type_info() {
        let analyzer = MemoryLayoutAnalyzer::new();
        
        assert_eq!(analyzer.get_type_info("int"), (4, 4));
        assert_eq!(analyzer.get_type_info("float"), (4, 4));
        assert_eq!(analyzer.get_type_info("bool"), (1, 1));
        assert_eq!(analyzer.get_type_info("string"), (16, 8));
    }
    
    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 4), 0);
        assert_eq!(align_up(1, 4), 4);
        assert_eq!(align_up(3, 4), 4);
        assert_eq!(align_up(4, 4), 4);
        assert_eq!(align_up(5, 4), 8);
    }
}
