// ==================== 数学标准库 ====================
// 基础数学函数、三角函数、常数、随机数等

// ==================== 数学常数 ====================
pub const PI: float = 3.14159265358979323846;
pub const E: float = 2.71828182845904523536;
pub const TAU: float = 6.28318530717958647692;
pub const SQRT2: float = 1.41421356237309504880;
pub const SQRT3: float = 1.73205080756887729353;
pub const LN2: float = 0.69314718055994530942;
pub const LN10: float = 2.30258509299404568402;
pub const LOG2E: float = 1.44269504088896340736;
pub const LOG10E: float = 0.43429448190325182765;
pub const EPSILON: float = 1e-12;

// ==================== 基础运算 ====================
pub fn abs<T>(n: T) -> T where T: Numeric {
    if n < T::zero() { -n } else { n }
}

pub fn min<T>(a: T, b: T) -> T where T: Numeric {
    if a < b { a } else { b }
}

pub fn max<T>(a: T, b: T) -> T where T: Numeric {
    if a > b { a } else { b }
}

pub fn clamp<T>(n: T, min_val: T, max_val: T) -> T where T: Numeric {
    if n < min_val { min_val }
    else if n > max_val { max_val }
    else { n }
}

pub fn sign<T>(n: T) -> int where T: Numeric {
    if n < T::zero() { -1 }
    else if n > T::zero() { 1 }
    else { 0 }
}

pub fn floor<T>(n: T) -> int where T: Numeric {
    __floor(n)
}

pub fn ceil<T>(n: T) -> int where T: Numeric {
    __ceil(n)
}

pub fn round<T>(n: T) -> int where T: Numeric {
    __round(n)
}

pub fn trunc<T>(n: T) -> int where T: Numeric {
    __trunc(n)
}

pub fn fract<T>(n: T) -> float where T: Numeric {
    __fract(n)
}

// ==================== 幂与根 ====================
pub fn sqrt(n: float) -> float {
    __sqrt(n)
}

pub fn cbrt(n: float) -> float {
    __cbrt(n)
}

pub fn pow(base: float, exp: float) -> float {
    __pow(base, exp)
}

pub fn hypot(x: float, y: float) -> float {
    __hypot(x, y)
}

pub fn powi(n: float, exp: int) -> float {
    __powi(n, exp)
}

// ==================== 对数 ====================
pub fn ln(n: float) -> float {
    __log(n)
}

pub fn log2(n: float) -> float {
    __log2(n)
}

pub fn log10(n: float) -> float {
    __log10(n)
}

pub fn log_base(n: float, base: float) -> float {
    __log(n) / __log(base)
}

// ==================== 三角函数 ====================
pub fn sin(n: float) -> float {
    __sin(n)
}

pub fn cos(n: float) -> float {
    __cos(n)
}

pub fn tan(n: float) -> float {
    __tan(n)
}

pub fn asin(n: float) -> float {
    __asin(n)
}

pub fn acos(n: float) -> float {
    __acos(n)
}

pub fn atan(n: float) -> float {
    __atan(n)
}

pub fn atan2(y: float, x: float) -> float {
    __atan2(y, x)
}

pub fn sinh(n: float) -> float {
    __sinh(n)
}

pub fn cosh(n: float) -> float {
    __cosh(n)
}

pub fn tanh(n: float) -> float {
    __tanh(n)
}

pub fn asinh(n: float) -> float {
    __asinh(n)
}

pub fn acosh(n: float) -> float {
    __acosh(n)
}

pub fn atanh(n: float) -> float {
    __atanh(n)
}

// ==================== 角度转换 ====================
pub fn degrees(radians: float) -> float {
    radians * 180.0 / PI
}

pub fn radians(degrees: float) -> float {
    degrees * PI / 180.0
}

// ==================== 整数运算 ====================
pub fn gcd(a: int, b: int) -> int {
    let mut x = a;
    let mut y = b;
    while y != 0 {
        let t = y;
        y = x % y;
        x = t;
    }
    if x < 0 { -x } else { x }
}

