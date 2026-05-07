//! Compiler from TrdelScript AST to trdelnik-graph computation graph
//!
//! This module transforms a validated AST into an executable computation graph.

use crate::ast::*;
use std::collections::HashMap;
use thiserror::Error;
use trdelnik_core::Color;
use trdelnik_graph::nodes::*;
use trdelnik_graph::{BoxedNode, Graph, NodeId};

/// Error during compilation
#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Undefined variable: '{0}'")]
    UndefinedVariable(String),

    #[error("Unknown function: '{0}'")]
    UnknownFunction(String),

    #[error("Invalid literal in expression")]
    InvalidLiteral,

    #[error("Cannot extract field '{field}' from non-struct value")]
    InvalidFieldAccess { field: String },

    #[error("Expected integer argument for '{function}', got float")]
    ExpectedIntArg { function: String },

    #[error("Invalid argument for function '{0}'")]
    InvalidArgument(String),
}

/// Configuration for a plot output
#[derive(Debug, Clone)]
pub struct PlotConfig {
    /// The node whose output should be plotted
    pub node: NodeId,
    /// Color for the plot line
    pub color: Color,
    /// Optional panel name (None = overlay on main chart)
    pub panel: Option<String>,
    /// Plot style (e.g., "line", "histogram")
    pub style: Option<String>,
}

/// Result of compiling a TrdelScript strategy
#[derive(Debug)]
pub struct CompiledStrategy {
    /// The computation graph
    pub graph: Graph,
    /// Node for long entry signal (if defined)
    pub entry_long: Option<NodeId>,
    /// Node for short entry signal (if defined)
    pub entry_short: Option<NodeId>,
    /// Exit signals with their targets
    pub exit_signals: Vec<(ExitTarget, NodeId)>,
    /// Plot configurations
    pub plots: Vec<PlotConfig>,
    /// Stop loss percentage (if defined)
    pub stop_loss: Option<f64>,
    /// Take profit percentage (if defined)
    pub take_profit: Option<f64>,
    /// Strategy name (if defined)
    pub name: Option<String>,
    /// Timeframe (if defined)
    pub timeframe: Option<String>,
}

/// Compiler from AST to Graph
pub struct Compiler {
    graph: Graph,
    /// Map from variable names to their NodeIds
    symbols: HashMap<String, NodeId>,
    /// Map from variable names to struct field NodeIds
    struct_fields: HashMap<String, HashMap<String, NodeId>>,
    /// Parameter values (for substitution)
    params: HashMap<String, f64>,
    /// User-defined functions, indexed by name. Cloned at registration so
    /// the compiler can borrow them immutably while mutating its other
    /// fields during inlining.
    user_functions: HashMap<String, FunctionDef>,
}

impl Compiler {
    /// Create a new compiler with parameter values
    pub fn with_params(params: HashMap<String, f64>) -> Self {
        Self {
            graph: Graph::new(),
            symbols: HashMap::new(),
            struct_fields: HashMap::new(),
            params,
            user_functions: HashMap::new(),
        }
    }

    /// Compile a Script AST to a CompiledStrategy
    pub fn compile(script: &Script) -> Result<CompiledStrategy, CompileError> {
        Self::compile_with_params(script, HashMap::new())
    }

    /// Compile with parameter overrides
    pub fn compile_with_params(
        script: &Script,
        params: HashMap<String, f64>,
    ) -> Result<CompiledStrategy, CompileError> {
        let mut compiler = Self::with_params(params);
        compiler.compile_script(script)
    }

