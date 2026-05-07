//! Semantic analysis for TrdelScript
//!
//! This module performs semantic validation on the AST:
//! - Type checking
//! - Undefined variable detection
//! - Function signature validation
//! - Destructuring pattern validation

use crate::ast::*;
use std::collections::HashMap;

/// Information about known functions/indicators
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: &'static str,
    pub min_args: usize,
    pub max_args: usize,
    pub return_type: ValueType,
}

/// Value types in TrdelScript
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    /// Numeric value (f64)
    Number,
    /// Boolean value
    Bool,
    /// Struct with named fields
    Struct(Vec<String>),
}

/// Information about a symbol in scope
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub ty: ValueType,
    pub kind: SymbolKind,
    pub span: Span,
}

/// Kind of symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Param,
    Variable,
    DataSource,
}

/// Semantic error
#[derive(Debug, Clone)]
pub struct SemanticError {
    pub kind: SemanticErrorKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum SemanticErrorKind {
    UndefinedVariable(String),
    UndefinedFunction(String),
    WrongArgumentCount {
        function: String,
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    TypeMismatch {
        expected: String,
        actual: String,
    },
    InvalidDestructure {
        function: String,
        expected_fields: Vec<String>,
        actual_fields: Vec<String>,
    },
    DuplicateVariable(String),
    ConditionNotBoolean,
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            SemanticErrorKind::UndefinedVariable(name) => {
                write!(f, "Undefined variable: '{}'", name)
            }
            SemanticErrorKind::UndefinedFunction(name) => {
                write!(f, "Unknown function: '{}'", name)
            }
            SemanticErrorKind::WrongArgumentCount {
                function,
                expected_min,
                expected_max,
                actual,
            } => {
                if expected_min == expected_max {
                    write!(
                        f,
                        "Function '{}' expects {} arguments, got {}",
                        function, expected_min, actual
                    )
                } else {
                    write!(
                        f,
                        "Function '{}' expects {}-{} arguments, got {}",
                        function, expected_min, expected_max, actual
                    )
                }
            }
            SemanticErrorKind::TypeMismatch { expected, actual } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, actual)
            }
            SemanticErrorKind::InvalidDestructure {
                function,
                expected_fields,
                actual_fields,
            } => {
                write!(
                    f,
                    "Cannot destructure '{}' output. Expected fields {:?}, got {:?}",
                    function, expected_fields, actual_fields
                )
            }
            SemanticErrorKind::DuplicateVariable(name) => {
                write!(f, "Duplicate variable declaration: '{}'", name)
            }
            SemanticErrorKind::ConditionNotBoolean => {
                write!(f, "Condition must evaluate to a boolean")
            }
        }
    }
}

impl std::error::Error for SemanticError {}

