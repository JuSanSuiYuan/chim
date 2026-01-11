pub fn parse(text: &str) -> i128 {
    let mut result = 0i128;
    for c in text.chars() {
        result = result * 60 + match c.to_ascii_lowercase() {
            '0'..='9' => c as i128 - '0' as i128,
            'a'..='z' => c as i128 - 'a' as i128 + 10,
            _ => 0,
        };
    }
    result
}

pub fn to_sexagesimal(value: i128) -> String {
    if value == 0 {
        return "0".to_string();
    }
    
    let mut result = String::new();
    let mut v = value.abs();
    
    while v > 0 {
        let digit = v % 60;
        match digit {
            0..=9 => result.push((b'0' + digit as u8) as char),
            10..=35 => result.push((b'a' + (digit - 10) as u8) as char),
            36..=59 => result.push((b'A' + (digit - 36) as u8) as char),
            _ => unreachable!(),
        }
        v /= 60;
    }
    
    if value < 0 {
        result.push('-');
    }
    
    result.chars().rev().collect()
}

pub fn from_sexagesimal(text: &str) -> Result<i128, String> {
    let mut result = 0i128;
    for c in text.chars() {
        result = result * 60 + match c.to_ascii_lowercase() {
            '0'..='9' => c as i128 - '0' as i128,
            'a'..='z' => c as i128 - 'a' as i128 + 10,
            _ => return Err(format!("invalid sexagesimal digit: {}", c)),
        };
    }
    Ok(result)
}

pub fn is_sexagesimal_digit(c: char) -> bool {
    matches!(c.to_ascii_lowercase(), '0'..='9' | 'a'..='z')
}

pub fn minutes_to_sexagesimal(minutes: i32) -> String {
    let m = minutes % 60;
    to_sexagesimal(m as i128)
}

pub fn seconds_to_sexagesimal(seconds: i32) -> String {
    let s = seconds % 60;
    to_sexagesimal(s as i128)
}

pub fn time_to_sexagesimal(hours: i32, minutes: i32, seconds: i32) -> String {
    format!("{}:{}:{}", 
        to_sexagesimal(hours as i128),
        to_sexagesimal(minutes as i128),
        to_sexagesimal(seconds as i128)
    )
}

pub fn degrees_to_sexagesimal(degrees: f64) -> String {
    let d = degrees.floor() as i32;
    let remainder = (degrees - d as f64) * 60.0;
    let m = remainder.floor() as i32;
    let s = (remainder - m as f64) * 60.0;
    
    format!("{}°{}'{}\"", 
        to_sexagesimal(d as i128),
        to_sexagesimal(m as i128),
        format!("{:.2}", s)
    )
}