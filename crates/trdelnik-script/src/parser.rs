//! Chumsky-based parser for TrdelScript
//!
//! This module parses tokens from the lexer into an Abstract Syntax Tree.

use crate::ast::*;
use crate::lexer::{OrderedFloat, Token};
use chumsky::prelude::*;

/// Parse a list of tokens into a Script AST.
pub fn parse(tokens: &[(Token, Span)]) -> Result<Script, Vec<Simple<Token>>> {
    let len = tokens.last().map(|(_, s)| s.end).unwrap_or(0);
    let stream = chumsky::Stream::from_iter(len..len + 1, tokens.iter().cloned());
    parser().parse(stream)
}

/// Main parser combinator for TrdelScript
fn parser() -> impl Parser<Token, Script, Error = Simple<Token>> {
    let script = strategy_decl()
        .or_not()
        .then(param_decl().repeated())
        .then(statement().repeated())
        .map(|((strategy, params), statements)| Script {
            strategy,
            params,
            statements,
        });

    script.then_ignore(end())
}

/// Parser for strategy declaration
fn strategy_decl() -> impl Parser<Token, StrategyDecl, Error = Simple<Token>> {
    let name = select! { Token::String(s) => s }.labelled("strategy name");

    let timeframe_clause = just(Token::Timeframe)
        .ignore_then(just(Token::Eq))
        .ignore_then(select! { Token::Ident(s) => s })
        .or_not();

    just(Token::Strategy)
        .ignore_then(name)
        .then(timeframe_clause)
        .map_with_span(|(name, timeframe), span| StrategyDecl {
            name,
            timeframe,
            span,
        })
}

/// Parser for parameter declaration
fn param_decl() -> impl Parser<Token, Spanned<ParamDecl>, Error = Simple<Token>> {
    let ident = select! { Token::Ident(name) => name }.labelled("parameter name");

    let param_type = select! {
        Token::TypeInt => ParamType::Int,
        Token::TypeFloat => ParamType::Float,
        Token::TypeBool => ParamType::Bool,
    }
    .labelled("type");

    let default_value = just(Token::Eq).ignore_then(literal()).or_not();

    just(Token::Param)
        .ignore_then(ident)
        .then_ignore(just(Token::Colon))
        .then(param_type)
        .then(default_value)
        .map_with_span(|((name, ty), default), span| {
            Spanned::new(ParamDecl { name, ty, default }, span)
        })
}

/// Parser for literal values
fn literal() -> impl Parser<Token, Literal, Error = Simple<Token>> + Clone {
    select! {
        Token::Int(n) => Literal::Int(n),
        Token::Float(OrderedFloat(n)) => Literal::Float(n),
        Token::True => Literal::Bool(true),
        Token::False => Literal::Bool(false),
        Token::String(s) => Literal::String(s),
    }
    .labelled("literal")
}

/// Parser for expressions with proper precedence handling
fn expr() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    recursive(|expr| {
        // Atom: literals, data sources, calls, identifiers, parenthesized
        let atom = atom_expr(expr.clone());

        // Field access: expr.field
        let field_access = atom
            .clone()
            .then(
                just(Token::Dot)
                    .ignore_then(select! { Token::Ident(s) => s })
                    .repeated(),
            )
            .foldl(|expr, field| {
                let span = expr.span.start..expr.span.end + field.len() + 1;
                Spanned::new(
                    Expr::FieldAccess {
                        expr: Box::new(expr),
                        field,
                    },
                    span,
                )
            });

        // Unary operators
        let unary = just(Token::Minus)
            .map(|_| UnaryOp::Neg)
            .or(just(Token::Not).map(|_| UnaryOp::Not))
            .map_with_span(|op, span: Span| (op, span))
            .repeated()
            .then(field_access)
            .foldr(|(op, op_span), expr| {
                let span = op_span.start..expr.span.end;
                Spanned::new(
                    Expr::UnaryOp {
                        op,
                        expr: Box::new(expr),
                    },
                    span,
                )
            });

        // Build binary operators with precedence
        let product = binary_op(
            unary,
            select! {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
            },
        );

        let sum = binary_op(
            product,
            select! {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
            },
        );

        let comparison = binary_op(
            sum,
            select! {
                Token::Lt => BinOp::Lt,
                Token::Gt => BinOp::Gt,
                Token::Lte => BinOp::Lte,
                Token::Gte => BinOp::Gte,
            },
        );

        let equality = binary_op(
            comparison,
            select! {
                Token::EqEq => BinOp::Eq,
                Token::NotEq => BinOp::Ne,
            },
        );

        let logical_and = binary_op(equality, just(Token::And).to(BinOp::And));

        binary_op(logical_and, just(Token::Or).to(BinOp::Or))
    })
}