/// Semantic analyzer for TrdelScript
pub struct SemanticAnalyzer {
    symbols: HashMap<String, SymbolInfo>,
    functions: HashMap<&'static str, FunctionInfo>,
    user_functions: HashMap<String, FunctionDef>,
    errors: Vec<SemanticError>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            symbols: HashMap::new(),
            functions: HashMap::new(),
            user_functions: HashMap::new(),
            errors: Vec::new(),
        };
        analyzer.register_builtins();
        analyzer
    }

    fn register_builtins(&mut self) {
        // Register data sources
        let data_sources = ["open", "high", "low", "close", "volume"];
        for name in data_sources {
            self.symbols.insert(
                name.to_string(),
                SymbolInfo {
                    ty: ValueType::Number,
                    kind: SymbolKind::DataSource,
                    span: 0..0,
                },
            );
        }

        // Register built-in functions
        let functions = [
            // Single-input indicators
            FunctionInfo {
                name: "sma",
                min_args: 2,
                max_args: 2,
                return_type: ValueType::Number,
            },
            FunctionInfo {
                name: "ema",
                min_args: 2,
                max_args: 2,
                return_type: ValueType::Number,
            },
            FunctionInfo {
                name: "wma",
                min_args: 2,
                max_args: 2,
                return_type: ValueType::Number,
            },
            FunctionInfo {
                name: "rsi",
                min_args: 2,
                max_args: 2,
                return_type: ValueType::Number,
            },
            FunctionInfo {
                name: "std_dev",
                min_args: 2,
                max_args: 2,
                return_type: ValueType::Number,
            },
            FunctionInfo {
                name: "roc",
                min_args: 2,
                max_args: 2,
                return_type: ValueType::Number,
            },
            FunctionInfo {
                name: "efficiency_ratio",
                min_args: 2,
                max_args: 2,
                return_type: ValueType::Number,
            },
            // Multi-output indicators
            FunctionInfo {
                name: "bollinger",
                min_args: 3,
                max_args: 3,
                return_type: ValueType::Struct(vec![
                    "upper".into(),
                    "middle".into(),
                    "lower".into(),
                ]),
            },
            FunctionInfo {
                name: "macd",
                min_args: 4,
                max_args: 4,
                return_type: ValueType::Struct(vec![
                    "macd".into(),
                    "signal".into(),
                    "histogram".into(),
                ]),
            },
            FunctionInfo {
                name: "ppo",
                min_args: 4,
                max_args: 4,
                return_type: ValueType::Struct(vec![
                    "ppo".into(),
                    "signal".into(),
                    "histogram".into(),
                ]),
            },
            // OHLC-based indicators (no input required)
            FunctionInfo {
                name: "atr",
                min_args: 1,
                max_args: 1,
                return_type: ValueType::Number,
            },
            FunctionInfo {
                name: "cci",
                min_args: 1,
                max_args: 2,
                return_type: ValueType::Number,
            },
            FunctionInfo {
                name: "stochastic",
                min_args: 2,
                max_args: 2,
                return_type: ValueType::Struct(vec!["k".into(), "d".into()]),
            },
            FunctionInfo {
                name: "keltner",
                min_args: 3,
                max_args: 3,
                return_type: ValueType::Struct(vec![
                    "upper".into(),
                    "middle".into(),
                    "lower".into(),
                ]),
            },
            FunctionInfo {
                name: "chandelier",
                min_args: 2,
                max_args: 2,
                return_type: ValueType::Struct(vec!["long_exit".into(), "short_exit".into()]),
            },
            // OHLCV-based indicators
            FunctionInfo {
                name: "obv",
                min_args: 0,
                max_args: 0,
                return_type: ValueType::Number,
            },
            FunctionInfo {
                name: "mfi",
                min_args: 1,
                max_args: 1,
                return_type: ValueType::Number,
            },
            // Signal functions
            FunctionInfo {
                name: "crossover",
                min_args: 2,
                max_args: 2,
                return_type: ValueType::Bool,
            },
            FunctionInfo {
                name: "crossunder",
                min_args: 2,
                max_args: 2,
                return_type: ValueType::Bool,
            },
            FunctionInfo {
                name: "cross",
                min_args: 2,
                max_args: 2,
                return_type: ValueType::Bool,
            },
            // Pattern-counting functions (O(1) per bar, not loop-based)
            FunctionInfo {
                name: "bars_since",
                min_args: 1,
                max_args: 1,
                return_type: ValueType::Number,
            },
            FunctionInfo {
                name: "count_when",
                min_args: 2,
                max_args: 2,
                return_type: ValueType::Number,
            },
        ];

        for func in functions {
            self.functions.insert(func.name, func);
        }
    }

    /// Analyze a parsed script for semantic errors
    pub fn analyze(&mut self, script: &Script) -> Result<(), Vec<SemanticError>> {
        // Register parameters
        for param in &script.params {
            self.register_param(&param.node, param.span.clone());
        }

        // Register user-defined functions and check their bodies in a
        // scope with their params bound. Done before analyzing statements
        // so forward references resolve.
        for func in &script.functions {
            self.register_user_function(&func.node, func.span.clone());
        }
        for func in &script.functions {
            self.analyze_function_body(&func.node);
        }

        // Analyze statements in order
        for stmt in &script.statements {
            self.analyze_statement(&stmt.node, stmt.span.clone());
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    fn register_user_function(&mut self, func: &FunctionDef, span: Span) {
        if self.functions.contains_key(func.name.as_str())
            || self.user_functions.contains_key(&func.name)
        {
            self.errors.push(SemanticError {
                kind: SemanticErrorKind::DuplicateVariable(func.name.clone()),
                span,
            });
            return;
        }
        self.user_functions.insert(func.name.clone(), func.clone());
    }

    fn analyze_function_body(&mut self, func: &FunctionDef) {
        // Bind params as Number-typed Variables for the duration of body
        // analysis; restore prior bindings afterwards.
        let mut shadowed: Vec<(String, Option<SymbolInfo>)> = Vec::new();
        for param_name in &func.params {
            let prev = self.symbols.insert(
                param_name.clone(),
                SymbolInfo {
                    ty: ValueType::Number,
                    kind: SymbolKind::Variable,
                    span: 0..0,
                },
            );
            shadowed.push((param_name.clone(), prev));
        }
        self.analyze_expr(&func.body.node, func.body.span.clone());
        for (name, prev) in shadowed {
            match prev {
                Some(v) => {
                    self.symbols.insert(name, v);
                }
                None => {
                    self.symbols.remove(&name);
                }
            }
        }
    }

    fn register_param(&mut self, param: &ParamDecl, span: Span) {
        let ty = match param.ty {
            ParamType::Int | ParamType::Float => ValueType::Number,
            ParamType::Bool => ValueType::Bool,
        };

        if self.symbols.contains_key(&param.name) {
            self.errors.push(SemanticError {
                kind: SemanticErrorKind::DuplicateVariable(param.name.clone()),
                span,
            });
        } else {
            self.symbols.insert(
                param.name.clone(),
                SymbolInfo {
                    ty,
                    kind: SymbolKind::Param,
                    span,
                },
            );
        }
    }

    fn analyze_statement(&mut self, stmt: &Statement, span: Span) {
        match stmt {
            Statement::Let(let_stmt) => self.analyze_let(let_stmt, span),
            Statement::Entry(entry) => self.analyze_entry(entry),
            Statement::Exit(exit) => self.analyze_exit(exit),
            Statement::Plot(plot) => {
                self.analyze_expr(&plot.expr.node, plot.expr.span.clone());
            }
            Statement::StopLoss(_) | Statement::TakeProfit(_) => {}
        }
    }

    fn analyze_let(&mut self, let_stmt: &LetStmt, span: Span) {
        let value_type = self.analyze_expr(&let_stmt.value.node, let_stmt.value.span.clone());

        match &let_stmt.pattern {
            Pattern::Simple(name) => {
                if let Some(ty) = value_type {
                    self.symbols.insert(
                        name.clone(),
                        SymbolInfo {
                            ty,
                            kind: SymbolKind::Variable,
                            span,
                        },
                    );
                }
            }
            Pattern::Destructure(fields) => {
                // Validate destructuring against the value type
                if let Some(ValueType::Struct(expected_fields)) = &value_type {
                    // Check that all requested fields exist
                    for field in fields {
                        if !expected_fields.contains(field) {
                            self.errors.push(SemanticError {
                                kind: SemanticErrorKind::InvalidDestructure {
                                    function: "expression".to_string(),
                                    expected_fields: expected_fields.clone(),
                                    actual_fields: fields.clone(),
                                },
                                span: span.clone(),
                            });
                            return;
                        }
                        // Register each field as a Number variable
                        self.symbols.insert(
                            field.clone(),
                            SymbolInfo {
                                ty: ValueType::Number,
                                kind: SymbolKind::Variable,
                                span: span.clone(),
                            },
                        );
                    }
                } else if value_type.is_some() {
                    self.errors.push(SemanticError {
                        kind: SemanticErrorKind::TypeMismatch {
                            expected: "struct".to_string(),
                            actual: "non-struct".to_string(),
                        },
                        span,
                    });
                }
            }
        }
    }

    fn analyze_entry(&mut self, entry: &EntryStmt) {
        let ty = self.analyze_expr(&entry.condition.node, entry.condition.span.clone());
        if let Some(ty) = ty {
            if ty != ValueType::Bool {
                self.errors.push(SemanticError {
                    kind: SemanticErrorKind::ConditionNotBoolean,
                    span: entry.condition.span.clone(),
                });
            }
        }
    }

    fn analyze_exit(&mut self, exit: &ExitStmt) {
        let ty = self.analyze_expr(&exit.condition.node, exit.condition.span.clone());
        if let Some(ty) = ty {
            if ty != ValueType::Bool {
                self.errors.push(SemanticError {
                    kind: SemanticErrorKind::ConditionNotBoolean,
                    span: exit.condition.span.clone(),
                });
            }
        }
    }

    fn analyze_expr(&mut self, expr: &Expr, span: Span) -> Option<ValueType> {
        match expr {
            Expr::Literal(lit) => Some(match lit {
                Literal::Int(_) | Literal::Float(_) => ValueType::Number,
                Literal::Bool(_) => ValueType::Bool,
                Literal::String(_) => ValueType::Number, // Strings are not really used in expressions
            }),
            Expr::Ident(name) => {
                if let Some(info) = self.symbols.get(name) {
                    Some(info.ty.clone())
                } else {
                    self.errors.push(SemanticError {
                        kind: SemanticErrorKind::UndefinedVariable(name.clone()),
                        span,
                    });
                    None
                }
            }
            Expr::DataSource(_) => Some(ValueType::Number),
            Expr::Call { name, args } => self.analyze_call(name, args, span),
            Expr::FieldAccess { expr, field } => {
                let expr_type = self.analyze_expr(&expr.node, expr.span.clone());
                if let Some(ValueType::Struct(fields)) = expr_type {
                    if fields.contains(field) {
                        Some(ValueType::Number)
                    } else {
                        self.errors.push(SemanticError {
                            kind: SemanticErrorKind::UndefinedVariable(format!(
                                "field '{}'",
                                field
                            )),
                            span,
                        });
                        None
                    }
                } else {
                    // If not a struct, it's an error
                    if expr_type.is_some() {
                        self.errors.push(SemanticError {
                            kind: SemanticErrorKind::TypeMismatch {
                                expected: "struct".to_string(),
                                actual: "non-struct".to_string(),
                            },
                            span,
                        });
                    }
                    None
                }
            }
            Expr::BinaryOp { left, op, right } => {
                self.analyze_expr(&left.node, left.span.clone());
                self.analyze_expr(&right.node, right.span.clone());
                // Return type depends on operator
                match op {
                    BinOp::Add
                    | BinOp::Sub
                    | BinOp::Mul
                    | BinOp::Div
                    | BinOp::Mod => Some(ValueType::Number),
                    BinOp::Eq
                    | BinOp::Ne
                    | BinOp::Lt
                    | BinOp::Gt
                    | BinOp::Lte
                    | BinOp::Gte
                    | BinOp::And
                    | BinOp::Or => Some(ValueType::Bool),
                }
            }
            Expr::UnaryOp { op, expr } => {
                let _ty = self.analyze_expr(&expr.node, expr.span.clone());
                match op {
                    UnaryOp::Neg => Some(ValueType::Number),
                    UnaryOp::Not => Some(ValueType::Bool),
                }
            }
            Expr::Index { expr, lag } => {
                let inner_ty = self.analyze_expr(&expr.node, expr.span.clone());
                if *lag < 0 {
                    self.errors.push(SemanticError {
                        kind: SemanticErrorKind::TypeMismatch {
                            expected: "non-negative integer".to_string(),
                            actual: format!("{}", lag),
                        },
                        span,
                    });
                }
                // Lagged value has the same type as the underlying expression.
                inner_ty
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_ty = self.analyze_expr(&cond.node, cond.span.clone());
                if let Some(ty) = &cond_ty {
                    if ty != &ValueType::Bool {
                        self.errors.push(SemanticError {
                            kind: SemanticErrorKind::ConditionNotBoolean,
                            span: cond.span.clone(),
                        });
                    }
                }
                let then_ty = self.analyze_expr(&then_branch.node, then_branch.span.clone());
                let else_ty = self.analyze_expr(&else_branch.node, else_branch.span.clone());
                // Pick a type to surface upward. If both branches agree, use
                // that. Otherwise default to Number — branches with mismatched
                // types still compile (SelectNode is value-agnostic), and the
                // user's explicit if/else expresses the intent.
                match (then_ty, else_ty) {
                    (Some(a), Some(b)) if a == b => Some(a),
                    _ => Some(ValueType::Number),
                }
            }
        }
    }

    fn analyze_call(&mut self, name: &str, args: &[Spanned<Expr>], span: Span) -> Option<ValueType> {
        // Analyze all arguments first
        for arg in args {
            self.analyze_expr(&arg.node, arg.span.clone());
        }

        // User function? Check arity and infer the return type as Number
        // (we don't analyze the body again here — it was analyzed when
        // registered, so we only need to validate the call shape).
        if let Some(func) = self.user_functions.get(name).cloned() {
            let arg_count = args.len();
            if arg_count != func.params.len() {
                self.errors.push(SemanticError {
                    kind: SemanticErrorKind::WrongArgumentCount {
                        function: name.to_string(),
                        expected_min: func.params.len(),
                        expected_max: func.params.len(),
                        actual: arg_count,
                    },
                    span,
                });
            }
            return Some(ValueType::Number);
        }

        // Built-in function?
        if let Some(func) = self.functions.get(name) {
            let arg_count = args.len();
            if arg_count < func.min_args || arg_count > func.max_args {
                self.errors.push(SemanticError {
                    kind: SemanticErrorKind::WrongArgumentCount {
                        function: name.to_string(),
                        expected_min: func.min_args,
                        expected_max: func.max_args,
                        actual: arg_count,
                    },
                    span,
                });
            }
            Some(func.return_type.clone())
        } else {
            self.errors.push(SemanticError {
                kind: SemanticErrorKind::UndefinedFunction(name.to_string()),
                span,
            });
            None
        }
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse;

    fn analyze_str(source: &str) -> Result<(), Vec<SemanticError>> {
        let tokens = lex(source).expect("lexer error");
        let script = parse(&tokens).expect("parse error");
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&script)
    }

    #[test]
    fn test_valid_simple_script() {
        let source = r#"
            let fast = sma(close, 12)
            let slow = sma(close, 26)
            entry long when crossover(fast, slow)
        "#;
        assert!(analyze_str(source).is_ok());
    }

    #[test]
    fn test_undefined_variable() {
        let source = "entry long when crossover(undefined_var, close)";
        let result = analyze_str(source);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(matches!(
            &errors[0].kind,
            SemanticErrorKind::UndefinedVariable(name) if name == "undefined_var"
        ));
    }

    #[test]
    fn test_undefined_function() {
        let source = "let x = unknown_func(close, 10)";
        let result = analyze_str(source);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(matches!(
            &errors[0].kind,
            SemanticErrorKind::UndefinedFunction(name) if name == "unknown_func"
        ));
    }

    #[test]
    fn test_wrong_argument_count() {
        let source = "let x = sma(close)"; // sma needs 2 args
        let result = analyze_str(source);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(matches!(
            &errors[0].kind,
            SemanticErrorKind::WrongArgumentCount { function, .. } if function == "sma"
        ));
    }

    #[test]
    fn test_valid_destructuring() {
        let source = r#"
            let { upper, middle, lower } = bollinger(close, 20, 2.0)
            entry long when close > upper
        "#;
        assert!(analyze_str(source).is_ok());
    }

    #[test]
    fn test_invalid_destructuring_field() {
        let source = r#"
            let { invalid_field } = bollinger(close, 20, 2.0)
        "#;
        let result = analyze_str(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_params_in_scope() {
        let source = r#"
            param period: int = 20
            let x = sma(close, period)
        "#;
        assert!(analyze_str(source).is_ok());
    }

    #[test]
    fn test_stochastic_destructuring() {
        let source = r#"
            let { k, d } = stochastic(14, 3)
            entry long when k > d
        "#;
        assert!(analyze_str(source).is_ok());
    }

    #[test]
    fn test_macd_destructuring() {
        let source = r#"
            let { macd, signal, histogram } = macd(close, 12, 26, 9)
            entry long when macd > signal
        "#;
        assert!(analyze_str(source).is_ok());
    }

    #[test]
    fn test_condition_must_be_boolean() {
        let source = "entry long when sma(close, 20)"; // sma returns Number, not Bool
        let result = analyze_str(source);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(matches!(
            &errors[0].kind,
            SemanticErrorKind::ConditionNotBoolean
        ));
    }

    #[test]
    fn test_data_sources_available() {
        let source = r#"
            let o = open
            let h = high
            let l = low
            let c = close
            let v = volume
        "#;
        assert!(analyze_str(source).is_ok());
    }
}
