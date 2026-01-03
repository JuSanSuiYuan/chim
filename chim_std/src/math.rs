//! 数学函数模块

pub fn abs(x: f64) -> f64 {
    x.abs()
}

pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

pub fn pow(x: f64, y: f64) -> f64 {
    x.powf(y)
}

pub fn sin(x: f64) -> f64 {
    x.sin()
}

pub fn cos(x: f64) -> f64 {
    x.cos()
}

pub fn tan(x: f64) -> f64 {
    x.tan()
}

pub const PI: f64 = std::f64::consts::PI;
pub const E: f64 = std::f64::consts::E;
pub const TAU: f64 = std::f64::consts::TAU;

/// 自然对数
pub fn ln(x: f64) -> f64 {
    x.ln()
}

/// 常用对数
pub fn log10(x: f64) -> f64 {
    x.log10()
}

/// 任意底对数
pub fn log(x: f64, base: f64) -> f64 {
    x.log(base)
}

/// e的x次方
pub fn exp(x: f64) -> f64 {
    x.exp()
}

/// 反正弦
pub fn asin(x: f64) -> f64 {
    x.asin()
}

/// 反余弦
pub fn acos(x: f64) -> f64 {
    x.acos()
}

/// 反正切
pub fn atan(x: f64) -> f64 {
    x.atan()
}

/// 两参数反正切
pub fn atan2(y: f64, x: f64) -> f64 {
    y.atan2(x)
}

/// 双曲正弦
pub fn sinh(x: f64) -> f64 {
    x.sinh()
}

/// 双曲余弦
pub fn cosh(x: f64) -> f64 {
    x.cosh()
}

/// 双曲正切
pub fn tanh(x: f64) -> f64 {
    x.tanh()
}

/// 向上取整
pub fn ceil(x: f64) -> f64 {
    x.ceil()
}

/// 向下取整
pub fn floor(x: f64) -> f64 {
    x.floor()
}

/// 四舍五入
pub fn round(x: f64) -> f64 {
    x.round()
}

/// 截断小数部分
pub fn trunc(x: f64) -> f64 {
    x.trunc()
}

/// 最大值
pub fn max(a: f64, b: f64) -> f64 {
    a.max(b)
}

/// 最小值
pub fn min(a: f64, b: f64) -> f64 {
    a.min(b)
}

/// 符号函数
pub fn signum(x: f64) -> f64 {
    x.signum()
}

/// 将值限制在范围内
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.clamp(min, max)
}

/// 角度转弧度
pub fn to_radians(degrees: f64) -> f64 {
    degrees.to_radians()
}

/// 弧度转角度
pub fn to_degrees(radians: f64) -> f64 {
    radians.to_degrees()
}

/// 阶乘
pub fn factorial(n: u64) -> u64 {
    if n == 0 || n == 1 {
        1
    } else {
        (2..=n).product()
    }
}

/// 最大公约数
pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

/// 最小公倍数
pub fn lcm(a: u64, b: u64) -> u64 {
    if a == 0 || b == 0 {
        0
    } else {
        (a * b) / gcd(a, b)
    }
}
