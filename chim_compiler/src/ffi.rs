use std::collections::HashMap;
use std::ffi::{CStr, CString, NulError, OsStr, OsString};
use std::os::raw::{c_char, c_double, c_float, c_int, c_long, c_void};
use std::path::Path;
use std::ptr;

pub type FFIResult<T> = Result<T, FFIError>;

#[derive(Debug, Clone)]
pub enum FFIError {
    NullPointer,
    InvalidUtf8,
    NulError(NulError),
    LibraryLoadError(String),
    SymbolNotFoundError(String),
    TypeMismatch(String),
    CallFailed(String),
}

impl std::fmt::Display for FFIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FFIError::NullPointer => write!(f, "Null pointer error"),
            FFIError::InvalidUtf8 => write!(f, "Invalid UTF-8 string"),
            FFIError::NulError(e) => write!(f, "Nul error: {}", e),
            FFIError::LibraryLoadError(msg) => write!(f, "Library load error: {}", msg),
            FFIError::SymbolNotFoundError(name) => {
                write!(f, "Symbol not found: {}", name)
            }
            FFIError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            FFIError::CallFailed(msg) => write!(f, "Call failed: {}", msg),
        }
    }
}

impl std::error::Error for FFIError {}

impl From<NulError> for FFIError {
    fn from(e: NulError) -> Self {
        FFIError::NulError(e)
    }
}

#[repr(C)]
pub enum FFICType {
    Void,
    Bool,
    Char,
    Short,
    Int,
    Long,
    LongLong,
    Float,
    Double,
    Pointer,
}

#[repr(C)]
pub enum FFICallConvention {
    C,
    StdCall,
    FastCall,
    ThisCall,
}

#[derive(Clone)]
pub struct FFIFunction {
    name: String,
    return_type: FFICType,
    params: Vec<FFICType>,
    convention: FFICallConvention,
}

impl FFIFunction {
    pub fn new(
        name: String,
        return_type: FFICType,
        params: Vec<FFICType>,
        convention: FFICallConvention,
    ) -> Self {
        FFIFunction {
            name,
            return_type,
            params,
            convention,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn return_type(&self) -> &FFICType {
        &self.return_type
    }

    pub fn params(&self) -> &[FFICType] {
        &self.params
    }

    pub fn convention(&self) -> &FFICallConvention {
        &self.convention
    }
}

pub struct FFILibrary {
    name: String,
    handle: *mut c_void,
    functions: HashMap<String, FFIFunction>,
}

unsafe impl Send for FFILibrary {}
unsafe impl Sync for FFILibrary {}

impl FFILibrary {
    pub fn new(name: String) -> FFIResult<Self> {
        let c_name = CString::new(name.clone())?;
        let handle = unsafe { libloading::Library::new(name.as_str()) }
            .map_err(|e| FFIError::LibraryLoadError(e.to_string()))?;

        Ok(FFILibrary {
            name,
            handle: ptr::null_mut(),
            functions: HashMap::new(),
        })
    }

    pub fn load<P: AsRef<Path>>(path: P) -> FFIResult<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let handle = unsafe { libloading::Library::new(path.as_ref()) }
            .map_err(|e| FFIError::LibraryLoadError(e.to_string()))?;

        Ok(FFILibrary {
            name: path_str,
            handle: ptr::null_mut(),
            functions: HashMap::new(),
        })
    }

    pub fn get_function(&self, name: &str) -> FFIResult<FFIFunction> {
        self.functions
            .get(name)
            .cloned()
            .ok_or_else(|| FFIError::SymbolNotFoundError(name.to_string()))
    }

    pub unsafe fn call(&self, func: &FFIFunction, args: &[FFIValue]) -> FFIResult<FFIValue> {
        match func.return_type() {
            FFICType::Void => Ok(FFIValue::Void),
            FFICType::Bool => Ok(FFIValue::Bool(true)),
            FFICType::Int => Ok(FFIValue::Int(0)),
            FFICType::Float => Ok(FFIValue::Float(0.0)),
            FFICType::Double => Ok(FFIValue::Double(0.0)),
            FFICType::Pointer => Ok(FFIValue::Pointer(ptr::null())),
            _ => Err(FFIError::TypeMismatch(format!(
                "Unsupported return type: {:?}",
                func.return_type()
            ))),
        }
    }

    pub fn register_function(&mut self, func: FFIFunction) {
        self.functions.insert(func.name().to_string(), func);
    }
}

impl Drop for FFILibrary {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                libloading::Library::close(self.handle as *mut _);
            }
        }
    }
}

#[derive(Clone)]
pub enum FFIValue {
    Void,
    Bool(bool),
    Char(c_char),
    Short(i16),
    Int(c_int),
    Long(c_long),
    LongLong(i64),
    Float(c_float),
    Double(c_double),
    Pointer(*mut c_void),
}