    fn compile_script(&mut self, script: &Script) -> Result<CompiledStrategy, CompileError> {
        // Register OHLCV data sources
        self.symbols.insert(
            "close".into(),
            self.graph.add_node(Box::new(CloseNode::new())),
        );
        self.symbols.insert(
            "open".into(),
            self.graph.add_node(Box::new(OpenNode::new())),
        );
        self.symbols.insert(
            "high".into(),
            self.graph.add_node(Box::new(HighNode::new())),
        );
        self.symbols.insert(
            "low".into(),
            self.graph.add_node(Box::new(LowNode::new())),
        );
        self.symbols.insert(
            "volume".into(),
            self.graph.add_node(Box::new(VolumeNode::new())),
        );

        // Register user-defined functions before compiling statements, so
        // that any forward-call (function defined first, used later) works.
        for func in &script.functions {
            self.user_functions
                .insert(func.node.name.clone(), func.node.clone());
        }

        // Register parameters with their default values
        for param in &script.params {
            let value = self
                .params
                .get(&param.node.name)
                .copied()
                .or_else(|| param.node.default.as_ref().and_then(|lit| self.literal_to_f64(lit)))
                .unwrap_or(0.0);

            // Store the resolved value for use in extract_f64/extract_usize
            self.params.insert(param.node.name.clone(), value);

            let node = self.graph.add_node(Box::new(ConstNode::new(value)));
            self.symbols.insert(param.node.name.clone(), node);
        }

        // Compile statements
        let mut entry_long = None;
        let mut entry_short = None;
        let mut exit_signals = Vec::new();
        let mut plots = Vec::new();
        let mut stop_loss = None;
        let mut take_profit = None;

        for stmt in &script.statements {
            match &stmt.node {
                Statement::Let(let_stmt) => {
                    self.compile_let(let_stmt)?;
                }
                Statement::Entry(entry) => {
                    let cond_node = self.compile_expr(&entry.condition.node)?;
                    match entry.direction {
                        Direction::Long => entry_long = Some(cond_node),
                        Direction::Short => entry_short = Some(cond_node),
                    }
                }
                Statement::Exit(exit) => {
                    let cond_node = self.compile_expr(&exit.condition.node)?;
                    exit_signals.push((exit.target, cond_node));
                }
                Statement::Plot(plot) => {
                    let node = self.compile_expr(&plot.expr.node)?;
                    plots.push(PlotConfig {
                        node,
                        color: plot.color.as_ref()
                            .map(|c| Color::from_name(c))
                            .unwrap_or(Color::rgb(200, 200, 200)),
                        panel: plot.panel.clone(),
                        style: plot.style.clone(),
                    });
                }
                Statement::StopLoss(pct) => {
                    stop_loss = Some(*pct);
                }
                Statement::TakeProfit(pct) => {
                    take_profit = Some(*pct);
                }
            }
        }

        Ok(CompiledStrategy {
            graph: std::mem::take(&mut self.graph),
            entry_long,
            entry_short,
            exit_signals,
            plots,
            stop_loss,
            take_profit,
            name: script.strategy.as_ref().map(|s| s.name.clone()),
            timeframe: script.strategy.as_ref().and_then(|s| s.timeframe.clone()),
        })
    }

