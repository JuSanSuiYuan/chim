// ==================== I/O 标准库 ====================
// 文件操作、标准输入输出、网络等功能

// ==================== 标准输入输出 ====================
pub struct Stdout;

impl Stdout {
    pub fn new() -> Stdout {
        Stdout
    }
    
    pub fn write(&self, s: string) {
        __print_str(s);
    }
    
    pub fn writeln(&self, s: string) {
        __print_str(s);
        __print_str("\n");
    }
    
    pub fn write_int(&self, n: int) {
        __print_int(n);
    }
    
    pub fn write_float(&self, f: float) {
        __print_float(f);
    }
    
    pub fn flush(&self) {
        __stdout_flush();
    }
}

pub struct Stdin;

impl Stdin {
    pub fn new() -> Stdin {
        Stdin
    }
    
    pub fn read_line(&self) -> string {
        __read_line()
    }
    
    pub fn read_int(&self) -> int {
        __read_int()
    }
    
    pub fn read_float(&self) -> float {
        __read_float()
    }
    
    pub fn read_char(&self) -> string {
        __read_char()
    }
    
    pub fn has_input(&self) -> bool {
        __stdin_has_input()
    }
}

// 全局实例
pub let stdout = Stdout::new();
pub let stdin = Stdin::new();

// ==================== 文件系统 ====================
pub struct File {
    path: string,
    handle: int,  // 文件句柄
    closed: bool,
}

impl File {
    pub fn open(path: string, mode: string) -> Option<File> {
        let handle = __file_open(path, mode);
        if handle >= 0 {
            Option::Some(File { path, handle, closed: false })
        } else {
            Option::None
        }
    }
    
    pub fn create(path: string) -> Option<File> {
        File::open(path, "w")
    }
    
    pub fn open_read(path: string) -> Option<File> {
        File::open(path, "r")
    }
    
    pub fn open_append(path: string) -> Option<File> {
        File::open(path, "a")
    }
    
    pub fn close(&mut self) {
        if !self.closed {
            __file_close(self.handle);
            self.closed = true;
        }
    }
    
    pub fn is_closed(&self) -> bool {
        self.closed
    }
    
    // 读取
    pub fn read(&self, size: int) -> string {
        __file_read(self.handle, size)
    }
    
    pub fn read_line(&self) -> string {
        __file_read_line(self.handle)
    }
    
    pub fn read_all(&self) -> string {
        __file_read_all(self.handle)
    }
    
    pub fn read_bytes(&self, size: int) -> &[byte] {
        __file_read_bytes(self.handle, size)
    }
    
    // 写入
    pub fn write(&self, s: string) {
        __file_write(self.handle, s);
    }
    
    pub fn writeln(&self, s: string) {
        __file_write(self.handle, s);
        __file_write(self.handle, "\n");
    }
    
    pub fn write_bytes(&self, bytes: &[byte]) {
        __file_write_bytes(self.handle, bytes);
    }
    
    // 位置
    pub fn seek(&self, pos: int) {
        __file_seek(self.handle, pos);
    }
    
    pub fn tell(&self) -> int {
        __file_tell(self.handle)
    }
    
    pub fn seek_end(&self) {
        __file_seek_end(self.handle);
    }
    
    // 其他
    pub fn flush(&self) {
        __file_flush(self.handle);
    }
    
    pub fn size(&self) -> int {
        let pos = self.tell();
        self.seek_end();
        let size = self.tell();
        self.seek(pos);
        size
    }
    
    pub fn exists(path: string) -> bool {
        __file_exists(path)
    }
    
    pub fn delete(path: string) -> bool {
        __file_delete(path)
    }
    
    pub fn rename(old_path: string, new_path: string) -> bool {
        __file_rename(old_path, new_path)
    }
    
    pub fn copy(src: string, dest: string) -> bool {
        __file_copy(src, dest)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        self.close();
    }
}

// ==================== 路径操作 ====================
pub struct Path {
    path: string,
}

impl Path {
    pub fn new(p: string) -> Path {
        Path { path: p }
    }
    
    pub fn file_name(&self) -> string {
        __path_file_name(self.path)
    }
    
    pub fn parent(&self) -> Option<Path> {
        let parent = __path_parent(self.path);
        if parent != "" { Option::Some(Path::new(parent)) }
        else { Option::None }
    }
    
    pub fn extension(&self) -> string {
        __path_extension(self.path)
    }
    
    pub fn without_extension(&self) -> string {
        __path_without_extension(self.path)
    }
    
    pub fn join(&self, other: &Path) -> Path {
        let joined = __path_join(self.path, other.path);
        Path::new(joined)
    }
    
    pub fn is_absolute(&self) -> bool {
        __path_is_absolute(self.path)
    }
    
    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }
    
    pub fn canonicalize(&self) -> Option<Path> {
        let canonical = __path_canonicalize(self.path);
        if canonical != "" { Option::Some(Path::new(canonical)) }
        else { Option::None }
    }
}

// ==================== 目录操作 ====================
pub struct Dir {
    path: string,
}

impl Dir {
    pub fn create(path: string) -> bool {
        __dir_create(path)
    }
    
    pub fn create_all(path: string) -> bool {
        __dir_create_all(path)
    }
    
    pub fn remove(path: string) -> bool {
        __dir_remove(path)
    }
    
    pub fn remove_all(path: string) -> bool {
        __dir_remove_all(path)
    }
    
    pub fn exists(path: string) -> bool {
        __dir_exists(path)
    }
    
    pub fn is_empty(path: string) -> bool {
        __dir_is_empty(path)
    }
    
    pub fn entries(path: string) -> Vec<string> {
        __dir_entries(path)
    }
    
    pub fn current() -> Path {
        Path::new(__dir_current())
    }
    
    pub fn set_current(path: string) -> bool {
        __dir_set_current(path)
    }
    
    pub fn temp() -> Path {
        Path::new(__dir_temp())
    }
    
    pub fn home() -> Path {
        Path::new(__dir_home())
    }
}
