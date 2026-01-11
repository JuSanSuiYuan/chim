// ==================== 时间标准库 ====================
// 时间获取、格式化、Duration 等功能

// ==================== 系统时间 ====================
pub fn now() -> Time {
    let ms = __time_now();
    Time::from_millis(ms)
}

pub fn unix_time() -> int {
    __unix_time()
}

pub fn unix_time_millis() -> int {
    __unix_time_millis()
}

// ==================== 时间点 ====================
pub struct Time {
    year: int,
    month: int,    // 1-12
    day: int,      // 1-31
    hour: int,     // 0-23
    minute: int,   // 0-59
    second: int,   // 0-59
    millis: int,   // 0-999
    weekday: int,  // 0=周一, 6=周日
}

impl Time {
    pub fn from_millis(millis: int) -> Time {
        __time_from_millis(millis)
    }
    
    pub fn now() -> Time {
        now()
    }
    
    // 属性
    pub fn year(&self) -> int { self.year }
    pub fn month(&self) -> int { self.month }
    pub fn day(&self) -> int { self.day }
    pub fn hour(&self) -> int { self.hour }
    pub fn minute(&self) -> int { self.minute }
    pub fn second(&self) -> int { self.second }
    pub fn millis(&self) -> int { self.millis }
    pub fn weekday(&self) -> int { self.weekday }
    
    // 格式化
    pub fn to_string(&self) -> string {
        self.format("%Y-%m-%d %H:%M:%S")
    }
    
    pub fn format(&self, fmt: string) -> string {
        __time_format(self, fmt)
    }
    
    pub fn to_iso8601(&self) -> string {
        format!("{}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            self.year, self.month, self.day,
            self.hour, self.minute, self.second, self.millis)
    }
    
    pub fn to_rfc2822(&self) -> string {
        let weekday_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        let month_names = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                          "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        format!("{}, {:02} {} {} {:02}:{:02}:{:02} +0000",
            weekday_names[self.weekday],
            self.day, month_names[self.month - 1],
            self.year, self.hour, self.minute, self.second)
    }
    
    // 转换
    pub fn to_millis(&self) -> int {
        __time_to_millis(self)
    }
    
    pub fn timestamp(&self) -> int {
        self.to_millis() / 1000
    }
    
    // 创建
    pub fn utc(year: int, month: int, day: int, hour: int, minute: int, second: int) -> Time {
        __time_create_utc(year, month, day, hour, minute, second, 0)
    }
    
    pub fn local(year: int, month: int, day: int, hour: int, minute: int, second: int) -> Time {
        __time_create_local(year, month, day, hour, minute, second, 0)
    }
    
    pub fn today() -> Time {
        let t = now();
        Time::utc(t.year(), t.month(), t.day(), 0, 0, 0)
    }
}

// ==================== 时长 ====================
pub struct Duration {
    millis: int,
}

impl Duration {
    pub fn from_millis(ms: int) -> Duration {
        Duration { millis: ms }
    }
    
    pub fn from_secs(s: int) -> Duration {
        Duration { millis: s * 1000 }
    }
    
    pub fn from_mins(m: int) -> Duration {
        Duration { millis: m * 60 * 1000 }
    }
    
    pub fn from_hours(h: int) -> Duration {
        Duration { millis: h * 60 * 60 * 1000 }
    }
    
    pub fn from_days(d: int) -> Duration {
        Duration { millis: d * 24 * 60 * 60 * 1000 }
    }
    
    pub fn zero() -> Duration {
        Duration { millis: 0 }
    }
    
    // 属性
    pub fn millis(&self) -> int { self.millis }
    pub fn secs(&self) -> int { self.millis / 1000 }
    pub fn mins(&self) -> int { self.millis / (60 * 1000) }
    pub fn hours(&self) -> int { self.millis / (60 * 60 * 1000) }
    pub fn days(&self) -> int { self.millis / (24 * 60 * 60 * 1000) }
    
    pub fn subsec_millis(&self) -> int { self.millis % 1000 }
    pub fn subsec_micros(&self) -> int { (self.millis % 1000) * 1000 }
    
    // 运算
    pub fn checked_add(&self, other: &Duration) -> Option<Duration> {
        let result = self.millis + other.millis;
        if result >= 0 { Option::Some(Duration { millis: result }) }
        else { Option::None }
    }
    
    pub fn checked_sub(&self, other: &Duration) -> Option<Duration> {
        let result = self.millis - other.millis;
        if result >= 0 { Option::Some(Duration { millis: result }) }
        else { Option::None }
    }
    
    pub fn mul(&self, n: int) -> Duration {
        Duration { millis: self.millis * n }
    }
    
    pub fn div(&self, n: int) -> Duration {
        Duration { millis: self.millis / n }
    }
    
    // 格式化
    pub fn to_string(&self) -> string {
        let secs = self.millis / 1000;
        let millis = self.millis % 1000;
        
        if secs < 60 {
            format!("{}.{:03}s", secs, millis)
        } else if secs < 3600 {
            let mins = secs / 60;
            let secs = secs % 60;
            format!("{}m {:02}.{:03}s", mins, secs, millis)
        } else if secs < 86400 {
            let hours = secs / 3600;
            let mins = (secs % 3600) / 60;
            format!("{}h {:02}m {:02}.{:03}s", hours, mins, secs % 60, millis)
        } else {
            let days = secs / 86400;
            let hours = (secs % 86400) / 3600;
            format!("{}d {:02}h {:02}m", days, hours, (secs % 3600) / 60)
        }
    }
}

// ==================== 睡眠 ====================
pub fn sleep(duration: Duration) {
    __sleep(duration.millis)
}

pub fn sleep_millis(ms: int) {
    __sleep(ms)
}

pub fn sleep_secs(s: int) {
    __sleep(s * 1000)
}

// ==================== 计时器 ====================
pub struct Instant {
    start_millis: int,
}

impl Instant {
    pub fn now() -> Instant {
        Instant { start_millis: __time_now() }
    }
    
    pub fn elapsed(&self) -> Duration {
        Duration::from_millis(__time_now() - self.start_millis)
    }
    
    pub fn elapsed_millis(&self) -> int {
        __time_now() - self.start_millis
    }
    
    pub fn elapsed_secs(&self) -> float {
        (__time_now() - self.start_millis) as float / 1000.0
    }
}

// ==================== 时区 ====================
pub struct TimeZone {
    name: string,
    offset_hours: int,
}

impl TimeZone {
    pub fn utc() -> TimeZone {
        TimeZone { name: "UTC".to_string(), offset_hours: 0 }
    }
    
    pub fn local() -> TimeZone {
        __timezone_local()
    }
    
    pub fn from_offset(offset_hours: int) -> TimeZone {
        let name = format!("UTC{}{}", 
            if offset_hours >= 0 { "+" } else { "" },
            offset_hours);
        TimeZone { name, offset_hours }
    }
    
    pub fn from_name(name: string) -> Option<TimeZone> {
        __timezone_from_name(name)
    }
    
    pub fn name(&self) -> string { self.name }
    pub fn offset_hours(&self) -> int { self.offset_hours }
    pub fn offset_secs(&self) -> int { self.offset_hours * 3600 }
    
    pub fn convert(&self, time: &Time) -> Time {
        __timezone_convert(time, self.offset_hours)
    }
}