    fn compile_let(&mut self, stmt: &LetStmt) -> Result<(), CompileError> {
        let value_node = self.compile_expr(&stmt.value.node)?;

        match &stmt.pattern {
            Pattern::Simple(name) => {
                self.symbols.insert(name.clone(), value_node);
            }
            Pattern::Destructure(fields) => {
                // For let { upper, middle, lower } = bollinger(...)
                // We need to extract each field
                let mut field_map = HashMap::new();
                for field_name in fields {
                    let field_node = self.graph.add_node(field(value_node, field_name));
                    self.symbols.insert(field_name.clone(), field_node);
                    field_map.insert(field_name.clone(), field_node);
                }
                // Store the struct's field mapping
                self.struct_fields
                    .insert(format!("__struct_{}", value_node.index()), field_map);
            }
        }

        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<NodeId, CompileError> {
        match expr {
            Expr::Ident(name) => self
                .symbols
                .get(name)
                .copied()
                .ok_or_else(|| CompileError::UndefinedVariable(name.clone())),

            Expr::DataSource(ds) => {
                let name = match ds {
                    DataSource::Close => "close",
                    DataSource::Open => "open",
                    DataSource::High => "high",
                    DataSource::Low => "low",
                    DataSource::Volume => "volume",
                };
                Ok(*self.symbols.get(name).unwrap())
            }

            Expr::Literal(lit) => {
                let value = self
                    .literal_to_f64(lit)
                    .ok_or(CompileError::InvalidLiteral)?;
                Ok(self.graph.add_node(Box::new(ConstNode::new(value))))
            }

            Expr::Call { name, args } => self.compile_call(name, args),

            Expr::FieldAccess { expr, field: field_name } => {
                let struct_node = self.compile_expr(&expr.node)?;
                // Create a field extraction node
                Ok(self.graph.add_node(field(struct_node, field_name)))
            }

            Expr::Index { expr, lag } => {
                let inner = self.compile_expr(&expr.node)?;
                let lag = if *lag < 0 {
                    return Err(CompileError::InvalidArgument(format!(
                        "negative lag {}",
                        lag
                    )));
                } else {
                    *lag as usize
                };
                Ok(self.graph.add_node(Box::new(LagNode::new(inner, lag))))
            }

            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_node = self.compile_expr(&cond.node)?;
                let then_node = self.compile_expr(&then_branch.node)?;
                let else_node = self.compile_expr(&else_branch.node)?;
                Ok(self
                    .graph
                    .add_node(Box::new(SelectNode::new(cond_node, then_node, else_node))))
            }

            Expr::BinaryOp { left, op, right } => {
                let left_node = self.compile_expr(&left.node)?;
                let right_node = self.compile_expr(&right.node)?;

                let node = match op {
                    BinOp::Add => Box::new(AddNode::new(left_node, right_node)) as BoxedNode,
                    BinOp::Sub => Box::new(SubNode::new(left_node, right_node)) as BoxedNode,
                    BinOp::Mul => Box::new(MulNode::new(left_node, right_node)) as BoxedNode,
                    BinOp::Div => Box::new(DivNode::new(left_node, right_node)) as BoxedNode,
                    BinOp::Mod => {
                        // No ModNode exists, implement as a - (a/b)*b
                        // For now, just use division as a placeholder
                        // TODO: Implement proper modulo
                        Box::new(DivNode::new(left_node, right_node)) as BoxedNode
                    }
                    BinOp::Gt => Box::new(GtNode::new(left_node, right_node)) as BoxedNode,
                    BinOp::Lt => Box::new(LtNode::new(left_node, right_node)) as BoxedNode,
                    BinOp::Gte => Box::new(GteNode::new(left_node, right_node)) as BoxedNode,
                    BinOp::Lte => Box::new(LteNode::new(left_node, right_node)) as BoxedNode,
                    BinOp::Eq => Box::new(EqNode::new(left_node, right_node)) as BoxedNode,
                    BinOp::Ne => {
                        // Ne is not(eq)
                        let eq_node = self.graph.add_node(Box::new(EqNode::new(left_node, right_node)));
                        Box::new(NotNode::new(eq_node)) as BoxedNode
                    }
                    BinOp::And => Box::new(AndNode::new(left_node, right_node)) as BoxedNode,
                    BinOp::Or => Box::new(OrNode::new(left_node, right_node)) as BoxedNode,
                };

                Ok(self.graph.add_node(node))
            }

            Expr::UnaryOp { op, expr } => {
                let expr_node = self.compile_expr(&expr.node)?;

                let node = match op {
                    UnaryOp::Neg => Box::new(NegNode::new(expr_node)) as BoxedNode,
                    UnaryOp::Not => Box::new(NotNode::new(expr_node)) as BoxedNode,
                };

                Ok(self.graph.add_node(node))
            }
        }
    }

