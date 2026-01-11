use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type I18nResult<T> = Result<T, I18nError>;

#[derive(Debug, Clone)]
pub enum I18nError {
    LocaleNotFound(String),
    TranslationNotFound(String),
    InvalidLocale(String),
    FormatError(String),
}

impl std::fmt::Display for I18nError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            I18nError::LocaleNotFound(locale) => {
                write!(f, "Locale not found: {}", locale)
            }
            I18nError::TranslationNotFound(key) => {
                write!(f, "Translation not found: {}", key)
            }
            I18nError::InvalidLocale(locale) => {
                write!(f, "Invalid locale: {}", locale)
            }
            I18nError::FormatError(msg) => write!(f, "Format error: {}", msg),
        }
    }
}

impl std::error::Error for I18nError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale {
    pub language: String,
    pub region: Option<String>,
    pub script: Option<String>,
}

impl Locale {
    pub fn new(language: String) -> Self {
        Locale {
            language,
            region: None,
            script: None,
        }
    }

    pub fn with_region(mut self, region: String) -> Self {
        self.region = Some(region);
        self
    }

    pub fn with_script(mut self, script: String) -> Self {
        self.script = Some(script);
        self
    }

    pub fn from_str(s: &str) -> I18nResult<Self> {
        let parts: Vec<&str> = s.split('-').collect();

        if parts.is_empty() || parts[0].is_empty() {
            return Err(I18nError::InvalidLocale(s.to_string()));
        }

        let language = parts[0].to_lowercase();
        let mut locale = Locale::new(language);

        if parts.len() > 1 {
            let region = parts[1].to_uppercase();
            locale = locale.with_region(region);
        }

        if parts.len() > 2 {
            let script = parts[2].to_title_case();
            locale = locale.with_script(script);
        }

        Ok(locale)
    }

    pub fn to_string(&self) -> String {
        let mut result = self.language.clone();

        if let Some(ref region) = self.region {
            result.push('-');
            result.push_str(region);
        }

        if let Some(ref script) = self.script {
            result.push('-');
            result.push_str(script);
        }

        result
    }

    pub fn matches(&self, other: &Locale) -> bool {
        if self.language != other.language {
            return false;
        }

        if let Some(ref region) = self.region {
            if let Some(ref other_region) = other.region {
                if region != other_region {
                    return false;
                }
            }
        }

        if let Some(ref script) = self.script {
            if let Some(ref other_script) = other.script {
                if script != other_script {
                    return false;
                }
            }
        }

        true
    }
}

impl Default for Locale {
    fn default() -> Self {
        Locale::new("en".to_string())
    }
}

pub struct Translation {
    pub key: String,
    pub value: String,
    pub context: Option<String>,
    pub plural: Option<PluralForms>,
}

#[derive(Debug, Clone)]
pub struct PluralForms {
    pub zero: Option<String>,
    pub one: Option<String>,
    pub two: Option<String>,
    pub few: Option<String>,
    pub many: Option<String>,
    pub other: String,
}

impl PluralForms {
    pub fn get(&self, n: usize) -> &str {
        match n {
            0 => self.zero.as_ref().unwrap_or(&self.other),
            1 => self.one.as_ref().unwrap_or(&self.other),
            2 => self.two.as_ref().unwrap_or(&self.other),
            3..=10 => self.few.as_ref().unwrap_or(&self.other),
            _ => self.many.as_ref().unwrap_or(&self.other),
        }
    }
}

pub struct MessageFormatter {
    locale: Locale,
}

impl MessageFormatter {
    pub fn new(locale: Locale) -> Self {
        MessageFormatter { locale }
    }

    pub fn format(&self, template: &str, args: &[&str]) -> String {
        let mut result = template.to_string();

        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("{{{}}}", i);
            result = result.replace(&placeholder, arg);
        }

        result
    }

    pub fn format_named(&self, template: &str, args: &HashMap<&str, &str>) -> String {
        let mut result = template.to_string();

        for (key, value) in args {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }

    pub fn format_plural(&self, template: &str, n: usize) -> String {
        let plural = PluralForms {
            zero: Some(format!("{} items", n)),
            one: Some(format!("{} item", n)),
            other: format!("{} items", n),
        };

        plural.get(n).to_string()
    }
}

pub struct I18n {
    current_locale: Arc<RwLock<Locale>>,
    translations: Arc<RwLock<HashMap<String, HashMap<String, Translation>>>>,
    formatter: MessageFormatter,
}

impl I18n {
    pub fn new(default_locale: Locale) -> Self {
        let formatter = MessageFormatter::new(default_locale.clone());
        I18n {
            current_locale: Arc::new(RwLock::new(default_locale)),
            translations: Arc::new(RwLock::new(HashMap::new())),
            formatter,
        }
    }