/// Helper to build binary operators with left associativity
fn binary_op<P, O>(
    operand: P,
    op: O,
) -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone
where
    P: Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone,
    O: Parser<Token, BinOp, Error = Simple<Token>> + Clone,
{
    operand
        .clone()
        .then(op.then(operand).repeated())
        .foldl(|left, (op, right)| {
            let span = left.span.start..right.span.end;
            Spanned::new(
                Expr::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            )
        })
}

/// Parser for atomic expressions (highest precedence)
fn atom_expr<P>(expr: P) -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone
where
    P: Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone,
{
    let literal_expr = literal().map_with_span(|lit, span| Spanned::new(Expr::Literal(lit), span));

    let data_source = select! {
        Token::Open => DataSource::Open,
        Token::High => DataSource::High,
        Token::Low => DataSource::Low,
        Token::Close => DataSource::Close,
        Token::Volume => DataSource::Volume,
    }
    .map_with_span(|ds, span| Spanned::new(Expr::DataSource(ds), span));

    let ident = select! { Token::Ident(s) => s };

    // Function call: name(args...)
    let call = ident
        .clone()
        .then(
            expr.clone()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .map_with_span(|(name, args), span| Spanned::new(Expr::Call { name, args }, span));

    // Simple identifier (not a call)
    let ident_expr = ident.map_with_span(|name, span| Spanned::new(Expr::Ident(name), span));

    // Parenthesized expression
    let paren_expr = expr
        .clone()
        .delimited_by(just(Token::LParen), just(Token::RParen));

    // Order matters: try call first, then data sources, literals, idents
    choice((literal_expr, data_source, call, ident_expr, paren_expr))
}

/// Parser for patterns (simple or destructuring)
fn pattern() -> impl Parser<Token, Pattern, Error = Simple<Token>> {
    let simple = select! { Token::Ident(name) => Pattern::Simple(name) };

    let destructure = select! { Token::Ident(name) => name }
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .map(Pattern::Destructure);

    destructure.or(simple)
}

/// Parser for let statements
fn let_stmt() -> impl Parser<Token, Statement, Error = Simple<Token>> {
    just(Token::Let)
        .ignore_then(pattern())
        .then_ignore(just(Token::Eq))
        .then(expr())
        .map(|(pattern, value)| Statement::Let(LetStmt { pattern, value }))
}

/// Parser for entry statements
fn entry_stmt() -> impl Parser<Token, Statement, Error = Simple<Token>> {
    let direction = select! {
        Token::Long => Direction::Long,
        Token::Short => Direction::Short,
    };

    just(Token::Entry)
        .ignore_then(direction)
        .then_ignore(just(Token::When))
        .then(expr())
        .map(|(direction, condition)| Statement::Entry(EntryStmt { direction, condition }))
}

/// Parser for exit statements
fn exit_stmt() -> impl Parser<Token, Statement, Error = Simple<Token>> {
    let target = select! {
        Token::Long => ExitTarget::Long,
        Token::Short => ExitTarget::Short,
        Token::All => ExitTarget::All,
    };

    just(Token::Exit)
        .ignore_then(target)
        .then_ignore(just(Token::When))
        .then(expr())
        .map(|(target, condition)| Statement::Exit(ExitStmt { target, condition }))
}

/// Parser for stop_loss statement
fn stop_loss() -> impl Parser<Token, Statement, Error = Simple<Token>> {
    let number = select! {
        Token::Int(n) => n as f64,
        Token::Float(OrderedFloat(n)) => n,
    };

    just(Token::StopLoss)
        .ignore_then(number)
        .then_ignore(just(Token::Percent).or_not())
        .map(Statement::StopLoss)
}

/// Parser for take_profit statement
fn take_profit() -> impl Parser<Token, Statement, Error = Simple<Token>> {
    let number = select! {
        Token::Int(n) => n as f64,
        Token::Float(OrderedFloat(n)) => n,
    };

    just(Token::TakeProfit)
        .ignore_then(number)
        .then_ignore(just(Token::Percent).or_not())
        .map(Statement::TakeProfit)
}

/// Parser for plot statements
fn plot_stmt() -> impl Parser<Token, Statement, Error = Simple<Token>> {
    let color_opt = just(Token::Color)
        .ignore_then(just(Token::Eq))
        .ignore_then(select! { Token::Ident(s) => s })
        .or_not();

    let panel_opt = just(Token::Panel)
        .ignore_then(just(Token::Eq))
        .ignore_then(select! { Token::String(s) => s })
        .or_not();

    let style_opt = just(Token::Style)
        .ignore_then(just(Token::Eq))
        .ignore_then(select! { Token::Ident(s) => s })
        .or_not();

    just(Token::Plot)
        .ignore_then(expr())
        .then(color_opt)
        .then(panel_opt)
        .then(style_opt)
        .map(|(((expr, color), panel), style)| {
            Statement::Plot(PlotStmt {
                expr,
                color,
                panel,
                style,
            })
        })
}

/// Parser for all statement types
fn statement() -> impl Parser<Token, Spanned<Statement>, Error = Simple<Token>> {
    choice((
        let_stmt(),
        entry_stmt(),
        exit_stmt(),
        stop_loss(),
        take_profit(),
        plot_stmt(),
    ))
    .map_with_span(Spanned::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    fn parse_str(source: &str) -> Result<Script, Vec<Simple<Token>>> {
        let tokens = lex(source).expect("lexer error");
        parse(&tokens)
    }

    #[test]
    fn test_parse_strategy() {
        let source = r#"strategy "SMA Crossover""#;
        let script = parse_str(source).expect("parse error");
        assert!(script.strategy.is_some());
        assert_eq!(script.strategy.as_ref().unwrap().name, "SMA Crossover");
    }

    #[test]
    fn test_parse_strategy_with_timeframe() {
        let source = r#"strategy "Test" timeframe = H1"#;
        let script = parse_str(source).expect("parse error");
        assert!(script.strategy.is_some());
        let strat = script.strategy.as_ref().unwrap();
        assert_eq!(strat.name, "Test");
        assert_eq!(strat.timeframe, Some("H1".to_string()));
    }

    #[test]
    fn test_parse_param() {
        let source = "param fast_period: int = 12";
        let script = parse_str(source).expect("parse error");
        assert_eq!(script.params.len(), 1);
        let param = &script.params[0].node;
        assert_eq!(param.name, "fast_period");
        assert_eq!(param.ty, ParamType::Int);
        assert!(matches!(param.default, Some(Literal::Int(12))));
    }

    #[test]
    fn test_parse_simple_let() {
        let source = "let fast = sma(close, 12)";
        let script = parse_str(source).expect("parse error");
        assert_eq!(script.statements.len(), 1);
        match &script.statements[0].node {
            Statement::Let(let_stmt) => {
                assert!(matches!(&let_stmt.pattern, Pattern::Simple(name) if name == "fast"));
                assert!(matches!(&let_stmt.value.node, Expr::Call { name, .. } if name == "sma"));
            }
            _ => panic!("expected let statement"),
        }
    }

    #[test]
    fn test_parse_destructuring_let() {
        let source = "let { upper, middle, lower } = bollinger(close, 20, 2.0)";
        let script = parse_str(source).expect("parse error");
        assert_eq!(script.statements.len(), 1);
        match &script.statements[0].node {
            Statement::Let(let_stmt) => {
                match &let_stmt.pattern {
                    Pattern::Destructure(fields) => {
                        assert_eq!(fields, &["upper", "middle", "lower"]);
                    }
                    _ => panic!("expected destructuring pattern"),
                }
            }
            _ => panic!("expected let statement"),
        }
    }

    #[test]
    fn test_parse_entry() {
        let source = "entry long when crossover(fast, slow)";
        let script = parse_str(source).expect("parse error");
        assert_eq!(script.statements.len(), 1);
        match &script.statements[0].node {
            Statement::Entry(entry) => {
                assert_eq!(entry.direction, Direction::Long);
            }
            _ => panic!("expected entry statement"),
        }
    }

    #[test]
    fn test_parse_exit() {
        let source = "exit all when rsi(close, 14) > 80";
        let script = parse_str(source).expect("parse error");
        assert_eq!(script.statements.len(), 1);
        match &script.statements[0].node {
            Statement::Exit(exit) => {
                assert_eq!(exit.target, ExitTarget::All);
            }
            _ => panic!("expected exit statement"),
        }
    }

    #[test]
    fn test_parse_stop_loss() {
        let source = "stop_loss 2%";
        let script = parse_str(source).expect("parse error");
        assert_eq!(script.statements.len(), 1);
        match &script.statements[0].node {
            Statement::StopLoss(pct) => {
                assert!((pct - 2.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected stop_loss statement"),
        }
    }

    #[test]
    fn test_parse_take_profit() {
        let source = "take_profit 6%";
        let script = parse_str(source).expect("parse error");
        assert_eq!(script.statements.len(), 1);
        match &script.statements[0].node {
            Statement::TakeProfit(pct) => {
                assert!((pct - 6.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected take_profit statement"),
        }
    }

    #[test]
    fn test_parse_plot() {
        let source = r#"plot fast_sma color=blue"#;
        let script = parse_str(source).expect("parse error");
        assert_eq!(script.statements.len(), 1);
        match &script.statements[0].node {
            Statement::Plot(plot) => {
                assert_eq!(plot.color, Some("blue".to_string()));
            }
            _ => panic!("expected plot statement"),
        }
    }

    #[test]
    fn test_parse_binary_expr() {
        let source = "let x = a + b * c";
        let script = parse_str(source).expect("parse error");
        match &script.statements[0].node {
            Statement::Let(let_stmt) => {
                // Should be Add(a, Mul(b, c)) due to precedence
                match &let_stmt.value.node {
                    Expr::BinaryOp { op, .. } => {
                        assert_eq!(*op, BinOp::Add);
                    }
                    _ => panic!("expected binary op"),
                }
            }
            _ => panic!("expected let statement"),
        }
    }

    #[test]
    fn test_parse_comparison_and_logic() {
        let source = "entry long when fast > slow and rsi < 70";
        let script = parse_str(source).expect("parse error");
        match &script.statements[0].node {
            Statement::Entry(entry) => {
                match &entry.condition.node {
                    Expr::BinaryOp { op, .. } => {
                        assert_eq!(*op, BinOp::And);
                    }
                    _ => panic!("expected binary op"),
                }
            }
            _ => panic!("expected entry statement"),
        }
    }

    #[test]
    fn test_parse_full_strategy() {
        let source = r#"
            strategy "SMA Crossover"
            param fast_period: int = 12
            param slow_period: int = 26
            let fast = sma(close, fast_period)
            let slow = sma(close, slow_period)
            entry long when crossover(fast, slow)
            entry short when crossunder(fast, slow)
            stop_loss 2%
            take_profit 6%
            plot fast color=blue
            plot slow color=red
        "#;
        let script = parse_str(source).expect("parse error");
        assert!(script.strategy.is_some());
        assert_eq!(script.params.len(), 2);
        assert!(script.statements.len() >= 6);
    }

    #[test]
    fn test_parse_data_sources() {
        let source = "let x = open + high + low + close + volume";
        let script = parse_str(source).expect("parse error");
        assert_eq!(script.statements.len(), 1);
    }

    #[test]
    fn test_parse_unary() {
        let source = "let x = -close";
        let script = parse_str(source).expect("parse error");
        match &script.statements[0].node {
            Statement::Let(let_stmt) => {
                match &let_stmt.value.node {
                    Expr::UnaryOp { op, .. } => {
                        assert_eq!(*op, UnaryOp::Neg);
                    }
                    _ => panic!("expected unary op"),
                }
            }
            _ => panic!("expected let statement"),
        }
    }

    #[test]
    fn test_parse_not() {
        let source = "entry long when not crossunder(fast, slow)";
        let script = parse_str(source).expect("parse error");
        match &script.statements[0].node {
            Statement::Entry(entry) => {
                match &entry.condition.node {
                    Expr::UnaryOp { op, .. } => {
                        assert_eq!(*op, UnaryOp::Not);
                    }
                    _ => panic!("expected unary op"),
                }
            }
            _ => panic!("expected entry statement"),
        }
    }

    #[test]
    fn test_parse_field_access() {
        let source = "let upper = bands.upper";
        let script = parse_str(source).expect("parse error");
        match &script.statements[0].node {
            Statement::Let(let_stmt) => {
                match &let_stmt.value.node {
                    Expr::FieldAccess { field, .. } => {
                        assert_eq!(field, "upper");
                    }
                    _ => panic!("expected field access"),
                }
            }
            _ => panic!("expected let statement"),
        }
    }
}