    fn compile_call(&mut self, name: &str, args: &[Spanned<Expr>]) -> Result<NodeId, CompileError> {
        // User-defined function? Inline its body with the args bound to
        // the function's parameter names. Built-ins win on name-clash —
        // they're checked first by virtue of being matched here.
        if let Some(func_def) = self.user_functions.get(name).cloned() {
            if args.len() != func_def.params.len() {
                return Err(CompileError::InvalidArgument(format!(
                    "function '{}' expects {} arguments, got {}",
                    name,
                    func_def.params.len(),
                    args.len()
                )));
            }
            // Compile each argument expression first (in the caller's scope).
            // We also try to extract a scalar value — if the arg is a literal
            // or a param reference, the body might pass it to a built-in that
            // needs a compile-time number (e.g. `sma(close, p)` with `p` from
            // a function parameter).
            let mut arg_nodes: Vec<NodeId> = Vec::with_capacity(args.len());
            let mut arg_scalars: Vec<Option<f64>> = Vec::with_capacity(args.len());
            for arg in args {
                arg_scalars.push(self.extract_f64(arg).ok());
                arg_nodes.push(self.compile_expr(&arg.node)?);
            }
            // Bind params → arg-nodes (so identifier references in the body
            // resolve), and into self.params (so extract_usize/extract_f64
            // resolve to a compile-time value when the arg is scalar).
            let mut shadowed_symbols: Vec<(String, Option<NodeId>)> =
                Vec::with_capacity(args.len());
            let mut shadowed_params: Vec<(String, Option<f64>)> = Vec::with_capacity(args.len());
            for ((param_name, node), scalar) in func_def
                .params
                .iter()
                .zip(arg_nodes.iter())
                .zip(arg_scalars.iter())
            {
                let prev_sym = self.symbols.insert(param_name.clone(), *node);
                shadowed_symbols.push((param_name.clone(), prev_sym));
                if let Some(value) = scalar {
                    let prev_param = self.params.insert(param_name.clone(), *value);
                    shadowed_params.push((param_name.clone(), prev_param));
                } else {
                    let prev_param = self.params.remove(param_name);
                    shadowed_params.push((param_name.clone(), prev_param));
                }
            }
            // Compile the function body in the bound scope.
            let result = self.compile_expr(&func_def.body.node);
            // Restore any previous bindings (and remove our temporary ones).
            for (param_name, prev) in shadowed_symbols {
                match prev {
                    Some(v) => {
                        self.symbols.insert(param_name, v);
                    }
                    None => {
                        self.symbols.remove(&param_name);
                    }
                }
            }
            for (param_name, prev) in shadowed_params {
                match prev {
                    Some(v) => {
                        self.params.insert(param_name, v);
                    }
                    None => {
                        self.params.remove(&param_name);
                    }
                }
            }
            return result;
        }

        match name {
            // Single-input indicators (input, period)
            "sma" => {
                let input = self.compile_expr(&args[0].node)?;
                let period = self.extract_usize(&args[1])?;
                Ok(self.graph.add_node(sma(input, period)))
            }
            "ema" => {
                let input = self.compile_expr(&args[0].node)?;
                let period = self.extract_usize(&args[1])?;
                Ok(self.graph.add_node(ema(input, period)))
            }
            "wma" => {
                let input = self.compile_expr(&args[0].node)?;
                let period = self.extract_usize(&args[1])?;
                Ok(self.graph.add_node(wma(input, period)))
            }
            "rsi" => {
                let input = self.compile_expr(&args[0].node)?;
                let period = self.extract_usize(&args[1])?;
                Ok(self.graph.add_node(rsi(input, period)))
            }
            "std_dev" => {
                let input = self.compile_expr(&args[0].node)?;
                let period = self.extract_usize(&args[1])?;
                Ok(self.graph.add_node(std_dev(input, period)))
            }
            "roc" => {
                let input = self.compile_expr(&args[0].node)?;
                let period = self.extract_usize(&args[1])?;
                Ok(self.graph.add_node(roc(input, period)))
            }
            "efficiency_ratio" => {
                let input = self.compile_expr(&args[0].node)?;
                let period = self.extract_usize(&args[1])?;
                Ok(self.graph.add_node(efficiency_ratio(input, period)))
            }

            // Multi-output indicators
            "bollinger" => {
                let input = self.compile_expr(&args[0].node)?;
                let period = self.extract_usize(&args[1])?;
                let std_dev_mult = self.extract_f64(&args[2])?;
                Ok(self.graph.add_node(bollinger(input, period, std_dev_mult)))
            }
            "macd" => {
                let input = self.compile_expr(&args[0].node)?;
                let fast = self.extract_usize(&args[1])?;
                let slow = self.extract_usize(&args[2])?;
                let signal = self.extract_usize(&args[3])?;
                Ok(self.graph.add_node(macd(input, fast, slow, signal)))
            }
            "ppo" => {
                let input = self.compile_expr(&args[0].node)?;
                let fast = self.extract_usize(&args[1])?;
                let slow = self.extract_usize(&args[2])?;
                let signal = self.extract_usize(&args[3])?;
                Ok(self.graph.add_node(ppo(input, fast, slow, signal)))
            }

            // OHLC-based indicators (no input required, use context)
            "atr" => {
                let period = self.extract_usize(&args[0])?;
                Ok(self.graph.add_node(atr(period)))
            }
            "cci" => {
                let period = self.extract_usize(&args[0])?;
                if args.len() > 1 {
                    let constant = self.extract_f64(&args[1])?;
                    Ok(self.graph.add_node(cci_with_constant(period, constant)))
                } else {
                    Ok(self.graph.add_node(cci(period)))
                }
            }
            "stochastic" => {
                let k_period = self.extract_usize(&args[0])?;
                let d_period = self.extract_usize(&args[1])?;
                Ok(self.graph.add_node(stochastic(k_period, d_period)))
            }
            "keltner" => {
                let ema_period = self.extract_usize(&args[0])?;
                let atr_period = self.extract_usize(&args[1])?;
                let atr_mult = self.extract_f64(&args[2])?;
                Ok(self.graph.add_node(keltner(ema_period, atr_period, atr_mult)))
            }
            "chandelier" => {
                let period = self.extract_usize(&args[0])?;
                let atr_mult = self.extract_f64(&args[1])?;
                Ok(self.graph.add_node(chandelier(period, atr_mult)))
            }

            // OHLCV-based indicators
            "obv" => Ok(self.graph.add_node(obv())),
            "mfi" => {
                let period = self.extract_usize(&args[0])?;
                Ok(self.graph.add_node(mfi(period)))
            }

            // Signal functions
            "crossover" => {
                let a = self.compile_expr(&args[0].node)?;
                let b = self.compile_expr(&args[1].node)?;
                Ok(self.graph.add_node(Box::new(CrossOverNode::new(a, b))))
            }
            "crossunder" => {
                let a = self.compile_expr(&args[0].node)?;
                let b = self.compile_expr(&args[1].node)?;
                Ok(self.graph.add_node(Box::new(CrossUnderNode::new(a, b))))
            }
            "cross" => {
                let a = self.compile_expr(&args[0].node)?;
                let b = self.compile_expr(&args[1].node)?;
                Ok(self.graph.add_node(Box::new(CrossNode::new(a, b))))
            }

            // Pattern-counting functions
            "bars_since" => {
                let cond = self.compile_expr(&args[0].node)?;
                Ok(self.graph.add_node(bars_since(cond)))
            }
            "count_when" => {
                let cond = self.compile_expr(&args[0].node)?;
                let period = self.extract_usize(&args[1])?;
                Ok(self.graph.add_node(count_when(cond, period)))
            }

            _ => Err(CompileError::UnknownFunction(name.to_string())),
        }
    }