    pub fn set_locale(&self, locale: Locale) {
        let mut current = self.current_locale.write().unwrap();
        *current = locale;
        self.formatter = MessageFormatter::new(current.clone());
    }

    pub fn get_locale(&self) -> Locale {
        let current = self.current_locale.read().unwrap();
        current.clone()
    }

    pub fn add_translation(&self, locale: &Locale, translation: Translation) {
        let mut translations = self.translations.write().unwrap();
        let locale_key = locale.to_string();

        translations
            .entry(locale_key)
            .or_insert_with(HashMap::new)
            .insert(translation.key.clone(), translation);
    }

    pub fn add_translations(&self, locale: &Locale, translations: Vec<Translation>) {
        let mut trans_map = self.translations.write().unwrap();
        let locale_key = locale.to_string();

        for translation in translations {
            trans_map
                .entry(locale_key.clone())
                .or_insert_with(HashMap::new)
                .insert(translation.key.clone(), translation);
        }
    }

    pub fn translate(&self, key: &str) -> I18nResult<String> {
        let current_locale = self.get_locale();
        let translations = self.translations.read().unwrap();
        let locale_key = current_locale.to_string();

        if let Some(locale_trans) = translations.get(&locale_key) {
            if let Some(translation) = locale_trans.get(key) {
                return Ok(translation.value.clone());
            }
        }

        Err(I18nError::TranslationNotFound(key.to_string()))
    }

    pub fn translate_with_args(&self, key: &str, args: &[&str]) -> I18nResult<String> {
        let template = self.translate(key)?;
        Ok(self.formatter.format(&template, args))
    }

    pub fn translate_named(&self, key: &str, args: &HashMap<&str, &str>) -> I18nResult<String> {
        let template = self.translate(key)?;
        Ok(self.formatter.format_named(&template, args))
    }

    pub fn translate_plural(&self, key: &str, n: usize) -> I18nResult<String> {
        let current_locale = self.get_locale();
        let translations = self.translations.read().unwrap();
        let locale_key = current_locale.to_string();

        if let Some(locale_trans) = translations.get(&locale_key) {
            if let Some(translation) = locale_trans.get(key) {
                if let Some(ref plural) = translation.plural {
                    return Ok(plural.get(n).to_string());
                }
            }
        }

        Err(I18nError::TranslationNotFound(key.to_string()))
    }

    pub fn has_translation(&self, key: &str) -> bool {
        let current_locale = self.get_locale();
        let translations = self.translations.read().unwrap();
        let locale_key = current_locale.to_string();

        if let Some(locale_trans) = translations.get(&locale_key) {
            return locale_trans.contains_key(key);
        }

        false
    }

    pub fn get_available_locales(&self) -> Vec<String> {
        let translations = self.translations.read().unwrap();
        translations.keys().cloned().collect()
    }

    pub fn get_translation_keys(&self, locale: &Locale) -> Vec<String> {
        let translations = self.translations.read().unwrap();
        let locale_key = locale.to_string();

        if let Some(locale_trans) = translations.get(&locale_key) {
            return locale_trans.keys().cloned().collect();
        }

        Vec::new()
    }

    pub fn load_from_json(&self, locale: &Locale, json: &str) -> I18nResult<()> {
        let parsed: HashMap<String, String> = serde_json::from_str(json)
            .map_err(|e| I18nError::FormatError(e.to_string()))?;

        let translations: Vec<Translation> = parsed
            .into_iter()
            .map(|(key, value)| Translation {
                key,
                value,
                context: None,
                plural: None,
            })
            .collect();

        self.add_translations(locale, translations);
        Ok(())
    }

    pub fn load_from_file(&self, locale: &Locale, path: &std::path::Path) -> I18nResult<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| I18nError::FormatError(e.to_string()))?;

        self.load_from_json(locale, &content)
    }
}

pub struct I18nBuilder {
    default_locale: Locale,
    translations: Vec<(Locale, Vec<Translation>)>,
}

impl I18nBuilder {
    pub fn new(default_locale: Locale) -> Self {
        I18nBuilder {
            default_locale,
            translations: Vec::new(),
        }
    }

    pub fn add_locale(mut self, locale: Locale, translations: Vec<Translation>) -> Self {
        self.translations.push((locale, translations));
        self
    }

    pub fn build(self) -> I18n {
        let i18n = I18n::new(self.default_locale);

        for (locale, translations) in self.translations {
            i18n.add_translations(&locale, translations);
        }

        i18n
    }
}

pub fn detect_system_locale() -> Locale {
    std::env::var("LANG")
        .ok()
        .and_then(|lang| {
            let locale_str = lang.split('.').next().unwrap_or(&lang);
            Locale::from_str(locale_str).ok()
        })
        .unwrap_or_default()
}

