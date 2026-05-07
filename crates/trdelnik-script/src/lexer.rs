//! Logos-based lexer for TrdelScript
//!
//! This module provides a fast, derive-based lexer for tokenizing TrdelScript source code.

use crate::ast::Span;
use logos::Logos;
use std::hash::{Hash, Hasher};

/// Wrapper for f64 that implements Hash and Eq via bit representation
#[derive(Debug, Clone, Copy)]
pub struct OrderedFloat(pub f64);

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for OrderedFloat {}

impl Hash for OrderedFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl std::fmt::Display for OrderedFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Token types for TrdelScript
#[derive(Logos, Debug, Clone, PartialEq, Eq, Hash)]
#[logos(skip r"[ \t\n\f]+")]           // Skip whitespace
#[logos(skip r"//[^\n]*")]             // Skip line comments
pub enum Token {
    // =========================================================================
    // Keywords
    // =========================================================================
    #[token("strategy")]
    Strategy,

    #[token("param")]
    Param,

    #[token("let")]
    Let,

    #[token("entry")]
    Entry,

    #[token("exit")]
    Exit,

    #[token("when")]
    When,

    #[token("plot")]
    Plot,

    #[token("long")]
    Long,

    #[token("short")]
    Short,

    #[token("all")]
    All,

    #[token("stop_loss")]
    StopLoss,

    #[token("take_profit")]
    TakeProfit,

    #[token("timeframe")]
    Timeframe,

    #[token("color")]
    Color,

    #[token("panel")]
    Panel,

    #[token("style")]
    Style,

    #[token("if")]
    If,

    #[token("then")]
    Then,

    #[token("else")]
    Else,

    #[token("fn")]
    Fn,

    // =========================================================================
    // Types
    // =========================================================================
    #[token("int")]
    TypeInt,

    #[token("float")]
    TypeFloat,

    #[token("bool")]
    TypeBool,

    // =========================================================================
    // Boolean literals
    // =========================================================================
    #[token("true")]
    True,

    #[token("false")]
    False,

    // =========================================================================
    // Built-in data sources
    // =========================================================================
    #[token("open")]
    Open,

    #[token("high")]
    High,

    #[token("low")]
    Low,

    #[token("close")]
    Close,

    #[token("volume")]
    Volume,

    // =========================================================================
    // Literals
    // =========================================================================
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok().map(OrderedFloat))]
    Float(OrderedFloat),

    #[regex(r"[0-9]+", priority = 2, callback = |lex| lex.slice().parse::<i64>().ok())]
    Int(i64),

    #[regex(r#""[^"]*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    String(String),

    // =========================================================================
    // Identifiers (must come after keywords to not match them)
    // =========================================================================
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", priority = 1, callback = |lex| lex.slice().to_string())]
    Ident(String),

    // =========================================================================
    // Operators
    // =========================================================================
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("%")]
    Percent,

    #[token("==")]
    EqEq,

    #[token("!=")]
    NotEq,

    #[token("<=")]
    Lte,

    #[token(">=")]
    Gte,

    #[token("<")]
    Lt,

    #[token(">")]
    Gt,

    #[token("=")]
    Eq,

    #[token("and")]
    And,

    #[token("or")]
    Or,

    #[token("not")]
    Not,

    // =========================================================================
    // Delimiters
    // =========================================================================
    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    #[token(".")]
    Dot,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Strategy => write!(f, "strategy"),
            Token::Param => write!(f, "param"),
            Token::Let => write!(f, "let"),
            Token::Entry => write!(f, "entry"),
            Token::Exit => write!(f, "exit"),
            Token::When => write!(f, "when"),
            Token::Plot => write!(f, "plot"),
            Token::Long => write!(f, "long"),
            Token::Short => write!(f, "short"),
            Token::All => write!(f, "all"),
            Token::StopLoss => write!(f, "stop_loss"),
            Token::TakeProfit => write!(f, "take_profit"),
            Token::Timeframe => write!(f, "timeframe"),
            Token::Color => write!(f, "color"),
            Token::Panel => write!(f, "panel"),
            Token::Style => write!(f, "style"),
            Token::TypeInt => write!(f, "int"),
            Token::TypeFloat => write!(f, "float"),
            Token::TypeBool => write!(f, "bool"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Open => write!(f, "open"),
            Token::High => write!(f, "high"),
            Token::Low => write!(f, "low"),
            Token::Close => write!(f, "close"),
            Token::Volume => write!(f, "volume"),
            Token::Int(n) => write!(f, "{}", n),
            Token::Float(n) => write!(f, "{}", n.0),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Eq => write!(f, "="),
            Token::EqEq => write!(f, "=="),
            Token::NotEq => write!(f, "!="),
            Token::Lt => write!(f, "<"),
            Token::Gt => write!(f, ">"),
            Token::Lte => write!(f, "<="),
            Token::Gte => write!(f, ">="),
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Not => write!(f, "not"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::Dot => write!(f, "."),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Else => write!(f, "else"),
            Token::Fn => write!(f, "fn"),
        }
    }
}

