//! Demo: trdelnik-backtest
//!
//! Zeigt den kompletten Backtesting-Flow:
//! 1. Strategie in TrdelScript definieren
//! 2. Kurs-Daten generieren
//! 3. Backtester konfigurieren
//! 4. Backtest ausführen
//! 5. Ergebnisse anzeigen

use trdelnik_backtest::{
    BacktestConfig, Backtester, PercentOfEquity, PercentageCommission, PercentageSlippage,
    RiskConfig, TrailingStopConfig,
};
use trdelnik_core::{Candle, CandleSeries, Timestamp};
use trdelnik_script::compile;

fn main() {
    println!("=== trdelnik-backtest Demo ===\n");

    // 1. Strategie definieren
    let strategy_source = r#"
        strategy "SMA Crossover"

        let fast = sma(close, 10)
        let slow = sma(close, 30)

        entry long when crossover(fast, slow)
        entry short when crossunder(fast, slow)

        stop_loss 3%
        take_profit 9%
    "#;

    println!("Kompiliere Strategie...");
    let strategy = match compile(strategy_source) {
        Ok(s) => {
            println!("  Name: {:?}", s.name);
            println!("  Entry Long: {}", s.entry_long.is_some());
            println!("  Entry Short: {}", s.entry_short.is_some());
            println!("  Stop Loss: {:?}%", s.stop_loss);
            println!("  Take Profit: {:?}%", s.take_profit);
            s
        }
        Err(e) => {
            eprintln!("Kompilierungsfehler: {}", e);
            return;
        }
    };

    // 2. Kurs-Daten generieren (simulierter Trend-Markt)
    println!("\nGeneriere Kurs-Daten...");
    let series = generate_trending_data(500);
    println!("  {} Kerzen generiert", series.len());
    if let Some((min, max)) = series.price_range() {
        println!("  Preis-Range: {:.2} - {:.2}", min, max);
    }

    // 3. Backtester konfigurieren
    println!("\nKonfiguriere Backtester...");
    let config = BacktestConfig::builder()
        .initial_capital(100_000.0)
        .position_sizer(PercentOfEquity::new(20.0)) // 20% pro Trade
        .slippage_model(PercentageSlippage::new(0.05)) // 0.05% Slippage
        .commission_model(PercentageCommission::new(0.1)) // 0.1% Kommission
        .allow_pyramiding(false)
        .max_positions(1)
        .risk_config(
            RiskConfig::new()
                .with_trailing_stop(TrailingStopConfig::new(2.0).with_activation(1.0)),
        )
        .build();

    println!("  Startkapital: ${:.2}", config.initial_capital);
    println!("  Max Positionen: {}", config.max_positions);

    // Debug: Signale prüfen
    println!("\nPrüfe Signale...");
    let mut executor = trdelnik_script::Executor::new(strategy.graph.clone());
    let exec_result = executor.process_series(&series);

    // Debug: Zeige Node 5, 6, 7 für viele Bars (5=slow SMA, 6=fast SMA, 7=crossover)
    println!("\n  SMA-Werte und Crossover (Node 5=slow, 6=fast, 7=crossover):");
    println!("  Bar | Slow SMA | Fast SMA | Crossover");
    println!("  ----|----------|----------|----------");
    for bar in 28..50 {
        let n5 = exec_result.get_output_f64(trdelnik_script::NodeId(5));
        let n6 = exec_result.get_output_f64(trdelnik_script::NodeId(6));
        let n7 = exec_result.get_output_f64(trdelnik_script::NodeId(7));

        let v5 = n5.get(bar).and_then(|x| *x).map(|v| format!("{:.2}", v)).unwrap_or("None".to_string());
        let v6 = n6.get(bar).and_then(|x| *x).map(|v| format!("{:.2}", v)).unwrap_or("None".to_string());
        let v7 = n7.get(bar).and_then(|x| *x).map(|v| format!("{:.1}", v)).unwrap_or("None".to_string());

        println!("  {:3} | {:>8} | {:>8} | {:>9}", bar, v5, v6, v7);
    }

    if let Some(entry_long) = strategy.entry_long {
        let signals = exec_result.get_output_f64(entry_long);
        let signal_count = signals.iter().filter(|s| s.map(|v| v > 0.5).unwrap_or(false)).count();
        println!("\n  Long Entry Node: {:?}", entry_long);
        println!("  Gesamt-Signale > 0.5: {}", signal_count);

        // Zähle auch None vs 0 vs 1
        let none_count = signals.iter().filter(|s| s.is_none()).count();
        let zero_count = signals.iter().filter(|s| s.map(|v| v < 0.5).unwrap_or(false)).count();
        println!("  None: {}, Null: {}, Eins: {}", none_count, zero_count, signal_count);
    }

    if let Some(entry_short) = strategy.entry_short {
        let signals = exec_result.get_output_f64(entry_short);
        let signal_count = signals.iter().filter(|s| s.map(|v| v > 0.5).unwrap_or(false)).count();
        println!("  Short Entry Signale: {}", signal_count);
    }

    // 4. Backtest ausführen
    println!("\nFühre Backtest aus...");
    let backtester = Backtester::new(config);

    match backtester.run(&strategy, &series) {
        Ok(result) => {
            println!("\n");
            result.print_summary();

            // Zusätzliche Details
            println!("\n=== Trade-Details ===");
            for (i, trade) in result.trades.iter().take(10).enumerate() {
                println!(
                    "  #{}: {} @ {:.2} -> {:.2} | P&L: ${:.2} ({:.2}%) | {}",
                    i + 1,
                    trade.side,
                    trade.entry_price,
                    trade.exit_price,
                    trade.net_pnl,
                    trade.return_pct(),
                    trade.exit_reason
                );
            }
            if result.trades.len() > 10 {
                println!("  ... und {} weitere Trades", result.trades.len() - 10);
            }

            // Equity Curve Statistiken
            println!("\n=== Equity Curve ===");
            if let Some(dd_info) = result.equity_curve.max_drawdown_info() {
                println!(
                    "  Max Drawdown: ${:.2} ({:.2}%)",
                    dd_info.max_drawdown, dd_info.max_drawdown_pct
                );
                println!(
                    "  Peak: ${:.2} @ Bar {}",
                    dd_info.peak_equity, dd_info.peak_bar
                );
                println!(
                    "  Trough: ${:.2} @ Bar {}",
                    dd_info.trough_equity, dd_info.trough_bar
                );
            }
        }
        Err(e) => {
            eprintln!("Backtest-Fehler: {}", e);
        }
    }
}

