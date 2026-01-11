// ==================== 类型转换标准库 ====================
// 各种类型之间的转换函数

// ==================== 基本类型转换 ====================
// int -> 其他类型
pub fn int_to_string(n: int) -> string {
    __int_to_string(n)
}

pub fn int_to_float(n: int) -> float {
    n as float
}

pub fn int_to_bool(n: int) -> bool {
    n != 0
}

// float -> 其他类型
pub fn float_to_string(f: float) -> string {
    __float_to_string(f)
}

pub fn float_to_int(f: float) -> int {
    f as int
}

// bool -> 其他类型
pub fn bool_to_string(b: bool) -> string {
    if b { "true" } else { "false" }
}

pub fn bool_to_int(b: bool) -> int {
    if b { 1 } else { 0 }
}

// string -> 其他类型
pub fn string_to_int(s: string) -> int {
    __string_to_int(s)
}

pub fn string_to_float(s: string) -> float {
    __string_to_float(s)
}

pub fn string_to_bool(s: string) -> bool {
    let lower = s.to_lowercase();
    lower == "true" || lower == "yes" || lower == "1"
}

// ==================== 格式化转换 ====================
// 使用格式说明符进行转换
pub fn format<T>(fmt: string, args: &[T]) -> string {
    __format(fmt, args)
}

// 整数进制转换
pub fn int_to_hex(n: int) -> string {
    __int_to_base(n, 16)
}

pub fn int_to_octal(n: int) -> string {
    __int_to_base(n, 8)
}

pub fn int_to_binary(n: int) -> string {
    __int_to_base(n, 2)
}

pub fn hex_to_int(s: string) -> int {
    __base_to_int(s, 16)
}

pub fn octal_to_int(s: string) -> int {
    __base_to_int(s, 8)
}

pub fn binary_to_int(s: string) -> int {
    __base_to_int(s, 2)
}

// ==================== 浮点精度转换 ====================
// 浮点数精度控制
pub fn round_to(f: float, decimals: int) -> float {
    __round_float(f, decimals)
}

pub fn truncate_to(f: float, decimals: int) -> float {
    __truncate_float(f, decimals)
}

// ==================== 字符转换 ====================
pub fn char_to_int(c: string) -> int {
    __char_code(c)
}

pub fn int_to_char(n: int) -> string {
    __code_char(n)
}

pub fn char_to_string(c: string) -> string {
    c
}

pub fn string_to_chars(s: string) -> &[string] {
    __string_to_chars(s)
}

pub fn chars_to_string(chars: &[string]) -> string {
    __chars_to_string(chars)
}

// ==================== 大小端转换 ====================
pub fn htons(n: int) -> int {
    __byte_swap16(n)
}

pub fn htonl(n: int) -> int {
    __byte_swap32(n)
}

pub fn htonll(n: int) -> int {
    __byte_swap64(n)
}

pub fn ntohs(n: int) -> int {
    __byte_swap16(n)
}

pub fn ntohl(n: int) -> int {
    __byte_swap32(n)
}

pub fn ntohll(n: int) -> int {
    __byte_swap64(n)
}

// ==================== Base64 编解码 ====================
pub fn base64_encode(data: string) -> string {
    __base64_encode(data)
}

pub fn base64_decode(s: string) -> string {
    __base64_decode(s)
}

pub fn base64_encode_bytes(data: &[byte]) -> string {
    __base64_encode_bytes(data)
}

pub fn base64_decode_to_bytes(s: string) -> &[byte] {
    __base64_decode_to_bytes(s)
}

// ==================== URL 编解码 ====================
pub fn url_encode(s: string) -> string {
    __url_encode(s)
}

pub fn url_decode(s: string) -> string {
    __url_decode(s)
}

pub fn url_encode_component(s: string) -> string {
    __url_encode_component(s)
}

pub fn url_decode_component(s: string) -> string {
    __url_decode_component(s)
}

// ==================== HTML 转义 ====================
pub fn html_escape(s: string) -> string {
    let result = s;
    result = result.replace("&", "&amp;");
    result = result.replace("<", "&lt;");
    result = result.replace(">", "&gt;");
    result = result.replace("\"", "&quot;");
    result = result.replace("'", "&#39;");
    result
}

pub fn html_unescape(s: string) -> string {
    let result = s;
    result = result.replace("&amp;", "&");
    result = result.replace("&lt;", "<");
    result = result.replace("&gt;", ">");
    result = result.replace("&quot;", "\"");
    result = result.replace("&#39;", "'");
    result
}

// ==================== JSON 转换 ====================
pub fn json_escape(s: string) -> string {
    let result = s;
    result = result.replace("\\", "\\\\");
    result = result.replace("\"", "\\\"");
    result = result.replace("\n", "\\n");
    result = result.replace("\r", "\\r");
    result = result.replace("\t", "\\t");
    result
}

pub fn json_unescape(s: string) -> string {
    let result = s;
    result = result.replace("\\\\", "\u{FFFF}");
    result = result.replace("\\\"", "\"");
    result = result.replace("\\n", "\n");
    result = result.replace("\\r", "\r");
    result = result.replace("\\t", "\t");
    result = result.replace("\u{FFFF}", "\\");
    result
}

// ==================== UUID ====================
pub fn uuid_v4() -> string {
    __uuid_v4()
}

pub fn uuid_v5(ns: string, name: string) -> string {
    __uuid_v5(ns, name)
}

pub fn uuid_parse(s: string) -> bool {
    string_len(s) == 36 &&
    s.char_at(8) == "-" &&
    s.char_at(13) == "-" &&
    s.char_at(18) == "-" &&
    s.char_at(23) == "-"
}

// ==================== 哈希转换 ====================
// 字符串哈希
pub fn hash_crc32(s: string) -> int {
    __hash_crc32(s)
}

pub fn hash_md5(s: string) -> string {
    __hash_md5(s)
}

pub fn hash_sha1(s: string) -> string {
    __hash_sha1(s)
}

pub fn hash_sha256(s: string) -> string {
    __hash_sha256(s)
}

// ==================== 智能转换 trait ====================
pub trait From<T> {
    fn from(value: T) -> Self;
}

pub trait Into<T> {
    fn into(self) -> T;
}

impl From<int> for float {
    fn from(value: int) -> float {
        value as float
    }
}

impl From<float> for int {
    fn from(value: float) -> int {
        value as int
    }
}

impl From<int> for string {
    fn from(value: int) -> string {
        int_to_string(value)
    }
}

impl From<float> for string {
    fn from(value: float) -> string {
        float_to_string(value)
    }
}

impl From<bool> for string {
    fn from(value: bool) -> string {
        bool_to_string(value)
    }
}

impl<T> Into<T> for T where T: From<T> {
    fn into(self) -> T {
        T::from(self)
    }
}

// ==================== As trait ====================
pub trait AsRef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}

pub trait AsMut<T: ?Sized> {
    fn as_mut(&mut self) -> &mut T;
}

impl AsRef<str> for string {
    fn as_ref(&self) -> &str {
        self
    }
}

impl AsMut<str> for string {
    fn as_mut(&mut self) -> &mut str {
        self
    }
}

// ==================== Borrow trait ====================
pub trait Borrow<T: ?Sized> {
    fn borrow(&self) -> &T;
}

impl Borrow<str> for string {
    fn borrow(&self) -> &str {
        self
    }
}