/// Lex a source string into a vector of (token, span) pairs.
///
/// Returns an error if any invalid tokens are encountered.
pub fn lex(source: &str) -> Result<Vec<(Token, Span)>, LexError> {
    let mut lexer = Token::lexer(source);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    while let Some(result) = lexer.next() {
        let span = lexer.span();
        match result {
            Ok(token) => tokens.push((token, span)),
            Err(()) => {
                errors.push(LexError {
                    span: span.clone(),
                    text: source[span].to_string(),
                });
            }
        }
    }

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors.remove(0))
    }
}

/// Error from lexing an invalid token
#[derive(Debug, Clone)]
pub struct LexError {
    pub span: Span,
    pub text: String,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid token '{}' at {:?}", self.text, self.span)
    }
}

impl std::error::Error for LexError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let source = "strategy param let entry exit when plot";
        let tokens = lex(source).unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                Token::Strategy,
                Token::Param,
                Token::Let,
                Token::Entry,
                Token::Exit,
                Token::When,
                Token::Plot,
            ]
        );
    }

    #[test]
    fn test_literals() {
        let source = r#"42 3.14 "hello world""#;
        let tokens = lex(source).unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                Token::Int(42),
                Token::Float(OrderedFloat(3.14)),
                Token::String("hello world".to_string()),
            ]
        );
    }

    #[test]
    fn test_operators() {
        let source = "+ - * / % == != < > <= >= and or not";
        let tokens = lex(source).unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                Token::Percent,
                Token::EqEq,
                Token::NotEq,
                Token::Lt,
                Token::Gt,
                Token::Lte,
                Token::Gte,
                Token::And,
                Token::Or,
                Token::Not,
            ]
        );
    }

    #[test]
    fn test_data_sources() {
        let source = "open high low close volume";
        let tokens = lex(source).unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(
            kinds,
            vec![Token::Open, Token::High, Token::Low, Token::Close, Token::Volume,]
        );
    }

    #[test]
    fn test_identifiers() {
        let source = "fast_sma slow_sma my_var123";
        let tokens = lex(source).unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                Token::Ident("fast_sma".to_string()),
                Token::Ident("slow_sma".to_string()),
                Token::Ident("my_var123".to_string()),
            ]
        );
    }

    #[test]
    fn test_delimiters() {
        let source = "( ) { } , : .";
        let tokens = lex(source).unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                Token::LParen,
                Token::RParen,
                Token::LBrace,
                Token::RBrace,
                Token::Comma,
                Token::Colon,
                Token::Dot,
            ]
        );
    }

    #[test]
    fn test_comments_skipped() {
        let source = "let x = 5 // this is a comment\nlet y = 10";
        let tokens = lex(source).unwrap();
        // Should not include comment tokens
        assert!(tokens.iter().all(|(t, _)| !matches!(t, Token::Ident(s) if s.contains("comment"))));
    }

    #[test]
    fn test_complete_expression() {
        let source = "let fast = sma(close, 12)";
        let tokens = lex(source).unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                Token::Let,
                Token::Ident("fast".to_string()),
                Token::Eq,
                Token::Ident("sma".to_string()),
                Token::LParen,
                Token::Close,
                Token::Comma,
                Token::Int(12),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn test_destructuring() {
        let source = "let { upper, middle, lower } = bollinger(close, 20, 2.0)";
        let tokens = lex(source).unwrap();
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::LBrace)));
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::RBrace)));
    }

    #[test]
    fn test_entry_statement() {
        let source = "entry long when crossover(fast, slow)";
        let tokens = lex(source).unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                Token::Entry,
                Token::Long,
                Token::When,
                Token::Ident("crossover".to_string()),
                Token::LParen,
                Token::Ident("fast".to_string()),
                Token::Comma,
                Token::Ident("slow".to_string()),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn test_booleans() {
        let source = "true false";
        let tokens = lex(source).unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![Token::True, Token::False,]);
    }

    #[test]
    fn test_percentage() {
        let source = "stop_loss 2 %";
        let tokens = lex(source).unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![Token::StopLoss, Token::Int(2), Token::Percent,]);
    }
}
