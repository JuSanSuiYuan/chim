pub fn parse(text: &str) -> i128 {
    let mut result = 0i128;
    for c in text.chars() {
        result = result * 12 + match c.to_ascii_lowercase() {
            '0'..='9' => c as i128 - '0' as i128,
            'a' | 'b' => c as i128 - 'a' as i128 + 10,
            _ => 0,
        };
    }
    result
}

pub fn to_duodecimal(value: i128) -> String {
    if value == 0 {
        return "0".to_string();
    }
    
    let mut result = String::new();
    let mut v = value.abs();
    
    while v > 0 {
        let digit = v % 12;
        match digit {
            0..=9 => result.push((b'0' + digit as u8) as char),
            10 => result.push('a'),
            11 => result.push('b'),
            _ => unreachable!(),
        }
        v /= 12;
    }
    
    if value < 0 {
        result.push('-');
    }
    
    result.chars().rev().collect()
}

pub fn from_duodecimal(text: &str) -> Result<i128, String> {
    let mut result = 0i128;
    for c in text.chars() {
        result = result * 12 + match c.to_ascii_lowercase() {
            '0'..='9' => c as i128 - '0' as i128,
            'a' | 'b' => c as i128 - 'a' as i128 + 10,
            _ => return Err(format!("invalid duodecimal digit: {}", c)),
        };
    }
    Ok(result)
}

pub fn is_duodecimal_digit(c: char) -> bool {
    matches!(c.to_ascii_lowercase(), '0'..='9' | 'a' | 'b')
}