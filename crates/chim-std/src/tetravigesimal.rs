pub fn parse(text: &str) -> i128 {
    let mut result = 0i128;
    for c in text.chars() {
        result = result * 24 + match c.to_ascii_lowercase() {
            '0'..='9' => c as i128 - '0' as i128,
            'a'..='n' => c as i128 - 'a' as i128 + 10,
            _ => 0,
        };
    }
    result
}

pub fn to_tetravigesimal(value: i128) -> String {
    if value == 0 {
        return "0".to_string();
    }
    
    let mut result = String::new();
    let mut v = value.abs();
    
    while v > 0 {
        let digit = v % 24;
        match digit {
            0..=9 => result.push((b'0' + digit as u8) as char),
            10..=23 => result.push((b'a' + (digit - 10) as u8) as char),
            _ => unreachable!(),
        }
        v /= 24;
    }
    
    if value < 0 {
        result.push('-');
    }
    
    result.chars().rev().collect()
}

pub fn from_tetravigesimal(text: &str) -> Result<i128, String> {
    let mut result = 0i128;
    for c in text.chars() {
        result = result * 24 + match c.to_ascii_lowercase() {
            '0'..='9' => c as i128 - '0' as i128,
            'a'..='n' => c as i128 - 'a' as i128 + 10,
            _ => return Err(format!("invalid tetravigesimal digit: {}", c)),
        };
    }
    Ok(result)
}

pub fn is_tetravigesimal_digit(c: char) -> bool {
    matches!(c.to_ascii_lowercase(), '0'..='9' | 'a'..='n')
}

pub fn hours_to_tetravigesimal(hours: i32) -> String {
    let h = hours % 24;
    to_tetravigesimal(h as i128)
}

pub fn tetravigesimal_to_hours(text: &str) -> Result<i32, String> {
    Ok(from_tetravigesimal(text)? as i32)
}