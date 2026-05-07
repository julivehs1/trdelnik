//! Abstract Syntax Tree types for TrdelScript
//!
//! This module defines all the types that represent the parsed structure
//! of a TrdelScript program.

use std::ops::Range;

/// A source span representing a range of bytes in the source code
pub type Span = Range<usize>;

/// A value with an associated source span for error reporting
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

/// The root AST node representing an entire TrdelScript program
#[derive(Debug, Clone)]
pub struct Script {
    pub strategy: Option<StrategyDecl>,
    pub params: Vec<Spanned<ParamDecl>>,
    pub functions: Vec<Spanned<FunctionDef>>,
    pub statements: Vec<Spanned<Statement>>,
}

impl Script {
    pub fn new() -> Self {
        Self {
            strategy: None,
            params: Vec::new(),
            functions: Vec::new(),
            statements: Vec::new(),
        }
    }
}

/// A user-defined function with a single-expression body.
///
/// Calls are inlined at compile time — the body is substituted with the
/// argument expressions and the resulting graph is built normally. No
/// runtime closures, no recursion.
///
/// Example: `fn ma_diff(p) = sma(close, p) - sma(close, p * 2)`
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Spanned<Expr>,
}

impl Default for Script {
    fn default() -> Self {
        Self::new()
    }
}

/// Strategy declaration with optional timeframe
///
/// Example: `strategy "SMA Crossover" timeframe = H1`
#[derive(Debug, Clone)]
pub struct StrategyDecl {
    pub name: String,
    pub timeframe: Option<String>,
    pub span: Span,
}

/// Parameter declaration for user-configurable values
///
/// Example: `param fast_period: int = 12`
#[derive(Debug, Clone)]
pub struct ParamDecl {
    pub name: String,
    pub ty: ParamType,
    pub default: Option<Literal>,
}

/// Parameter types supported by TrdelScript
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
    Int,
    Float,
    Bool,
}

/// A statement in TrdelScript
#[derive(Debug, Clone)]
pub enum Statement {
    /// Variable binding: `let x = expr` or `let { a, b } = expr`
    Let(LetStmt),
    /// Entry signal: `entry long when condition`
    Entry(EntryStmt),
    /// Exit signal: `exit all when condition`
    Exit(ExitStmt),
    /// Plot statement: `plot expr color=blue`
    Plot(PlotStmt),
    /// Stop loss: `stop_loss 2%`
    StopLoss(f64),
    /// Take profit: `take_profit 6%`
    TakeProfit(f64),
}

/// Let statement for variable bindings
#[derive(Debug, Clone)]
pub struct LetStmt {
    pub pattern: Pattern,
    pub value: Spanned<Expr>,
}

/// Patterns for variable binding
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Simple binding: `let x = ...`
    Simple(String),
    /// Destructuring: `let { upper, middle, lower } = ...`
    Destructure(Vec<String>),
}

/// Expression types in TrdelScript
#[derive(Debug, Clone)]
pub enum Expr {
    /// Literal value
    Literal(Literal),
    /// Variable reference
    Ident(String),
    /// OHLCV data source
    DataSource(DataSource),
    /// Function call: `sma(close, 20)`
    Call {
        name: String,
        args: Vec<Spanned<Expr>>,
    },
    /// Field access: `bollinger.upper`
    FieldAccess {
        expr: Box<Spanned<Expr>>,
        field: String,
    },
    /// History access: `expr[n]` — value of `expr` n bars ago.
    /// `lag` must be a non-negative integer literal.
    Index {
        expr: Box<Spanned<Expr>>,
        lag: i64,
    },
    /// Conditional expression: `if cond then a else b`. Compiles to a
    /// SelectNode at runtime.
    If {
        cond: Box<Spanned<Expr>>,
        then_branch: Box<Spanned<Expr>>,
        else_branch: Box<Spanned<Expr>>,
    },
    /// Binary operation: `a + b`, `a > b`
    BinaryOp {
        left: Box<Spanned<Expr>>,
        op: BinOp,
        right: Box<Spanned<Expr>>,
    },
    /// Unary operation: `-x`, `not x`
    UnaryOp {
        op: UnaryOp,
        expr: Box<Spanned<Expr>>,
    },
}

/// Built-in OHLCV data sources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSource {
    Open,
    High,
    Low,
    Close,
    Volume,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Ne,
    Lt,
    Gt,
    Lte,
    Gte,
    // Logical
    And,
    Or,
}

impl BinOp {
    /// Returns the precedence level of this operator (higher = binds tighter)
    pub fn precedence(&self) -> u8 {
        match self {
            BinOp::Or => 1,
            BinOp::And => 2,
            BinOp::Eq | BinOp::Ne => 3,
            BinOp::Lt | BinOp::Gt | BinOp::Lte | BinOp::Gte => 4,
            BinOp::Add | BinOp::Sub => 5,
            BinOp::Mul | BinOp::Div | BinOp::Mod => 6,
        }
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Numeric negation: `-x`
    Neg,
    /// Logical negation: `not x`
    Not,
}

/// Literal values
#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

/// Entry statement for defining entry signals
#[derive(Debug, Clone)]
pub struct EntryStmt {
    pub direction: Direction,
    pub condition: Spanned<Expr>,
}

/// Trade direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Long,
    Short,
}

/// Exit statement for defining exit signals
#[derive(Debug, Clone)]
pub struct ExitStmt {
    pub target: ExitTarget,
    pub condition: Spanned<Expr>,
}

/// Exit target specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitTarget {
    Long,
    Short,
    All,
}

/// Plot statement for visualizing values
#[derive(Debug, Clone)]
pub struct PlotStmt {
    pub expr: Spanned<Expr>,
    pub color: Option<String>,
    pub panel: Option<String>,
    pub style: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binop_precedence() {
        // Multiplication binds tighter than addition
        assert!(BinOp::Mul.precedence() > BinOp::Add.precedence());
        // Comparison binds tighter than logical
        assert!(BinOp::Gt.precedence() > BinOp::And.precedence());
        // And binds tighter than Or
        assert!(BinOp::And.precedence() > BinOp::Or.precedence());
    }
}
