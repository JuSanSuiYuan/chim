pub fn parse(text: &str) -> i128 {
    let mut result = 0i128;
    for c in text.chars() {
        result = result * 3 + match c {
            '0' => 0,
            '1' => 1,
            '2' => 2,
            _ => 0,
        };
    }
    result
}