pub fn lcm(a: int, b: int) -> int {
    if a == 0 || b == 0 { 0 }
    else { (a / gcd(a, b)) * b }
}

pub fn is_even(n: int) -> bool {
    n % 2 == 0
}

pub fn is_odd(n: int) -> bool {
    n % 2 != 0
}

pub fn factorial(n: int) -> int {
    if n < 0 { 0 }
    else if n <= 1 { 1 }
    else {
        let result = 1;
        for i in 2..=n {
            result = result * i;
        }
        result
    }
}

pub fn fibonacci(n: int) -> int {
    if n <= 0 { 0 }
    else if n == 1 { 1 }
    else {
        let mut a = 0;
        let mut b = 1;
        for _ in 2..=n {
            let c = a + b;
            a = b;
            b = c;
        }
        b
    }
}

pub fn is_prime(n: int) -> bool {
    if n <= 1 { false }
    else if n <= 3 { true }
    else if n % 2 == 0 || n % 3 == 0 { false }
    else {
        let mut i = 5;
        while i * i <= n {
            if n % i == 0 || n % (i + 2) == 0 {
                return false;
            }
            i = i + 6;
        }
        true
    }
}

pub fn divisors(n: int) -> Vec<int> {
    let mut result = Vec::new();
    if n == 0 { return result; }
    for i in 1..=__sqrt(n as float) as int {
        if n % i == 0 {
            result.push(i);
            if i != n / i {
                result.push(n / i);
            }
        }
    }
    result.sort();
    result
}

// ==================== 浮点检查 ====================
pub fn is_nan(n: float) -> bool {
    __is_nan(n)
}

pub fn is_inf(n: float) -> bool {
    __is_inf(n)
}

pub fn is_finite(n: float) -> bool {
    __is_finite(n)
}

// ==================== 随机数 ====================
pub struct Rng {
    state: int,
}

impl Rng {
    pub fn new() -> Rng {
        Rng { state: __time_now() as int }
    }
    
    pub fn with_seed(seed: int) -> Rng {
        Rng { state: seed }
    }
    
    pub fn next(&mut self) -> int {
        self.state = (self.state * 1103515245 + 12345) % 2147483648;
        self.state
    }
    
    pub fn next_range(&mut self, min: int, max: int) -> int {
        min + (self.next() % (max - min + 1))
    }
    
    pub fn next_float(&mut self) -> float {
        self.next() as float / 2147483647.0
    }
    
    pub fn next_range_float(&mut self, min: float, max: float) -> float {
        min + (self.next_float() * (max - min))
    }
    
    pub fn next_bool(&mut self) -> bool {
        self.next() % 2 == 0
    }
    
    pub fn shuffle<T>(&mut self, arr: &mut [T]) {
        let len = array_len(arr);
        for i in 0..len {
            let j = self.next_range(0, len - 1);
            let tmp = arr[i];
            arr[i] = arr[j];
            arr[j] = tmp;
        }
    }
    
    pub fn choose<T>(&mut self, arr: &[T]) -> &T {
        let idx = self.next_range(0, array_len(arr) - 1);
        &arr[idx]
    }
}

// 全局随机数生成器
pub let RNG = Rng::new();

// ==================== 数值 trait ====================
pub trait Numeric: Add + Sub + Mul + Div + Rem + PartialOrd {
    fn zero() -> Self;
    fn one() -> Self;
    fn from_int(n: int) -> Self;
    fn to_int(&self) -> int;
}

impl Numeric for int {
    fn zero() -> int { 0 }
    fn one() -> int { 1 }
    fn from_int(n: int) -> int { n }
    fn to_int(&self) -> int { *self }
}

impl Numeric for float {
    fn zero() -> float { 0.0 }
    fn one() -> float { 1.0 }
    fn from_int(n: int) -> float { n as float }
    fn to_int(&self) -> int { *self as int }
}
