// ==================== 格式化模块 ====================
// 核心功能：格式化字符串、打印宏支持

pub mod fmt {
    use crate::stdlib::prelude::{Option, Result};

    #[derive(Debug, Clone, PartialEq)]
    pub enum FormatSpec {
        Default,
        Binary,
        Octal,
        HexLower,
        HexUpper,
        Decimal,
        Float,
        Scientific,
        General,
        String,
        Char,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Alignment {
        Left,
        Right,
        Center,
    }

    pub struct Formatter {
        width: Option<int>,
        precision: Option<int>,
        spec: FormatSpec,
        sign: bool,
        alternate: bool,
        padding: char,
        alignment: Alignment,
        output: String,
    }

    impl Formatter {
        pub fn new() -> Formatter {
            Formatter {
                width: None,
                precision: None,
                spec: FormatSpec::Default,
                sign: false,
                alternate: false,
                padding: ' ',
                alignment: Alignment::Right,
                output: String::new(),
            }
        }

        pub fn width(&mut self, w: int) -> &mut Formatter {
            self.width = Some(w);
            self
        }

        pub fn precision(&mut self, p: int) -> &mut Formatter {
            self.precision = Some(p);
            self
        }

        pub fn spec(&mut self, s: FormatSpec) -> &mut Formatter {
            self.spec = s;
            self
        }

        pub fn fill(&mut self, c: char) -> &mut Formatter {
            self.padding = c;
            self
        }

        pub fn alignment(&mut self, a: Alignment) -> &mut Formatter {
            self.alignment = a;
            self
        }

        pub fn sign(&mut self, b: bool) -> &mut Formatter {
            self.sign = b;
            self
        }

        pub fn alternate(&mut self, b: bool) -> &mut Formatter {
            self.alternate = b;
            self
        }

        pub fn write_string(&mut self, s: string) -> Result<(), Error> {
            self.output.push_str(&s);
            Ok(())
        }

        pub fn write_char(&mut self, c: char) -> Result<(), Error> {
            self.output.push(c);
            Ok(())
        }

        pub fn write_int(&mut self, n: int) -> Result<(), Error> {
            let s = format_int_to_string(n, &self.spec, self.alternate);
            self.pad_and_write(&s)
        }

        pub fn write_float(&mut self, n: float) -> Result<(), Error> {
            let s = format_float_to_string(n, &self.spec, self.precision);
            self.pad_and_write(&s)
        }

        fn pad_and_write(&mut self, s: &string) -> Result<(), Error> {
            match self.width {
                Some(w) if (s.len() as int) < w => {
                    let pad_len = w - (s.len() as int);
                    match self.alignment {
                        Alignment::Left => {
                            self.output.push_str(s);
                            self.output.push_str(&" ".repeat(pad_len as int));
                        }
                        Alignment::Center => {
                            let left = pad_len / 2;
                            let right = pad_len - left;
                            self.output.push_str(&" ".repeat(left as int));
                            self.output.push_str(s);
                            self.output.push_str(&" ".repeat(right as int));
                        }
                        Alignment::Right => {
                            self.output.push_str(&" ".repeat(pad_len as int));
                            self.output.push_str(s);
                        }
                    }
                    Ok(())
                }
                _ => {
                    self.output.push_str(s);
                    Ok(())
                }
            }
        }

        pub fn finish(&mut self) -> string {
            self.output.clone()
        }
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

    pub trait Display {
        fn fmt(&self, f: &mut Formatter) -> Result<(), Error>;
    }

    pub trait Debug {
        fn fmt(&self, f: &mut Formatter) -> Result<(), Error>;
    }

    fn format_int_to_string(n: int, spec: &FormatSpec, alternate: bool) -> string {
        if n == 0 {
            return "0".to_string();
        }
        
        let mut num = n;
        let negative = num < 0;
        if negative {
            num = -num;
        }
        
        let mut chars = Vec::new();
        while num > 0 {
            let d = (num % 10) as int;
            chars.push(int_to_char(d));
            num = num / 10;
        }
        
        chars.reverse();
        let mut result = chars.into_iter().collect::<string>();
        
        if negative {
            result = "-" + &result;
        } else if spec == &FormatSpec::Decimal && spec != &FormatSpec::Default {
            result = "+" + &result;
        }
        
        match spec {
            FormatSpec::Binary => {
                if alternate { "0b" + &result } else { result }
            }
            FormatSpec::Octal => {
                if alternate { "0o" + &result } else { result }
            }
            FormatSpec::HexLower => {
                if alternate { "0x" + &result } else { result }
            }
            FormatSpec::HexUpper => {
                let upper = result.chars().map(|c| {
                    if c >= 'a' && c <= 'f' {
                        (c as u8 - 32) as char
                    } else {
                        c
                    }
                }).collect();
                if alternate { "0X" + &upper } else { upper }
            }
            _ => result,
        }
    }

    fn int_to_char(d: int) -> char {
        if d < 10 {
            (b'0' + (d as u8)) as char
        } else {
            (b'a' + (d as u8 - 10)) as char
        }
    }

