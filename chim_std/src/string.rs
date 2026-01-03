//! 字符串处理模块

/// 字符串拼接
pub fn concat(strings: &[&str]) -> String {
    strings.concat()
}

/// 字符串分割
pub fn split(s: &str, delimiter: char) -> Vec<String> {
    s.split(delimiter).map(|s| s.to_string()).collect()
}

/// 字符串包含检测
pub fn contains(s: &str, pattern: &str) -> bool {
    s.contains(pattern)
}

/// 字符串前缀检测
pub fn starts_with(s: &str, prefix: &str) -> bool {
    s.starts_with(prefix)
}

/// 字符串后缀检测
pub fn ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

/// 字符串替换
pub fn replace(s: &str, from: &str, to: &str) -> String {
    s.replace(from, to)
}

pub fn len(s: &str) -> usize {
    s.len()
}

pub fn to_uppercase(s: &str) -> String {
    s.to_uppercase()
}

pub fn to_lowercase(s: &str) -> String {
    s.to_lowercase()
}

/// 移除两端空白
pub fn trim(s: &str) -> String {
    s.trim().to_string()
}

/// 移除左端空白
pub fn trim_start(s: &str) -> String {
    s.trim_start().to_string()
}

/// 移除右端空白
pub fn trim_end(s: &str) -> String {
    s.trim_end().to_string()
}

/// 判断是否为空
pub fn is_empty(s: &str) -> bool {
    s.is_empty()
}

/// 字符串切片
pub fn substring(s: &str, start: usize, end: usize) -> String {
    s.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

/// 查找子串位置
pub fn find(s: &str, pattern: &str) -> Option<usize> {
    s.find(pattern)
}

/// 字符串重复
pub fn repeat(s: &str, count: usize) -> String {
    s.repeat(count)
}

/// 字符串反转
pub fn reverse(s: &str) -> String {
    s.chars().rev().collect()
}

/// 分行
pub fn lines(s: &str) -> Vec<String> {
    s.lines().map(|line| line.to_string()).collect()
}

/// 字符串构建器
pub struct StringBuilder {
    buffer: String,
}

impl StringBuilder {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: String::with_capacity(capacity),
        }
    }
    
    pub fn append(&mut self, s: &str) -> &mut Self {
        self.buffer.push_str(s);
        self
    }
    
    pub fn append_char(&mut self, c: char) -> &mut Self {
        self.buffer.push(c);
        self
    }
    
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
    
    pub fn to_string(&self) -> String {
        self.buffer.clone()
    }
    
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl Default for StringBuilder {
    fn default() -> Self {
        Self::new()
    }
}