impl FFIValue {
    pub fn as_bool(&self) -> FFIResult<bool> {
        match self {
            FFIValue::Bool(b) => Ok(*b),
            _ => Err(FFIError::TypeMismatch("Expected bool".to_string())),
        }
    }

    pub fn as_int(&self) -> FFIResult<c_int> {
        match self {
            FFIValue::Int(i) => Ok(*i),
            _ => Err(FFIError::TypeMismatch("Expected int".to_string())),
        }
    }

    pub fn as_float(&self) -> FFIResult<c_float> {
        match self {
            FFIValue::Float(f) => Ok(*f),
            _ => Err(FFIError::TypeMismatch("Expected float".to_string())),
        }
    }

    pub fn as_double(&self) -> FFIResult<c_double> {
        match self {
            FFIValue::Double(d) => Ok(*d),
            _ => Err(FFIError::TypeMismatch("Expected double".to_string())),
        }
    }

    pub fn as_pointer(&self) -> FFIResult<*mut c_void> {
        match self {
            FFIValue::Pointer(p) => Ok(*p),
            _ => Err(FFIError::TypeMismatch("Expected pointer".to_string())),
        }
    }
}

pub struct CStringHelper;

impl CStringHelper {
    pub fn to_c_string(s: &str) -> FFIResult<CString> {
        CString::new(s).map_err(FFIError::NulError)
    }

    pub fn from_c_string(c_str: *const c_char) -> FFIResult<String> {
        if c_str.is_null() {
            return Err(FFIError::NullPointer);
        }
        unsafe {
            CStr::from_ptr(c_str)
                .to_str()
                .map(|s| s.to_string())
                .map_err(|_| FFIError::InvalidUtf8)
        }
    }

    pub fn to_c_char(s: &str) -> FFIResult<*mut c_char> {
        let c_string = Self::to_c_string(s)?;
        Ok(c_string.into_raw())
    }

    pub fn from_c_char(c_char: *mut c_char) -> FFIResult<String> {
        Self::from_c_string(c_char)
    }
}

pub struct FFIBinding {
    name: String,
    library: String,
    functions: Vec<FFIFunction>,
}

impl FFIBinding {
    pub fn new(name: String, library: String) -> Self {
        FFIBinding {
            name,
            library,
            functions: Vec::new(),
        }
    }

    pub fn add_function(&mut self, func: FFIFunction) {
        self.functions.push(func);
    }

    pub fn functions(&self) -> &[FFIFunction] {
        &self.functions
    }

    pub fn library(&self) -> &str {
        &self.library
    }
}

pub mod libc {
    use super::*;

    pub const STDIN_FILENO: c_int = 0;
    pub const STDOUT_FILENO: c_int = 1;
    pub const STDERR_FILENO: c_int = 2;

    extern "C" {
        pub fn printf(format: *const c_char, ...) -> c_int;
        pub fn fprintf(stream: *mut c_void, format: *const c_char, ...) -> c_int;
        pub fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
        pub fn scanf(format: *const c_char, ...) -> c_int;
        pub fn fscanf(stream: *mut c_void, format: *const c_char, ...) -> c_int;
        pub fn sscanf(s: *const c_char, format: *const c_char, ...) -> c_int;
        pub fn malloc(size: usize) -> *mut c_void;
        pub fn free(ptr: *mut c_void);
        pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
        pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
        pub fn strlen(s: *const c_char) -> usize;
        pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
        pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
        pub fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
        pub fn atoi(s: *const c_char) -> c_int;
        pub fn atof(s: *const c_char) -> c_double;
        pub fn fopen(filename: *const c_char, mode: *const c_char) -> *mut c_void;
        pub fn fclose(stream: *mut c_void) -> c_int;
        pub fn fread(
            ptr: *mut c_void,
            size: usize,
            nmemb: usize,
            stream: *mut c_void,
        ) -> usize;
        pub fn fwrite(
            ptr: *const c_void,
            size: usize,
            nmemb: usize,
            stream: *mut c_void,
        ) -> usize;
        pub fn fseek(stream: *mut c_void, offset: c_long, whence: c_int) -> c_int;
        pub fn ftell(stream: *mut c_void) -> c_long;
        pub fn time(tloc: *mut c_long) -> c_long;
        pub fn sleep(seconds: c_uint) -> c_uint;
    }
}

pub mod libm {
    use super::*;

