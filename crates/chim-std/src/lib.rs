use chim_ast::{Expr, Literal, LiteralKind};
use chim_span::Span;

pub mod ternary;
pub mod balanced;
pub mod duodecimal;
pub mod tetravigesimal;
pub mod sexagesimal;

pub fn parse_ternary_literal(text: &str) -> i128 {
    ternary::parse(text)
}

pub fn parse_balanced_ternary(text: &str) -> i128 {
    balanced::parse(text)
}

pub fn parse_duodecimal(text: &str) -> i128 {
    duodecimal::parse(text)
}

pub fn parse_tetravigesimal(text: &str) -> i128 {
    tetravigesimal::parse(text)
}

pub fn parse_sexagesimal(text: &str) -> i128 {
    sexagesimal::parse(text)
}

pub fn to_ternary(value: i128) -> String {
    if value == 0 {
        return "0".to_string();
    }
    
    let mut result = String::new();
    let mut v = value.abs();
    
    while v > 0 {
        let digit = v % 3;
        match digit {
            0 => result.push('0'),
            1 => result.push('1'),
            2 => result.push('2'),
            _ => unreachable!(),
        }
        v /= 3;
    }
    
    if value < 0 {
        result.push('-');
    }
    
    result.chars().rev().collect()
}

pub fn to_balanced_ternary(value: i128) -> String {
    if value == 0 {
        return "0".to_string();
    }
    
    let mut result = String::new();
    let mut v = value;
    
    while v != 0 {
        let digit = v % 3;
        match digit {
            0 => {
                result.push('0');
                v = v / 3;
            }
            1 => {
                result.push('1');
                v = (v - 1) / 3;
            }
            2 => {
                result.push('-');
                v = (v + 1) / 3;
            }
            _ => unreachable!(),
        }
    }
    
    result.chars().rev().collect()
}

pub fn from_ternary(text: &str) -> Result<i128, String> {
    let mut result = 0i128;
    for c in text.chars() {
        result = result * 3 + match c {
            '0' => 0,
            '1' => 1,
            '2' => 2,
            _ => return Err(format!("invalid ternary digit: {}", c)),
        };
    }
    Ok(result)
}

pub fn from_balanced_ternary(text: &str) -> Result<i128, String> {
    let mut result = 0i128;
    for c in text.chars() {
        result = result * 3 + match c {
            '-' => -1,
            '0' => 0,
            '1' => 1,
            _ => return Err(format!("invalid balanced ternary digit: {}", c)),
        };
    }
    Ok(result)
}

pub fn is_ternary_digit(c: char) -> bool {
    matches!(c, '0' | '1' | '2')
}

pub fn is_balanced_ternary_digit(c: char) -> bool {
    matches!(c, '-' | '0' | '1')
}

pub fn is_duodecimal_digit(c: char) -> bool {
    duodecimal::is_duodecimal_digit(c)
}

pub fn is_tetravigesimal_digit(c: char) -> bool {
    tetravigesimal::is_tetravigesimal_digit(c)
}

pub fn is_sexagesimal_digit(c: char) -> bool {
    sexagesimal::is_sexagesimal_digit(c)
}