    fn format_float_to_string(n: float, spec: &FormatSpec, precision: Option<int>) -> string {
        let p = precision.unwrap_or(6);
        
        if n.is_nan() {
            return "NaN".to_string();
        }
        if n.is_infinite() {
            return if n > 0.0 { "inf".to_string() } else { "-inf".to_string() };
        }
        
        let negative = n < 0.0;
        let mut num = n.abs();
        
        match spec {
            FormatSpec::Scientific | FormatSpec::General => {
                format_float_scientific(num, p)
            }
            FormatSpec::Float | FormatSpec::Default => {
                format_float_fixed(num, p, negative)
            }
            _ => format_float_fixed(num, p, negative),
        }
    }

    fn format_float_fixed(n: float, precision: int, negative: bool) -> string {
        let int_part = (n as int);
        let frac_part = n - (int_part as float);
        
        let mut result = int_to_string(int_part);
        
        if precision > 0 {
            result.push('.');
            let mut temp = frac_part;
            for _ in 0..precision {
                temp = temp * 10.0;
                let digit = (temp as int);
                result.push(int_to_char(digit));
                temp = temp - (digit as float);
            }
        }
        
        if negative {
            result = "-" + &result;
        }
        
        result
    }

    fn format_float_scientific(n: float, precision: int) -> string {
        if n == 0.0 {
            return "0e+0".to_string();
        }
        
        let mut exp = 0;
        let mut num = n;
        
        while num >= 10.0 {
            num = num / 10.0;
            exp = exp + 1;
        }
        while num < 1.0 && num > 0.0 {
            num = num * 10.0;
            exp = exp - 1;
        }
        
        let int_part = (num as int);
        let frac_part = num - (int_part as float);
        
        let mut result = int_to_string(int_part);
        
        if precision > 0 {
            result.push('.');
            let mut temp = frac_part;
            for _ in 0..precision {
                temp = temp * 10.0;
                let digit = (temp as int);
                result.push(int_to_char(digit));
                temp = temp - (digit as float);
            }
        }
        
        let sign = if exp >= 0 { "+" } else { "-" };
        result = result + "e" + &sign + &int_to_string(exp.abs());
        
        result
    }

    fn int_to_string(n: int) -> string {
        if n == 0 {
            return "0".to_string();
        }
        
        let mut num = n;
        let negative = num < 0;
        if negative {
            num = -num;
        }
        
        let mut chars = Vec::new();
        while num > 0 {
            let d = (num % 10) as int;
            chars.push(int_to_char(d));
            num = num / 10;
        }
        
        chars.reverse();
        let mut result = chars.into_iter().collect::<string>();
        
        if negative {
            result = "-" + &result;
        }
        
        result
    }

    pub fn format<T: Display>(value: T) -> string {
        let mut f = Formatter::new();
        value.fmt(&mut f).unwrap();
        f.finish()
    }

    pub fn print<T: Display>(value: T) {
        let s = format(value);
        __print_str(s);
    }

    pub fn println<T: Display>(value: T) {
        let s = format(value);
        __print_str(s + "\n");
    }

    pub fn eprint<T: Display>(value: T) {
        let s = format(value);
        __print_str(s);
    }

    pub fn eprintln<T: Display>(value: T) {
        let s = format(value);
        __print_str(s + "\n");
    }
}

// ==================== 内置类型格式化实现 ====================

impl fmt::Display for int {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_int(*self)
    }
}

impl fmt::Display for float {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_float(*self)
    }
}

impl fmt::Display for string {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_string(self.clone())
    }
}

impl fmt::Display for bool {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_string(if *self { "true".to_string() } else { "false".to_string() })
    }
}

impl fmt::Display for char {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_char(*self)
    }
}

impl<T: fmt::Display> fmt::Display for Option<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Some(v) => {
                f.write_string("Some(".to_string())?;
                v.fmt(f)?;
                f.write_string(")".to_string())?;
                Ok(())
            }
            None => {
                f.write_string("None".to_string())
            }
        }
    }
}

impl<T: fmt::Display, E: fmt::Display> fmt::Display for Result<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Ok(v) => {
                f.write_string("Ok(".to_string())?;
                v.fmt(f)?;
                f.write_string(")".to_string())?;
                Ok(())
            }
            Err(e) => {
                f.write_string("Err(".to_string())?;
                e.fmt(f)?;
                f.write_string(")".to_string())?;
                Ok(())
            }
        }
    }
}

// Debug 实现

impl fmt::Debug for int {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Debug for float {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_string(format!("{}", *self))
    }
}

impl fmt::Debug for string {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_string("\"" + self + "\"")
    }
}

impl fmt::Debug for bool {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}

impl<T: fmt::Debug> fmt::Debug for Option<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Some(v) => {
                f.write_string("Some(".to_string())?;
                v.fmt(f)?;
                f.write_string(")".to_string())?;
                Ok(())
            }
            None => {
                f.write_string("None".to_string())
            }
        }
    }
}

impl<T: fmt::Debug, E: fmt::Debug> fmt::Debug for Result<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Ok(v) => {
                f.write_string("Ok(".to_string())?;
                v.fmt(f)?;
                f.write_string(")".to_string())?;
                Ok(())
            }
            Err(e) => {
                f.write_string("Err(".to_string())?;
                e.fmt(f)?;
                f.write_string(")".to_string())?;
                Ok(())
            }
        }
    }
}