    extern "C" {
        pub fn sin(x: c_double) -> c_double;
        pub fn cos(x: c_double) -> c_double;
        pub fn tan(x: c_double) -> c_double;
        pub fn asin(x: c_double) -> c_double;
        pub fn acos(x: c_double) -> c_double;
        pub fn atan(x: c_double) -> c_double;
        pub fn atan2(y: c_double, x: c_double) -> c_double;
        pub fn sinh(x: c_double) -> c_double;
        pub fn cosh(x: c_double) -> c_double;
        pub fn tanh(x: c_double) -> c_double;
        pub fn exp(x: c_double) -> c_double;
        pub fn log(x: c_double) -> c_double;
        pub fn log10(x: c_double) -> c_double;
        pub fn pow(x: c_double, y: c_double) -> c_double;
        pub fn sqrt(x: c_double) -> c_double;
        pub fn ceil(x: c_double) -> c_double;
        pub fn floor(x: c_double) -> c_double;
        pub fn fabs(x: c_double) -> c_double;
        pub fn fmod(x: c_double, y: c_double) -> c_double;
    }
}

pub mod libpthread {
    use super::*;

    extern "C" {
        pub fn pthread_create(
            thread: *mut usize,
            attr: *const c_void,
            start_routine: extern "C" fn(*mut c_void) -> *mut c_void,
            arg: *mut c_void,
        ) -> c_int;
        pub fn pthread_join(thread: usize, retval: *mut *mut c_void) -> c_int;
        pub fn pthread_detach(thread: usize) -> c_int;
        pub fn pthread_exit(retval: *mut c_void);
        pub fn pthread_self() -> usize;
        pub fn pthread_equal(t1: usize, t2: usize) -> c_int;
        pub fn pthread_mutex_init(
            mutex: *mut c_void,
            attr: *const c_void,
        ) -> c_int;
        pub fn pthread_mutex_lock(mutex: *mut c_void) -> c_int;
        pub fn pthread_mutex_unlock(mutex: *mut c_void) -> c_int;
        pub fn pthread_mutex_destroy(mutex: *mut c_void) -> c_int;
        pub fn pthread_cond_init(cond: *mut c_void, attr: *const c_void) -> c_int;
        pub fn pthread_cond_wait(cond: *mut c_void, mutex: *mut c_void) -> c_int;
        pub fn pthread_cond_signal(cond: *mut c_void) -> c_int;
        pub fn pthread_cond_broadcast(cond: *mut c_void) -> c_int;
        pub fn pthread_cond_destroy(cond: *mut c_void) -> c_int;
    }
}

pub mod libdl {
    use super::*;

    extern "C" {
        pub fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
        pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        pub fn dlclose(handle: *mut c_void) -> c_int;
        pub fn dlerror() -> *const c_char;
    }
}

pub type c_uint = u32;

pub struct FFIBuilder {
    bindings: Vec<FFIBinding>,
}

impl FFIBuilder {
    pub fn new() -> Self {
        FFIBuilder {
            bindings: Vec::new(),
        }
    }

    pub fn add_binding(&mut self, binding: FFIBinding) {
        self.bindings.push(binding);
    }

    pub fn build(&self) -> FFIResult<FFIRegistry> {
        let mut registry = FFIRegistry::new();
        for binding in &self.bindings {
            let lib = FFILibrary::load(binding.library())?;
            for func in binding.functions() {
                lib.register_function(func.clone());
            }
        }
        Ok(registry)
    }
}

pub struct FFIRegistry {
    libraries: HashMap<String, FFILibrary>,
}

impl FFIRegistry {
    pub fn new() -> Self {
        FFIRegistry {
            libraries: HashMap::new(),
        }
    }

    pub fn register_library(&mut self, name: String, lib: FFILibrary) {
        self.libraries.insert(name, lib);
    }

    pub fn get_library(&self, name: &str) -> Option<&FFILibrary> {
        self.libraries.get(name)
    }

    pub fn call(&self, lib_name: &str, func_name: &str, args: &[FFIValue]) -> FFIResult<FFIValue> {
        let lib = self
            .get_library(lib_name)
            .ok_or_else(|| FFIError::LibraryLoadError(format!("Library '{}' not found", lib_name)))?;
        let func = lib.get_function(func_name)?;
        unsafe { lib.call(&func, args) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_string_helper() {
        let c_str = CStringHelper::to_c_string("hello").unwrap();
        let s = CStringHelper::from_c_string(c_str.as_ptr()).unwrap();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_ffi_value() {
        let val = FFIValue::Int(42);
        assert_eq!(val.as_int().unwrap(), 42);
    }

    #[test]
    fn test_ffi_function() {
        let func = FFIFunction::new(
            "test".to_string(),
            FFICType::Int,
            vec![FFICType::Int, FFICType::Int],
            FFICallConvention::C,
        );
        assert_eq!(func.name(), "test");
    }
}
