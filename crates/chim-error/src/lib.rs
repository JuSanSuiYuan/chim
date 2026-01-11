use chim_span::{FileId, Span};
use std::fmt;
use std::sync::Arc;

pub trait Diagnostic: fmt::Display + Send + Sync {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Lexer,
    Parser,
    TypeMismatch,
    UndefinedIdentifier,
    Redefinition,
    LifetimeError,
    BorrowError,
    EcsError,
    ActorError,
    Codegen,
    Io,
    Internal,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub span: Span,
    pub message: String,
    pub style: LabelStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelStyle {
    Primary,
    Secondary,
    Note,
    Help,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub replacements: Vec<Replacement>,
    pub applicability: Applicability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applicability {
    MachineApplicable,
    HasPlaceholders,
    Incorrect,
    Unapplicable,
}

#[derive(Debug, Clone)]
pub struct Replacement {
    pub span: Span,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct DiagnosticMessage {
    pub message: String,
    pub code: Option<String>,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
    Bug,
}

#[derive(Debug, Clone)]
pub struct ChimError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Option<Span>,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub suggestions: Vec<Suggestion>,
    pub code: Option<String>,
}

impl ChimError {
    pub fn new(kind: ErrorKind, message: String) -> Self {
        ChimError {
            kind,
            message,
            span: None,
            labels: Vec::new(),
            notes: Vec::new(),
            suggestions: Vec::new(),
            code: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_label(mut self, span: Span, message: String) -> Self {
        self.labels.push(Label {
            span,
            message,
            style: LabelStyle::Primary,
        });
        self
    }

    pub fn with_secondary_label(mut self, span: Span, message: String) -> Self {
        self.labels.push(Label {
            span,
            message,
            style: LabelStyle::Secondary,
        });
        self
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.notes.push(note);
        self
    }

    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn labels(&self) -> &[Label] {
        &self.labels
    }

    pub fn notes(&self) -> &[String] {
        &self.notes
    }
}

impl fmt::Display for ChimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)?;
        if let Some(span) = &self.span {
            write!(f, " at {}", span)?;
        }
        Ok(())
    }
}

impl std::error::Error for ChimError {}

#[derive(Debug, Default)]
pub struct ErrorReporter {
    source_map: Option<Arc<chim_span::SourceMap>>,
    errors: Vec<ChimError>,
    warnings: Vec<ChimError>,
}

impl ErrorReporter {
    pub fn new() -> Self {
        ErrorReporter::default()
    }

    pub fn with_source_map(mut self, source_map: Arc<chim_span::SourceMap>) -> Self {
        self.source_map = Some(source_map);
        self
    }

    pub fn report_error(&mut self, error: ChimError) {
        self.errors.push(error);
    }

