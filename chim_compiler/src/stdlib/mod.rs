// ==================== Chim 标准库 ====================
// 基础类型、字符串、集合、I/O、数学、格式化、文件系统、正则、网络、序列化、泛型、模块、生态系统、CTFE 等标准功能

pub mod prelude;
pub mod types;
pub mod string;
pub mod collections;
pub mod io;
pub mod math;
pub mod time;
pub mod error;
pub mod conv;
pub mod fmt;
pub mod path;
pub mod fs;
pub mod regex;
pub mod net;
pub mod serde;
pub mod generics;
pub mod module;
pub mod ecosystem;
pub mod ctfe;
pub mod const_fn;
pub mod pattern;
pub mod termination;
pub mod sized;
pub mod effect;
pub mod content_addressable;

// ==================== 预导入模块 ====================
// 编译器自动导入这些模块

pub fn load_prelude() -> &'static str {
    r#"
        // Chim 标准库预导入
        // 内置类型方法
        
        // Int 方法
        fn int_abs(x: int) -> int = if x < 0 { -x } else { x };
        fn int_max(a: int, b: int) -> int = if a > b { a } else { b };
        fn int_min(a: int, b: int) -> int = if a < b { a } else { b };
        fn int_clamp(val: int, min: int, max: int) -> int = 
            if val < min { min } else if val > max { max } else { val };
        
        // Float 方法  
        fn float_abs(x: float) -> float = if x < 0.0 { -x } else { x };
        fn float_max(a: float, b: float) -> float = if a > b { a } else { b };
        fn float_min(a: float, b: float) -> float = if a < b { a } else { b };
        fn float_clamp(val: float, min: float, max: float) -> float =
            if val < min { min } else if val > max { max } else { val };
        
        // Bool 方法
        fn bool_not(x: bool) -> bool = if x { false } else { true };
        
        // 基础 I/O
        fn print(msg: string) = __print_str(msg);
        fn println(msg: string) = __print_str(msg + "\n");
        fn print_int(x: int) = __print_int(x);
        fn print_float(x: float) = __print_float(x);
        fn read_line() -> string = __read_line();
        fn read_int() -> int = __read_int();
        
        // 字符串操作
        fn string_len(s: string) -> int = __string_len(s);
        fn string_concat(a: string, b: string) -> string = __string_concat(a, b);
        fn string_substr(s: string, start: int, len: int) -> string = __string_substr(s, start, len);
        
        // 数组操作
        fn array_len(a: &[T]) -> int = __array_len(a);
        fn array_push(a: &mut [T], value: T) = __array_push(a, value);
        fn array_pop(a: &mut [T]) -> T = __array_pop(a);
        
        // 数学常量
        let PI: float = 3.14159265358979323846;
        let E: float = 2.71828182845904523536;
        let TAU: float = 6.28318530717958647692;
        
        // 格式化输出
        fn format_int(n: int) -> string = __format_int(n);
        fn format_float(f: float) -> string = __format_float(f);
        
        // 路径操作
        fn path_new(p: string) -> path = path::Path::new(p);
        fn path_file_name(p: &path) -> option[string] = p.file_name();
        fn path_parent(p: &path) -> option[path] = p.parent();
        fn path_extension(p: &path) -> option[string] = p.extension();
        fn path_is_absolute(p: &path) -> bool = p.is_absolute();
        fn path_join(a: &path, b: &path) -> path = a.join(b);
        
        // 文件系统操作
        fn fs_read_to_string(p: &path) -> result[string, string] = fs::read_to_string(p);
        fn fs_write_string(p: &path, content: string) -> result[(), string] = fs::write_string(p, content);
        fn fs_exists(p: &path) -> bool = fs::exists(p);
        fn fs_is_file(p: &path) -> bool = fs::is_file(p);
        fn fs_is_dir(p: &path) -> bool = fs::is_dir(p);
        
        // 正则表达式操作
        fn regex_new(pattern: string) -> result[regex::Regex, string] = regex::Regex::new(pattern);
        fn regex_is_match(re: &regex::Regex, text: &string) -> bool = re.is_match(text);
        fn regex_find(re: &regex::Regex, text: &string) -> option[regex::Match] = re.find(text);
        fn regex_replace(re: &regex::Regex, text: &string, replacement: &string) -> string = re.replace(text, replacement);
        
        // 网络操作
        fn net_tcp_connect(host: &string, port: u16) -> result[net::TcpStream, net::Error] = net::TcpStream::connect(host, port);
        fn net_tcp_listener_bind(addr: &net::SocketAddr) -> result[net::TcpListener, net::Error] = net::TcpListener::bind(addr);
        fn net_udp_socket_bind(addr: &net::SocketAddr) -> result[net::UdpSocket, net::Error] = net::UdpSocket::bind(addr);
        fn net_http_get(url: &string) -> result[net::http::Response, net::Error] = net::http::get(url);
        
        // 序列化操作
        fn serde_json_to_string(value: &serde::Value) -> string = serde::json::to_string(value);
        fn serde_json_from_string(s: &string) -> result[serde::Value, serde::json::Error] = serde::json::from_string(s);
        fn serde_value_null() -> serde::Value = serde::Value::null();
        fn serde_value_bool(b: bool) -> serde::Value = serde::Value::bool(b);
        fn serde_value_int(n: int) -> serde::Value = serde::Value::int(n);
        fn serde_value_string(s: string) -> serde::Value = serde::Value::string(s);
        
        // 泛型约束操作
        fn generics_new_param(name: string) -> generics::GenericParam = generics::GenericParam::new(name);
        fn generics_add_bound(param: &mut generics::GenericParam, bound: generics::Constraint) = param.add_constraint(bound);
        
        // 模块操作
        fn module_new(name: string, path: string) -> module::Module = module::Module::new(name, path);
        fn module_add_item(m: &mut module::Module, item: module::ModuleItem) = m.add_item(item);
        fn module_resolve(m: &mut module::Module, path: &string) -> result[module::Module, module::Error> = m.resolve_module(path);
        
        // 生态系统操作
        fn ecosystem_new_doc_generator(output_dir: string) -> ecosystem::DocGenerator = ecosystem::DocGenerator::new(output_dir);
        fn ecosystem_new_test_runner() -> ecosystem::TestRunner = ecosystem::TestRunner::new();
        fn ecosystem_new_profiler() -> ecosystem::Profiler = ecosystem::Profiler::new();
        fn ecosystem_new_formatter() -> ecosystem::Formatter = ecosystem::Formatter::new();
        
        // ==================== CTFE 编译期求值 ====================
        
        // CTFE 辅助函数
        fn ctfe_eval(expr: &expression) -> result[ctfe::Value, ctfe::Error] = ctfe::evaluate(expr);
        fn ctfe_is_const(expr: &expression) -> bool = ctfe::is_const_expr(expr);
        
        // CTFE 数学函数
        fn ctfe_abs(x: int) -> int = if x < 0 { -x } else { x };
        fn ctfe_min(a: int, b: int) -> int = if a < b { a } else { b };
        fn ctfe_max(a: int, b: int) -> int = if a > b { a } else { b };
        fn ctfe_clamp(val: int, min: int, max: int) -> int =
            if val < min { min } else if val > max { max } else { val };
        
        fn ctfe_sqrt(x: float) -> float = x ** 0.5;
        fn ctfe_pow(x: float, y: float) -> float = x ** y;
        fn ctfe_floor(x: float) -> float = if x < 0.0 { -(-x) ** 1.0 ** 0.5 } else { x };
        
        // CTFE 字符串函数
        fn ctfe_len(s: string) -> int = s.len();
        fn ctfe_concat(a: string, b: string) -> string = a + b;
        fn ctfe_uppercase(s: string) -> string = s.to_uppercase();
        fn ctfe_lowercase(s: string) -> string = s.to_lowercase();
        
        // CTFE 数组函数
        fn ctfe_array_len(arr: &[T]) -> int = arr.len();
        
        // CTFE 类型操作
        fn ctfe_size_of(T: type) -> int = sizeof(T);
        fn ctfe_type_name(T: type) -> string = type_name(T);
        
        // 数学常量（可在 CTFE 中使用）
        let CTFE_PI: float = 3.14159265358979323846;
        let CTFE_E: float = 2.71828182845904523536;
        let CTFE_TAU: float = 6.28318530717958647692;
    "#
}
