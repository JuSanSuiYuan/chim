// ==================== 正则表达式模块 ====================
// 支持常见的正则表达式操作：匹配、查找、替换、分割

pub mod regex {
    use crate::stdlib::prelude::{Option, Result, Vec, String};

    #[derive(Debug, Clone, PartialEq)]
    pub struct Regex {
        pattern: string,
        compiled: CompiledRegex,
    }

    #[derive(Debug, Clone)]
    struct CompiledRegex {
        pattern: string,
        flags: RegexFlags,
    }

    #[derive(Debug, Clone, Default)]
    struct RegexFlags {
        ignore_case: bool,
        multi_line: bool,
        dot_matches_newline: bool,
    }

    #[derive(Debug, Clone)]
    pub struct Match {
        start: int,
        end: int,
        matched: string,
        groups: Vec<Option<string>>,
    }

    #[derive(Debug, Clone)]
    pub struct Error {
        message: string,
    }

    impl Error {
        pub fn new(msg: string) -> Error {
            Error { message: msg }
        }
    }

    pub type Result<T> = std::result::Result<T, Error>;

    impl Regex {
        pub fn new(pattern: string) -> Result<Regex> {
            if pattern.is_empty() {
                return Err(Error::new("empty pattern".to_string()));
            }
            Ok(Regex {
                pattern: pattern.clone(),
                compiled: CompiledRegex {
                    pattern,
                    flags: RegexFlags::default(),
                },
            })
        }

        pub fn is_match(&self, text: &string) -> bool {
            self.find(text).is_some()
        }

        pub fn find(&self, text: &string) -> Option<Match> {
            self.find_at(text, 0)
        }

        pub fn find_at(&self, text: &string, start: int) -> Option<Match> {
            let pattern = &self.compiled.pattern;
            let search_result = simple_search(pattern, text, start);
            search_result
        }

        pub fn find_all(&self, text: &string) -> Vec<Match> {
            let mut matches = Vec::new();
            let mut pos = 0;
            while pos < string_len(text) {
                match self.find_at(text, pos) {
                    Some(m) => {
                        matches.push(m.clone());
                        pos = m.end + 1;
                        if m.matched.is_empty() {
                            pos = pos + 1;
                        }
                    }
                    None => break,
                }
            }
            matches
        }

        pub fn replace(&self, text: &string, replacement: &string) -> string {
            self.replace_n(text, replacement, -1)
        }

        pub fn replace_n(&self, text: &string, replacement: &string, n: int) -> string {
            let mut result = String::new();
            let mut pos = 0;
            let mut count = 0;
            while (n < 0 || count < n) && pos < string_len(text) {
                match self.find_at(text, pos) {
                    Some(m) => {
                        result.push_str(&text[pos..m.start]);
                        result.push_str(replacement);
                        pos = m.end;
                        count = count + 1;
                        if m.matched.is_empty() && pos < string_len(text) {
                            result.push_str(&text[pos..pos+1]);
                            pos = pos + 1;
                        }
                    }
                    None => {
                        result.push_str(&text[pos..]);
                        break;
                    }
                }
            }
            if pos < string_len(text) {
                result.push_str(&text[pos..]);
            }
            result
        }

        pub fn split(&self, text: &string) -> Vec<string> {
            self.splitn(text, -1)
        }

        pub fn splitn(&self, text: &string, n: int) -> Vec<string> {
            let mut parts = Vec::new();
            let mut pos = 0;
            let mut count = 0;
            while (n < 0 || count < n - 1) && pos < string_len(text) {
                match self.find_at(text, pos) {
                    Some(m) => {
                        parts.push(text[pos..m.start].to_string());
                        pos = m.end;
                        count = count + 1;
                        if m.matched.is_empty() && pos < string_len(text) {
                            pos = pos + 1;
                        }
                    }
                    None => break,
                }
            }
            parts.push(text[pos..].to_string());
            parts
        }

        pub fn captures(&self, text: &string) -> Option<Vec<Option<string>>> {
            self.find(text).map(|m| m.groups)
        }

        pub fn text(&self) -> string {
            self.pattern.clone()
        }
    }

    fn simple_search(pattern: &string, text: &string, start: int) -> Option<Match> {
        let p = pattern.as_bytes();
        let t = text.as_bytes();
        let p_len = p.len();
        let t_len = t.len();
        
        if p_len == 0 {
            return Some(Match {
                start: start,
                end: start,
                matched: "".to_string(),
                groups: Vec::new(),
            });
        }
        
        let mut i = start;
        while i <= t_len - p_len {
            let mut j = 0;
            let mut matched = true;
            while j < p_len {
                let pc = p[j] as char;
                let tc = t[i + j] as char;
                
                if pc == '%' && j + 1 < p_len {
                    let next = p[j + 1] as char;
                    match next {
                        'd' if tc >= '0' && tc <= '9' => { j = j + 2; continue; }
                        'a' => { j = j + 2; continue; }
                        's' => { j = j + 2; continue; }
                        _ => {}
                    }
                }
                
                if tc != pc {
                    matched = false;
                    break;
                }
                j = j + 1;
            }
            
            if matched {
                let matched_str = text[i..i + p_len].to_string();
                return Some(Match {
                    start: i,
                    end: i + p_len - 1,
                    matched: matched_str,
                    groups: Vec::new(),
                });
            }
            
            i = i + 1;
        }
        
        None
    }

    pub fn is_match(pattern: &string, text: &string) -> bool {
        match Regex::new(pattern.clone()) {
            Ok(re) => re.is_match(text),
            Err(_) => false,
        }
    }

    pub fn find(pattern: &string, text: &string) -> Option<Match> {
        match Regex::new(pattern.clone()) {
            Ok(re) => re.find(text),
            Err(_) => None,
        }
    }

    pub fn replace(pattern: &string, text: &string, replacement: &string) -> string {
        match Regex::new(pattern.clone()) {
            Ok(re) => re.replace(text, replacement),
            Err(_) => text.clone(),
        }
    }

    pub fn split(pattern: &string, text: &string) -> Vec<string> {
        match Regex::new(pattern.clone()) {
            Ok(re) => re.split(text),
            Err(_) => vec![text.clone()],
        }
    }
}

// ==================== 常用正则表达式 ====================

pub mod regex_captures {
    use super::regex::Regex;

    pub fn extract_email(text: &string) -> Option<string> {
        let re = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
        re.find(text).map(|m| m.matched)
    }

    pub fn extract_phone(text: &string) -> Option<string> {
        let re = Regex::new(r"\+?[0-9]{10,15}").unwrap();
        re.find(text).map(|m| m.matched)
    }

    pub fn extract_url(text: &string) -> Option<string> {
        let re = Regex::new(r"https?://[^\s]+").unwrap();
        re.find(text).map(|m| m.matched)
    }

    pub fn extract_ip(text: &string) -> Option<string> {
        let re = Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();
        re.find(text).map(|m| m.matched)
    }

    pub fn is_valid_email(email: &string) -> bool {
        let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        re.is_match(email)
    }

    pub fn is_valid_url(url: &string) -> bool {
        let re = Regex::new(r"^https?://[^\s]+$").unwrap();
        re.is_match(url)
    }

    pub fn is_valid_ip(ip: &string) -> bool {
        let re = Regex::new(r"^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$").unwrap();
        if !re.is_match(ip) {
            return false;
        }
        true
    }
}