    pub fn report_warning(&mut self, warning: ChimError) {
        self.warnings.push(warning);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    pub fn take_errors(&mut self) -> Vec<ChimError> {
        std::mem::take(&mut self.errors)
    }

    pub fn take_warnings(&mut self) -> Vec<ChimError> {
        std::mem::take(&mut self.warnings)
    }

    pub fn format(&self) -> String {
        let mut output = String::new();

        for error in &self.errors {
            output.push_str(&self.format_diagnostic(error));
            output.push('\n');
        }

        for warning in &self.warnings {
            output.push_str(&self.format_diagnostic(warning));
            output.push('\n');
        }

        output
    }

    fn format_diagnostic(&self, diag: &ChimError) -> String {
        let mut output = String::new();

        let severity = match diag.kind {
            ErrorKind::Lexer => "lexer error",
            ErrorKind::Parser => "syntax error",
            ErrorKind::TypeMismatch => "type error",
            ErrorKind::UndefinedIdentifier => "undefined identifier",
            ErrorKind::Redefinition => "redefinition error",
            ErrorKind::LifetimeError => "lifetime error",
            ErrorKind::BorrowError => "borrow check error",
            ErrorKind::EcsError => "ECS error",
            ErrorKind::ActorError => "Actor error",
            ErrorKind::Codegen => "code generation error",
            ErrorKind::Io => "I/O error",
            ErrorKind::Internal => "internal compiler error",
        };

        output.push_str(&format!("error[{}]: {}\n", diag.kind.as_str(), diag.message));

        if let Some(span) = &diag.span {
            output.push_str(&format!("  --> {}\n", span));

            if let Some(source_map) = &self.source_map {
                if let Some(file) = source_map.get_file(span.file_id) {
                    let lines = file.snippet_with_context(span, 2);
                    for line in &lines {
                        output.push_str(&format!("{}\n", line));
                    }
                }
            }
        }

        for label in &diag.labels {
            output.push_str(&format!("  {}: {}\n", label.style.as_str(), label.message));
        }

        for note in &diag.notes {
            output.push_str(&format!("  note: {}\n", note));
        }

        for suggestion in &diag.suggestions {
            output.push_str(&format!("  help: {}\n", suggestion.message));
        }

        output
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Lexer => write!(f, "E0001"),
            ErrorKind::Parser => write!(f, "E0002"),
            ErrorKind::TypeMismatch => write!(f, "E0003"),
            ErrorKind::UndefinedIdentifier => write!(f, "E0004"),
            ErrorKind::Redefinition => write!(f, "E0005"),
            ErrorKind::LifetimeError => write!(f, "E0006"),
            ErrorKind::BorrowError => write!(f, "E0007"),
            ErrorKind::EcsError => write!(f, "E0008"),
            ErrorKind::ActorError => write!(f, "E0009"),
            ErrorKind::Codegen => write!(f, "E0010"),
            ErrorKind::Io => write!(f, "E0011"),
            ErrorKind::Internal => write!(f, "E0999"),
        }
    }
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorKind::Lexer => "lexer",
            ErrorKind::Parser => "parser",
            ErrorKind::TypeMismatch => "type",
            ErrorKind::UndefinedIdentifier => "undefined",
            ErrorKind::Redefinition => "redefinition",
            ErrorKind::LifetimeError => "lifetime",
            ErrorKind::BorrowError => "borrow",
            ErrorKind::EcsError => "ecs",
            ErrorKind::ActorError => "actor",
            ErrorKind::Codegen => "codegen",
            ErrorKind::Io => "io",
            ErrorKind::Internal => "internal",
        }
    }
}

impl fmt::Display for LabelStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LabelStyle::Primary => write!(f, "^^"),
            LabelStyle::Secondary => write!(f, "--"),
            LabelStyle::Note => write!(f, "##"),
            LabelStyle::Help => write!(f, "??"),
        }
    }
}

impl LabelStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            LabelStyle::Primary => "^^",
            LabelStyle::Secondary => "--",
            LabelStyle::Note => "##",
            LabelStyle::Help => "??",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chim_span::{FileId, Span};

    #[test]
    fn test_error_creation() {
        let error = ChimError::new(ErrorKind::TypeMismatch, "mismatched types".to_string());
        assert_eq!(error.kind, ErrorKind::TypeMismatch);
        assert_eq!(error.message, "mismatched types");
    }

    #[test]
    fn test_error_with_span() {
        let error = ChimError::new(ErrorKind::TypeMismatch, "mismatched types".to_string())
            .with_span(Span::new(FileId(0), 10, 20, 1, 10));
        assert!(error.span.is_some());
    }

    #[test]
    fn test_error_with_label() {
        let error = ChimError::new(ErrorKind::TypeMismatch, "mismatched types".to_string())
            .with_label(Span::new(FileId(0), 10, 20, 1, 10), "expected i32, found str".to_string());
        assert_eq!(error.labels.len(), 1);
    }

    #[test]
    fn test_error_reporter() {
        let mut reporter = ErrorReporter::new();
        let error = ChimError::new(ErrorKind::TypeMismatch, "mismatched types".to_string());
        reporter.report_error(error);
        assert!(reporter.has_errors());
        assert_eq!(reporter.error_count(), 1);
    }
}
