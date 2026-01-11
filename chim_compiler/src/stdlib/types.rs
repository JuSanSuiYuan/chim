// ==================== 类型标准库 ====================
// 类型转换、类型检查、反射等功能

// 获取类型名称
pub fn type_of<T>(value: &T) -> string {
    __type_name::<T>()
}

// 检查类型
pub fn is_int<T>(_: &T) -> bool {
    __type_is_int::<T>()
}

pub fn is_float<T>(_: &T) -> bool {
    __type_is_float::<T>()
}

pub fn is_bool<T>(_: &T) -> bool {
    __type_is_bool::<T>()
}

pub fn is_string<T>(_: &T) -> bool {
    __type_is_string::<T>()
}

pub fn is_array<T>(_: &T) -> bool {
    __type_is_array::<T>()
}

// 类型转换
pub fn to_int<T>(value: &T) -> int {
    __to_int(value)
}

pub fn to_float<T>(value: &T) -> float {
    __to_float(value)
}

pub fn to_bool<T>(value: &T) -> bool {
    __to_bool(value)
}

pub fn to_string<T>(value: &T) -> string {
    __to_string(value)
}

// 安全转换（返回 Option）
pub fn as_int<T>(value: &T) -> Option<int> {
    if is_int(value) { Option::Some(to_int(value)) }
    else { Option::None }
}

pub fn as_float<T>(value: &T) -> Option<float> {
    if is_float(value) { Option::Some(to_float(value)) }
    else { Option::None }
}

// 强制类型转换（不安全）
pub fn unsafe_cast<T, U>(value: &T) -> &U {
    __unsafe_cast(value)
}
