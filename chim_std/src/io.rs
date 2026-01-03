//! IO 模块

use std::io::{self, Write, BufRead, BufReader, Read};
use std::fs::{File, OpenOptions};
use std::path::Path;

pub type IOError = String;
pub type IOResult<T> = Result<T, IOError>;

pub fn print(s: &str) {
    print!("{}", s);
    let _ = io::stdout().flush();
}

pub fn println(s: &str) {
    println!("{}", s);
}

/// 从标准输入读取一行
pub fn read_line() -> IOResult<String> {
    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .map_err(|e| e.to_string())?;
    
    // 移除末尾的换行符
    if buffer.ends_with('\n') {
        buffer.pop();
        if buffer.ends_with('\r') {
            buffer.pop();
        }
    }
    
    Ok(buffer)
}

/// 文件读写
pub struct FileHandle {
    file: File,
}

impl FileHandle {
    /// 打开文件读取
    pub fn open<P: AsRef<Path>>(path: P) -> IOResult<Self> {
        File::open(path)
            .map(|file| FileHandle { file })
            .map_err(|e| e.to_string())
    }
    
    /// 创建文件写入
    pub fn create<P: AsRef<Path>>(path: P) -> IOResult<Self> {
        File::create(path)
            .map(|file| FileHandle { file })
            .map_err(|e| e.to_string())
    }
    
    /// 追加模式打开文件
    pub fn append<P: AsRef<Path>>(path: P) -> IOResult<Self> {
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .map(|file| FileHandle { file })
            .map_err(|e| e.to_string())
    }
    
    /// 读取文件所有内容
    pub fn read_to_string(&mut self) -> IOResult<String> {
        let mut contents = String::new();
        self.file
            .read_to_string(&mut contents)
            .map_err(|e| e.to_string())?;
        Ok(contents)
    }
    
    /// 逐行读取
    pub fn read_lines(&self) -> IOResult<Vec<String>> {
        let reader = BufReader::new(&self.file);
        let mut lines = Vec::new();
        
        for line in reader.lines() {
            lines.push(line.map_err(|e| e.to_string())?);
        }
        
        Ok(lines)
    }
    
    /// 写入字符串
    pub fn write_str(&mut self, s: &str) -> IOResult<()> {
        self.file
            .write_all(s.as_bytes())
            .map_err(|e| e.to_string())
    }
    
    /// 写入字节
    pub fn write_bytes(&mut self, bytes: &[u8]) -> IOResult<()> {
        self.file
            .write_all(bytes)
            .map_err(|e| e.to_string())
    }
    
    /// 刷新缓冲区
    pub fn flush(&mut self) -> IOResult<()> {
        self.file
            .flush()
            .map_err(|e| e.to_string())
    }
}