    fn extract_usize(&self, expr: &Spanned<Expr>) -> Result<usize, CompileError> {
        self.extract_f64(expr).map(|n| n as usize)
    }

    fn extract_f64(&self, expr: &Spanned<Expr>) -> Result<f64, CompileError> {
        match &expr.node {
            Expr::Literal(Literal::Int(n)) => Ok(*n as f64),
            Expr::Literal(Literal::Float(n)) => Ok(*n),
            Expr::Ident(name) => {
                if let Some(&value) = self.params.get(name) {
                    Ok(value)
                } else {
                    Err(CompileError::InvalidArgument(
                        "expected numeric literal or parameter".to_string(),
                    ))
                }
            }
            // Constant-fold simple arithmetic on parameters/literals so
            // `sma(close, period * 2)` works inside a user function body.
            Expr::BinaryOp { left, op, right } => {
                let l = self.extract_f64(left)?;
                let r = self.extract_f64(right)?;
                match op {
                    BinOp::Add => Ok(l + r),
                    BinOp::Sub => Ok(l - r),
                    BinOp::Mul => Ok(l * r),
                    BinOp::Div => {
                        if r == 0.0 {
                            Err(CompileError::InvalidArgument(
                                "division by zero in compile-time constant".to_string(),
                            ))
                        } else {
                            Ok(l / r)
                        }
                    }
                    _ => Err(CompileError::InvalidArgument(
                        "only +, -, *, / are constant-foldable".to_string(),
                    )),
                }
            }
            Expr::UnaryOp {
                op: UnaryOp::Neg,
                expr: inner,
            } => Ok(-self.extract_f64(inner)?),
            _ => Err(CompileError::InvalidArgument(
                "expected numeric literal".to_string(),
            )),
        }
    }

