//! Main backtesting orchestrator.
//!
//! `Backtester` is responsible for *strategy orchestration*: it runs the
//! computation graph, reads entry/exit signals out of the result, sizes
//! positions, and submits orders. Order execution, fills, water marks,
//! protective exits and equity tracking live in [`BacktestBroker`].
//!
//! The split makes the same strategy code reusable later against a
//! paper-trading or live broker — the orchestration logic stays here,
//! the backend behind the [`trdelnik_broker::Broker`] trait swaps out.

use trdelnik_broker::{Broker, Order, OrderSide};
use trdelnik_core::{AxisCoordinate, CandleSeries};
use trdelnik_graph::Executor;
use trdelnik_script::ast::ExitTarget;
use trdelnik_script::CompiledStrategy;

use crate::config::BacktestConfig;
use crate::engine::broker::BacktestBroker;
use crate::error::{BacktestError, BacktestResult};
use crate::models::{ExitReason, PositionSide};
use crate::result::BacktestResultData;
use crate::sizing::SizingContext;

/// The main backtesting engine
pub struct Backtester {
    config: BacktestConfig,
}

impl Backtester {
    /// Create a new backtester with the given configuration
    pub fn new(config: BacktestConfig) -> Self {
        Self { config }
    }

    /// Run a backtest with the given strategy and data
    pub fn run<X: AxisCoordinate>(
        &self,
        strategy: &CompiledStrategy,
        series: &CandleSeries<X>,
    ) -> BacktestResult<BacktestResultData<X>> {
        if strategy.entry_long.is_none() && strategy.entry_short.is_none() {
            return Err(BacktestError::NoEntrySignals);
        }

        let mut executor = Executor::new(strategy.graph.clone());
        let warmup = executor.max_warmup_period();

        if series.len() < warmup + 1 {
            return Err(BacktestError::InsufficientData {
                required: warmup + 1,
                actual: series.len(),
            });
        }

        // Pre-compute the full signal series. (When live streaming lands
        // this becomes a per-bar stream; the orchestration loop is the
        // same shape.)
        let result = executor.process_series(series);

        // Strategy-level SL/TP overrides config.
        let mut broker = BacktestBroker::new(self.config.clone());
        broker.strategy_stop_loss_pct = strategy.stop_loss.or(self.config.risk_config.stop_loss_pct);
        broker.strategy_take_profit_pct = strategy
            .take_profit
            .or(self.config.risk_config.take_profit_pct);

        let initial_capital = self.config.initial_capital;

        for (bar_index, candle) in series.candles().iter().enumerate() {
            // Advance broker clock: water marks, protective exits, equity.
            broker.on_bar(candle)?;

            // Skip warmup; the broker still ticks so equity points exist
            // for the curve, but we don't act on signals yet.
            if bar_index < warmup {
                continue;
            }

            // If max-drawdown halted the broker, don't submit anything.
            if broker.risk_halt() {
                continue;
            }

            // Signal-driven exits (before entries — closing legacy
            // positions clears the way for new ones if pyramiding off).
            for (target, node_id) in &strategy.exit_signals {
                if signal_true(&result, *node_id, bar_index) {
                    match target {
                        ExitTarget::All => broker.close_all(ExitReason::Signal)?,
                        ExitTarget::Long => {
                            broker.close_side(PositionSide::Long, ExitReason::Signal)?
                        }
                        ExitTarget::Short => {
                            broker.close_side(PositionSide::Short, ExitReason::Signal)?
                        }
                    }
                }
            }

            // Entries.
            if let Some(node) = strategy.entry_long {
                if signal_true(&result, node, bar_index) {
                    self.submit_entry(
                        &mut broker,
                        OrderSide::Long,
                        candle.close,
                        candle.x,
                        bar_index,
                    );
                }
            }
            if let Some(node) = strategy.entry_short {
                if signal_true(&result, node, bar_index) {
                    self.submit_entry(
                        &mut broker,
                        OrderSide::Short,
                        candle.close,
                        candle.x,
                        bar_index,
                    );
                }
            }
        }

        // Close any remaining positions at end of data.
        if !broker.positions().is_empty() {
            broker.close_all(ExitReason::EndOfData)?;
        }

        let total_commission = broker.total_commission();
        let total_slippage = broker.total_slippage();
        let final_state = broker.into_state();

        Ok(BacktestResultData::new(
            initial_capital,
            final_state.equity,
            final_state.trades,
            final_state.equity_curve,
            total_commission,
            total_slippage,
        ))
    }

    /// Compute size via the configured sizer and submit a market order.
    /// Failures (insufficient cash, max positions, pyramiding off) are
    /// silently ignored — they're not strategy errors, they're routine
    /// "skip this bar" outcomes that match the legacy behaviour.
    fn submit_entry<X: AxisCoordinate>(
        &self,
        broker: &mut BacktestBroker<X>,
        side: OrderSide,
        close: f64,
        x: X,
        bar_index: usize,
    ) {
        let position_side = match side {
            OrderSide::Long => PositionSide::Long,
            OrderSide::Short => PositionSide::Short,
        };

        // Slippage-adjusted price for sizing — mirrors what the broker
        // will actually fill at. The risk-based sizer needs this to know
        // distance-to-stop accurately.
        let fill_price = self
            .config
            .slippage_model
            .adjusted_price(close, position_side, true);

        let stop_loss_pct = broker
            .strategy_stop_loss_pct
            .or(self.config.risk_config.stop_loss_pct);

        let mut sizing_ctx = SizingContext::new(
            broker.equity(),
            broker.cash(),
            fill_price,
            position_side,
            x.to_plot_value(),
            bar_index,
        )
        .with_open_positions(broker.positions().len());

        if let Some(pct) = stop_loss_pct {
            let sl = match position_side {
                PositionSide::Long => fill_price * (1.0 - pct / 100.0),
                PositionSide::Short => fill_price * (1.0 + pct / 100.0),
            };
            sizing_ctx = sizing_ctx.with_stop_loss(sl);
        }

        let quantity = self.config.position_sizer.calculate_size(&sizing_ctx);
        if quantity <= 0.0 {
            return;
        }

        let _ = broker.place_order(Order::market(side, quantity));
    }
}

fn signal_true(
    result: &trdelnik_graph::ExecutionResult,
    node_id: trdelnik_graph::NodeId,
    bar_index: usize,
) -> bool {
    let signals = result.get_output_f64(node_id);
    matches!(signals.get(bar_index), Some(Some(s)) if *s > 0.5)
}

impl std::fmt::Debug for Backtester {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Backtester")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backtester_creation() {
        let config = BacktestConfig::default();
        let backtester = Backtester::new(config);
        assert!(backtester.config.initial_capital > 0.0);
    }
}
