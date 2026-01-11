pub fn parse(text: &str) -> i128 {
    let mut result = 0i128;
    for c in text.chars() {
        result = result * 3 + match c {
            '-' => -1,
            '0' => 0,
            '1' => 1,
            _ => 0,
        };
    }
    result
}

pub fn is_balanced(text: &str) -> bool {
    let mut balance = 0i32;
    for c in text.chars() {
        match c {
            '-' => balance -= 1,
            '1' => balance += 1,
            '0' => {}
            _ => return false,
        }
    }
    balance == 0
}

pub fn to_decimal(text: &str) -> i128 {
    parse(text)
}

pub fn from_decimal(value: i128) -> String {
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