pub fn format_number(n: f64, locale: &Locale) -> String {
    match locale.language.as_str() {
        "en" => format!("{:.2}", n),
        "de" => format!("{:.2}", n).replace('.', ","),
        "fr" => format!("{:.2}", n).replace('.', ","),
        "zh" => format!("{:.2}", n),
        "ja" => format!("{:.2}", n),
        _ => format!("{:.2}", n),
    }
}

pub fn format_date(date: &chrono::DateTime<chrono::Utc>, locale: &Locale) -> String {
    match locale.language.as_str() {
        "en" => date.format("%Y-%m-%d").to_string(),
        "de" => date.format("%d.%m.%Y").to_string(),
        "fr" => date.format("%d/%m/%Y").to_string(),
        "zh" => date.format("%Y年%m月%d日").to_string(),
        "ja" => date.format("%Y年%m月%d日").to_string(),
        _ => date.format("%Y-%m-%d").to_string(),
    }
}

pub fn format_currency(amount: f64, currency: &str, locale: &Locale) -> String {
    let formatted = format_number(amount, locale);
    match locale.language.as_str() {
        "en" => format!("{} {}", currency, formatted),
        "de" | "fr" => format!("{} {}", formatted, currency),
        "zh" => format!("{}{}", currency, formatted),
        "ja" => format!("{}{}", currency, formatted),
        _ => format!("{} {}", currency, formatted),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_from_str() {
        let locale = Locale::from_str("en-US").unwrap();
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, Some("US".to_string()));
    }

    #[test]
    fn test_locale_to_string() {
        let locale = Locale::new("zh".to_string()).with_region("CN".to_string());
        assert_eq!(locale.to_string(), "zh-CN");
    }

    #[test]
    fn test_locale_matches() {
        let locale1 = Locale::from_str("en-US").unwrap();
        let locale2 = Locale::from_str("en-GB").unwrap();
        let locale3 = Locale::from_str("zh-CN").unwrap();

        assert!(locale1.matches(&locale2));
        assert!(!locale1.matches(&locale3));
    }

    #[test]
    fn test_message_formatter() {
        let locale = Locale::new("en".to_string());
        let formatter = MessageFormatter::new(locale);
        let result = formatter.format("Hello, {}!", &["World"]);
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_message_formatter_named() {
        let locale = Locale::new("en".to_string());
        let formatter = MessageFormatter::new(locale);
        let mut args = HashMap::new();
        args.insert("name", "Alice");
        args.insert("age", "30");
        let result = formatter.format_named("Name: {name}, Age: {age}", &args);
        assert_eq!(result, "Name: Alice, Age: 30");
    }

    #[test]
    fn test_i18n() {
        let locale = Locale::new("en".to_string());
        let i18n = I18n::new(locale.clone());

        let translation = Translation {
            key: "hello".to_string(),
            value: "Hello, World!".to_string(),
            context: None,
            plural: None,
        };
        i18n.add_translation(&locale, translation);

        assert_eq!(i18n.translate("hello").unwrap(), "Hello, World!");
    }

    #[test]
    fn test_i18n_with_args() {
        let locale = Locale::new("en".to_string());
        let i18n = I18n::new(locale.clone());

        let translation = Translation {
            key: "greeting".to_string(),
            value: "Hello, {0}!".to_string(),
            context: None,
            plural: None,
        };
        i18n.add_translation(&locale, translation);

        assert_eq!(
            i18n.translate_with_args("greeting", &["Alice"]).unwrap(),
            "Hello, Alice!"
        );
    }

    #[test]
    fn test_i18n_builder() {
        let locale = Locale::new("en".to_string());
        let translations = vec![Translation {
            key: "test".to_string(),
            value: "Test".to_string(),
            context: None,
            plural: None,
        }];

        let i18n = I18nBuilder::new(locale.clone())
            .add_locale(locale, translations)
            .build();

        assert_eq!(i18n.translate("test").unwrap(), "Test");
    }

    #[test]
    fn test_format_number() {
        let locale = Locale::new("en".to_string());
        assert_eq!(format_number(1234.56, &locale), "1234.56");

        let locale = Locale::new("de".to_string());
        assert_eq!(format_number(1234.56, &locale), "1234,56");
    }

    #[test]
    fn test_format_currency() {
        let locale = Locale::new("en".to_string());
        assert_eq!(format_currency(1234.56, "USD", &locale), "USD 1234.56");

        let locale = Locale::new("zh".to_string());
        assert_eq!(format_currency(1234.56, "CNY", &locale), "CNY1234.56");
    }
}