/// Generiert simulierte Kurs-Daten mit Zyklen (für Crossover-Signale)
fn generate_trending_data(num_bars: usize) -> CandleSeries<Timestamp> {
    let mut series = CandleSeries::new();
    let mut rng_state: u64 = 42;

    for i in 0..num_bars {
        // Pseudo-Random für Rauschen
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let random = ((rng_state >> 16) & 0x7FFF) as f64 / 32767.0 - 0.5;

        // Basis-Preis mit Zyklen (erzeugt Crossovers)
        // Langsamer Zyklus + schnellerer Zyklus + leichter Aufwärtstrend
        let slow_cycle = (i as f64 * 0.03).sin() * 15.0; // ~210 Bars pro Zyklus
        let fast_cycle = (i as f64 * 0.1).sin() * 5.0; // ~63 Bars pro Zyklus
        let trend = i as f64 * 0.02; // leichter Aufwärtstrend
        let noise = random * 2.0;

        let close = 100.0 + slow_cycle + fast_cycle + trend + noise;

        // OHLC aus Close ableiten
        let volatility = 1.5;
        let open = close + (random * volatility);
        let high = open.max(close) + random.abs() * volatility;
        let low = open.min(close) - random.abs() * volatility;

        let volume = 10000.0 + (random + 0.5) * 50000.0;
        let timestamp = Timestamp::new((i as i64) * 3600 * 1000); // 1h Kerzen

        series.push(Candle::new(timestamp, open, high, low, close, volume));
    }

    series
}
