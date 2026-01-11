// ==================== 字符串标准库 ====================
// 字符串操作、查找、替换、格式化等功能

pub struct String {
    data: string,
    length: int,
}

impl String {
    // 构造函数
    pub fn new() -> String {
        String { data: "", length: 0 }
    }
    
    pub fn from(s: string) -> String {
        String { 
            data: s, 
            length: string_len(s) 
        }
    }
    
    // 属性
    pub fn len(&self) -> int {
        self.length
    }
    
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
    
    // 访问
    pub fn char_at(&self, index: int) -> string {
        if index < 0 || index >= self.length {
            return "";
        }
        string_substr(self.data, index, 1)
    }
    
    pub fn bytes(&self) -> &[byte] {
        __string_bytes(&self.data)
    }
    
    // 查找
    pub fn find(&self, substr: string) -> int {
        __string_find(self.data, substr)
    }
    
    pub fn find_from(&self, substr: string, start: int) -> int {
        __string_find_from(self.data, substr, start)
    }
    
    pub fn rfind(&self, substr: string) -> int {
        __string_rfind(self.data, substr)
    }
    
    pub fn contains(&self, substr: string) -> bool {
        __string_find(self.data, substr) >= 0
    }
    
    pub fn starts_with(&self, prefix: string) -> bool {
        __string_starts_with(self.data, prefix)
    }
    
    pub fn ends_with(&self, suffix: string) -> bool {
        __string_ends_with(self.data, suffix)
    }
    
    // 截取
    pub fn substr(&self, start: int) -> String {
        if start >= self.length { return String::new(); }
        String::from(string_substr(self.data, start, self.length - start))
    }
    
    pub fn substr_range(&self, start: int, len: int) -> String {
        if start < 0 || len <= 0 { return String::new(); }
        String::from(string_substr(self.data, start, len))
    }
    
    // 修改
    pub fn trim(&self) -> String {
        let start = 0;
        let end = self.length;
        String::from(__string_trim(self.data))
    }
    
    pub fn trim_start(&self) -> String {
        String::from(__string_trim_start(self.data))
    }
    
    pub fn trim_end(&self) -> String {
        String::from(__string_trim_end(self.data))
    }
    
    pub fn to_lowercase(&self) -> String {
        String::from(__string_lowercase(self.data))
    }
    
    pub fn to_uppercase(&self) -> String {
        String::from(__string_uppercase(self.data))
    }
    
    // 转换
    pub fn to_int(&self) -> Option<int> {
        let n = __string_to_int(self.data);
        if n == 0 && !self.contains("0") {
            Option::None
        } else {
            Option::Some(n)
        }
    }
    
    pub fn to_float(&self) -> Option<float> {
        let f = __string_to_float(self.data);
        if f == 0.0 && !self.contains("0") {
            Option::None
        } else {
            Option::Some(f)
        }
    }
    
    // 分割与连接
    pub fn split(&self, sep: string) -> &[string] {
        __string_split(self.data, sep)
    }
    
    pub fn join(&self, parts: &[string]) -> string {
        __string_join(self.data, parts)
    }
    
    // 重复
    pub fn repeat(&self, count: int) -> String {
        String::from(__string_repeat(self.data, count))
    }
    
    // 替换
    pub fn replace(&self, old: string, new: string) -> String {
        String::from(__string_replace(self.data, old, new))
    }
    
    pub fn replace_once(&self, old: string, new: string) -> String {
        String::from(__string_replace_once(self.data, old, new))
    }
}

// 运算符重载
impl String {
    pub fn add(&self, other: &String) -> String {
        String::from(self.data + other.data)
    }
}

// 字符串字面量函数
pub fn string(s: string) -> String {
    String::from(s)
}

pub fn string_from_chars(chars: &[char]) -> string {
    __chars_to_string(chars)
}

pub fn string_to_chars(s: string) -> &[char] {
    __string_to_chars(s)
}
