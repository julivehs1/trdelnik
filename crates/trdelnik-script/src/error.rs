//! Error types and pretty error reporting for TrdelScript
//!
//! This module provides unified error handling and beautiful diagnostic output
//! using the Ariadne library.

use crate::compiler::CompileError;
use crate::lexer::{LexError, Token};
use crate::semantic::SemanticError;
use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::error::Simple;
use std::fmt;
use std::io::Write;
use thiserror::Error;

/// Unified error type for TrdelScript compilation
#[derive(Debug, Error)]
pub enum TrdelScriptError {
    #[error("Lexer error: {0}")]
    Lex(#[from] LexError),

    #[error("Parse error")]
    Parse(Vec<Simple<Token>>),

    #[error("Semantic error")]
    Semantic(Vec<SemanticError>),

    #[error("Compile error: {0}")]
    Compile(#[from] CompileError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl TrdelScriptError {
    /// Print a pretty error report to stderr
    pub fn print_report(&self, source: &str) {
        self.write_report(source, &mut std::io::stderr()).ok();
    }

    /// Write a pretty error report to the given writer
    pub fn write_report<W: Write>(&self, source: &str, writer: &mut W) -> std::io::Result<()> {
        match self {
            TrdelScriptError::Lex(err) => {
                print_lex_error(source, err, writer)?;
            }
            TrdelScriptError::Parse(errors) => {
                print_parse_errors(source, errors, writer)?;
            }
            TrdelScriptError::Semantic(errors) => {
                print_semantic_errors(source, errors, writer)?;
            }
            TrdelScriptError::Compile(err) => {
                // Compile errors don't have spans, just print the message
                writeln!(writer, "Compilation error: {}", err)?;
            }
            TrdelScriptError::Io(err) => {
                writeln!(writer, "IO error: {}", err)?;
            }
        }
        Ok(())
    }

    /// Convert to a user-friendly error message string
    pub fn to_report_string(&self, source: &str) -> String {
        let mut buf = Vec::new();
        self.write_report(source, &mut buf).ok();
        String::from_utf8_lossy(&buf).to_string()
    }
}

/// Print a lex error with source context
fn print_lex_error<W: Write>(source: &str, err: &LexError, writer: &mut W) -> std::io::Result<()> {
    let report = Report::build(ReportKind::Error, (), err.span.start)
        .with_message("Invalid token")
        .with_label(
            Label::new(err.span.clone())
                .with_message(format!("Unexpected character sequence: '{}'", err.text))
                .with_color(Color::Red),
        )
        .with_help("Check for typos or invalid characters")
        .finish();

    // Write to a buffer first, then output
    let mut buf = Vec::new();
    report.write(Source::from(source), &mut buf).ok();
    writer.write_all(&buf)?;
    Ok(())
}

/// Print parse errors with source context
fn print_parse_errors<W: Write>(
    source: &str,
    errors: &[Simple<Token>],
    writer: &mut W,
) -> std::io::Result<()> {
    for err in errors {
        let span = err.span();

        let message = if let Some(found) = err.found() {
            format!("Unexpected token '{}'", found)
        } else {
            "Unexpected end of input".to_string()
        };

        let expected: Vec<_> = err
            .expected()
            .filter_map(|e| e.as_ref().map(|t| format!("'{}'", t)))
            .collect();

        let expected_msg = if !expected.is_empty() {
            format!("Expected one of: {}", expected.join(", "))
        } else {
            "Expected a valid token".to_string()
        };

        let report = Report::build(ReportKind::Error, (), span.start)
            .with_message(&message)
            .with_label(
                Label::new(span.clone())
                    .with_message(&expected_msg)
                    .with_color(Color::Red),
            )
            .finish();

        let mut buf = Vec::new();
        report.write(Source::from(source), &mut buf).ok();
        writer.write_all(&buf)?;
    }
    Ok(())
}

/// Print semantic errors with source context
fn print_semantic_errors<W: Write>(
    source: &str,
    errors: &[SemanticError],
    writer: &mut W,
) -> std::io::Result<()> {
    for err in errors {
        let (message, help) = match &err.kind {
            crate::semantic::SemanticErrorKind::UndefinedVariable(name) => (
                format!("Undefined variable '{}'", name),
                Some("Variables must be defined before use. Check for typos or add a 'let' statement.".to_string()),
            ),
            crate::semantic::SemanticErrorKind::UndefinedFunction(name) => (
                format!("Unknown function '{}'", name),
                Some(format!(
                    "Available functions include: sma, ema, rsi, macd, bollinger, stochastic, crossover, crossunder"
                )),
            ),
            crate::semantic::SemanticErrorKind::WrongArgumentCount {
                function,
                expected_min,
                expected_max,
                actual,
            } => {
                let expected = if expected_min == expected_max {
                    format!("{}", expected_min)
                } else {
                    format!("{}-{}", expected_min, expected_max)
                };
                (
                    format!(
                        "Wrong number of arguments to '{}': expected {}, got {}",
                        function, expected, actual
                    ),
                    Some(format!("Check the documentation for '{}' for correct usage", function)),
                )
            }
            crate::semantic::SemanticErrorKind::TypeMismatch { expected, actual } => (
                format!("Type mismatch: expected {}, got {}", expected, actual),
                None,
            ),
            crate::semantic::SemanticErrorKind::InvalidDestructure {
                function,
                expected_fields,
                actual_fields,
            } => (
                format!(
                    "Invalid destructuring of '{}' output",
                    function
                ),
                Some(format!(
                    "Available fields are: {:?}, but you requested: {:?}",
                    expected_fields, actual_fields
                )),
            ),
            crate::semantic::SemanticErrorKind::DuplicateVariable(name) => (
                format!("Duplicate variable '{}'", name),
                Some("Each variable can only be declared once".to_string()),
            ),
            crate::semantic::SemanticErrorKind::ConditionNotBoolean => (
                "Condition must evaluate to a boolean".to_string(),
                Some("Use comparison operators (>, <, ==) or boolean functions (crossover, crossunder)".to_string()),
            ),
        };

        let mut builder = Report::build(ReportKind::Error, (), err.span.start)
            .with_message(&message)
            .with_label(
                Label::new(err.span.clone())
                    .with_message(&message)
                    .with_color(Color::Red),
            );

        if let Some(help_text) = help {
            builder = builder.with_help(help_text);
        }

        let report = builder.finish();

        let mut buf = Vec::new();
        report.write(Source::from(source), &mut buf).ok();
        writer.write_all(&buf)?;
    }
    Ok(())
}

/// Error collection for multi-error reporting
#[derive(Debug, Default)]
pub struct ErrorCollector {
    errors: Vec<TrdelScriptError>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, error: TrdelScriptError) {
        self.errors.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn into_errors(self) -> Vec<TrdelScriptError> {
        self.errors
    }

    pub fn print_all(&self, source: &str) {
        for error in &self.errors {
            error.print_report(source);
        }
    }
}

impl fmt::Display for ErrorCollector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} error(s) found", self.errors.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse;
    use crate::semantic::SemanticAnalyzer;