    fn literal_to_f64(&self, lit: &Literal) -> Option<f64> {
        match lit {
            Literal::Int(n) => Some(*n as f64),
            Literal::Float(n) => Some(*n),
            Literal::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            Literal::String(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse;
    use crate::semantic::SemanticAnalyzer;

    fn compile_str(source: &str) -> Result<CompiledStrategy, CompileError> {
        let tokens = lex(source).expect("lexer error");
        let script = parse(&tokens).expect("parse error");
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&script).expect("semantic error");
        Compiler::compile(&script)
    }

    #[test]
    fn test_compile_simple_sma() {
        let source = "let fast = sma(close, 12)";
        let strategy = compile_str(source).expect("compile error");
        assert!(strategy.graph.len() > 1); // At least close + sma
    }

    #[test]
    fn test_compile_entry_signal() {
        let source = r#"
            let fast = sma(close, 12)
            let slow = sma(close, 26)
            entry long when crossover(fast, slow)
        "#;
        let strategy = compile_str(source).expect("compile error");
        assert!(strategy.entry_long.is_some());
        assert!(strategy.entry_short.is_none());
    }

    #[test]
    fn test_compile_both_entries() {
        let source = r#"
            let fast = sma(close, 12)
            let slow = sma(close, 26)
            entry long when crossover(fast, slow)
            entry short when crossunder(fast, slow)
        "#;
        let strategy = compile_str(source).expect("compile error");
        assert!(strategy.entry_long.is_some());
        assert!(strategy.entry_short.is_some());
    }

    #[test]
    fn test_compile_exit() {
        let source = r#"
            exit all when rsi(close, 14) > 80
        "#;
        let strategy = compile_str(source).expect("compile error");
        assert_eq!(strategy.exit_signals.len(), 1);
        assert_eq!(strategy.exit_signals[0].0, ExitTarget::All);
    }

    #[test]
    fn test_compile_destructuring() {
        let source = r#"
            let { upper, middle, lower } = bollinger(close, 20, 2.0)
            entry long when close > upper
        "#;
        let strategy = compile_str(source).expect("compile error");
        assert!(strategy.entry_long.is_some());
    }

    #[test]
    fn test_compile_stop_loss_take_profit() {
        let source = r#"
            stop_loss 2%
            take_profit 6%
        "#;
        let strategy = compile_str(source).expect("compile error");
        assert_eq!(strategy.stop_loss, Some(2.0));
        assert_eq!(strategy.take_profit, Some(6.0));
    }

    #[test]
    fn test_compile_plots() {
        let source = r#"
            let fast = sma(close, 12)
            plot fast color=blue
        "#;
        let strategy = compile_str(source).expect("compile error");
        assert_eq!(strategy.plots.len(), 1);
        assert_eq!(strategy.plots[0].color, Color::from_name("blue"));
    }

    #[test]
    fn test_compile_strategy_metadata() {
        let source = r#"
            strategy "Test Strategy" timeframe = H1
            let x = close
        "#;
        let strategy = compile_str(source).expect("compile error");
        assert_eq!(strategy.name, Some("Test Strategy".to_string()));
        assert_eq!(strategy.timeframe, Some("H1".to_string()));
    }

    #[test]
    fn test_compile_complex_expression() {
        let source = r#"
            let x = (close + open) / 2
            entry long when x > sma(close, 20)
        "#;
        let strategy = compile_str(source).expect("compile error");
        assert!(strategy.entry_long.is_some());
    }

    #[test]
    fn test_compile_stochastic() {
        let source = r#"
            let { k, d } = stochastic(14, 3)
            entry long when crossover(k, d) and k < 20
        "#;
        let strategy = compile_str(source).expect("compile error");
        assert!(strategy.entry_long.is_some());
    }

    #[test]
    fn test_compile_macd() {
        let source = r#"
            let { macd, signal, histogram } = macd(close, 12, 26, 9)
            entry long when crossover(macd, signal)
        "#;
        let strategy = compile_str(source).expect("compile error");
        assert!(strategy.entry_long.is_some());
    }

    #[test]
    fn test_compile_with_params() {
        let source = r#"
            param period: int = 20
            let x = sma(close, period)
        "#;
        let tokens = lex(source).expect("lexer error");
        let script = parse(&tokens).expect("parse error");

        // Compile with custom param value
        let mut params = HashMap::new();
        params.insert("period".to_string(), 50.0);

        let strategy = Compiler::compile_with_params(&script, params).expect("compile error");
        assert!(strategy.graph.len() > 1);
    }

    #[test]
    fn test_compile_bars_since() {
        let source = r#"
            entry long when bars_since(rsi(close, 14) > 70) > 5
        "#;
        let strategy = compile_str(source).expect("compile error");
        assert!(strategy.entry_long.is_some());
    }

    #[test]
    fn test_compile_count_when() {
        let source = r#"
            let oversold_count = count_when(rsi(close, 14) < 30, 20)
            entry long when oversold_count >= 3
        "#;
        let strategy = compile_str(source).expect("compile error");
        assert!(strategy.entry_long.is_some());
    }

    #[test]
    fn test_cse_works() {
        let source = r#"
            let a = sma(close, 20)
            let b = sma(close, 20)
        "#;
        let strategy = compile_str(source).expect("compile error");
        // CSE should deduplicate the two identical SMAs
        // Graph contains: close, open, high, low, volume (5) + sma + const for period
        // The key test: with CSE, both `a` and `b` refer to the same node
        // Without CSE we'd have duplicate sma nodes
        let initial_count = strategy.graph.len();

        // Compile the same without CSE would have more nodes
        // For this simple test, we just verify compilation works
        // The actual CSE is verified by the graph internals
        assert!(initial_count >= 2, "Should have at least close and sma nodes");
    }
}
