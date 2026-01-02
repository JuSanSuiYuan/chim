use colored::Colorize;

/// 打印成功信息
pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

/// 打印错误信息
pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg);
}

/// 打印警告信息
pub fn warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg);
}

/// 打印信息
pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}

/// 解析包名和版本
/// 支持格式: "name@version" 或 "name"
pub fn parse_package_spec(spec: &str) -> (String, Option<String>) {
    if let Some(pos) = spec.find('@') {
        let name = spec[..pos].to_string();
        let version = spec[pos + 1..].to_string();
        (name, Some(version))
    } else {
        (spec.to_string(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_spec() {
        assert_eq!(
            parse_package_spec("lodash@4.17.21"),
            ("lodash".to_string(), Some("4.17.21".to_string()))
        );
        
        assert_eq!(
            parse_package_spec("express"),
            ("express".to_string(), None)
        );
    }
}