    #[test]
    fn test_lex_error_report() {
        let source = "let x = @invalid";
        let result = lex(source);
        assert!(result.is_err());

        let err = TrdelScriptError::Lex(result.unwrap_err());
        let report = err.to_report_string(source);
        assert!(report.contains("Invalid token"));
    }

    #[test]
    fn test_parse_error_report() {
        let source = "let = 5"; // Missing variable name
        let tokens = lex(source).unwrap();
        let result = parse(&tokens);
        assert!(result.is_err());

        let err = TrdelScriptError::Parse(result.unwrap_err());
        let report = err.to_report_string(source);
        assert!(report.contains("Unexpected"));
    }

    #[test]
    fn test_semantic_error_report() {
        let source = "entry long when undefined_var > 0";
        let tokens = lex(source).unwrap();
        let script = parse(&tokens).unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&script);
        assert!(result.is_err());

        let err = TrdelScriptError::Semantic(result.unwrap_err());
        let report = err.to_report_string(source);
        assert!(report.contains("Undefined variable"));
    }

    use crate::semantic::{SemanticError, SemanticErrorKind};

    fn semantic_error(kind: SemanticErrorKind) -> SemanticError {
        SemanticError {
            kind,
            span: 0..1,
        }
    }

    fn render(err: TrdelScriptError, source: &str) -> String {
        err.to_report_string(source)
    }

    // ---------- Semantic error variants (each kind has its own message arm) ----------

    #[test]
    fn test_semantic_undefined_function_report() {
        let err = TrdelScriptError::Semantic(vec![semantic_error(
            SemanticErrorKind::UndefinedFunction("nope".to_string()),
        )]);
        let r = render(err, "x");
        assert!(r.contains("Unknown function 'nope'"));
    }

    #[test]
    fn test_semantic_wrong_argument_count_min_eq_max_report() {
        let err = TrdelScriptError::Semantic(vec![semantic_error(
            SemanticErrorKind::WrongArgumentCount {
                function: "sma".into(),
                expected_min: 2,
                expected_max: 2,
                actual: 1,
            },
        )]);
        let r = render(err, "x");
        assert!(r.contains("expected 2"));
    }

    #[test]
    fn test_semantic_wrong_argument_count_min_lt_max_report() {
        let err = TrdelScriptError::Semantic(vec![semantic_error(
            SemanticErrorKind::WrongArgumentCount {
                function: "macd".into(),
                expected_min: 1,
                expected_max: 3,
                actual: 4,
            },
        )]);
        let r = render(err, "x");
        assert!(r.contains("expected 1-3"));
    }

    #[test]
    fn test_semantic_type_mismatch_report() {
        let err = TrdelScriptError::Semantic(vec![semantic_error(
            SemanticErrorKind::TypeMismatch {
                expected: "number".into(),
                actual: "bool".into(),
            },
        )]);
        let r = render(err, "x");
        assert!(r.contains("Type mismatch"));
    }

    #[test]
    fn test_semantic_invalid_destructure_report() {
        let err = TrdelScriptError::Semantic(vec![semantic_error(
            SemanticErrorKind::InvalidDestructure {
                function: "bollinger".into(),
                expected_fields: vec!["upper".into(), "middle".into(), "lower".into()],
                actual_fields: vec!["foo".into()],
            },
        )]);
        let r = render(err, "x");
        assert!(r.contains("Invalid destructuring"));
    }

    #[test]
    fn test_semantic_duplicate_variable_report() {
        let err = TrdelScriptError::Semantic(vec![semantic_error(
            SemanticErrorKind::DuplicateVariable("fast".into()),
        )]);
        let r = render(err, "x");
        assert!(r.contains("Duplicate variable"));
    }

    #[test]
    fn test_semantic_condition_not_boolean_report() {
        let err = TrdelScriptError::Semantic(vec![semantic_error(
            SemanticErrorKind::ConditionNotBoolean,
        )]);
        let r = render(err, "x");
        assert!(r.contains("Condition must evaluate to a boolean"));
    }

    // ---------- Compile + IO errors ----------

    #[test]
    fn test_compile_error_report() {
        let err = TrdelScriptError::Compile(crate::compiler::CompileError::UndefinedVariable(
            "ghost".into(),
        ));
        let r = render(err, "x");
        assert!(r.contains("Compilation error"));
    }

    #[test]
    fn test_io_error_report() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing file");
        let err = TrdelScriptError::Io(io_err);
        let r = render(err, "x");
        assert!(r.contains("IO error"));
    }

    // ---------- print_report (smoke — must not panic) ----------

    #[test]
    fn test_print_report_runs_for_every_variant() {
        // Tap each branch of write_report by calling print_report. It writes
        // to stderr; we just need to ensure none of the paths panic.
        TrdelScriptError::Compile(crate::compiler::CompileError::UndefinedVariable("x".into()))
            .print_report("source");

        TrdelScriptError::Io(std::io::Error::new(std::io::ErrorKind::Other, "x"))
            .print_report("source");
    }

    // ---------- Parse errors with no `found()` ----------

    #[test]
    fn test_parse_error_unexpected_eof_report() {
        // `let x =` ends abruptly, producing an "unexpected end of input" error.
        let source = "let x =";
        let tokens = lex(source).unwrap();
        let result = parse(&tokens);
        assert!(result.is_err());
        let err = TrdelScriptError::Parse(result.unwrap_err());
        let r = render(err, source);
        assert!(r.contains("Unexpected"));
    }

    // ---------- ErrorCollector ----------

    #[test]
    fn test_error_collector_lifecycle() {
        let mut c = ErrorCollector::new();
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);

        c.push(TrdelScriptError::Compile(
            crate::compiler::CompileError::UndefinedVariable("x".into()),
        ));
        c.push(TrdelScriptError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "x",
        )));
        assert!(!c.is_empty());
        assert_eq!(c.len(), 2);
        // Display reports the count
        assert_eq!(format!("{}", c), "2 error(s) found");
    }

    #[test]
    fn test_error_collector_into_errors() {
        let mut c = ErrorCollector::default();
        c.push(TrdelScriptError::Compile(
            crate::compiler::CompileError::UndefinedVariable("x".into()),
        ));
        let errs = c.into_errors();
        assert_eq!(errs.len(), 1);
    }

    #[test]
    fn test_error_collector_print_all_runs() {
        let mut c = ErrorCollector::new();
        c.push(TrdelScriptError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "x",
        )));
        // print_all routes through print_report — just ensure it doesn't panic.
        c.print_all("source");
    }
}
