//! Main backtesting engine

use crate::config::BacktestConfig;
use crate::error::{BacktestError, BacktestResult};
use crate::models::{ExitReason, Position, PositionSide, Trade};
use crate::result::BacktestResultData;
use crate::sizing::SizingContext;
use super::BacktestState;

use trdelnik_core::{AxisCoordinate, CandleSeries};
use trdelnik_graph::Executor;
use trdelnik_script::ast::ExitTarget;
use trdelnik_script::CompiledStrategy;

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
        // Validate inputs
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

        // Execute the graph to get all signals
        let result = executor.process_series(series);

        // Get effective stop/take profit (strategy overrides config)
        let stop_loss_pct = strategy.stop_loss.or(self.config.risk_config.stop_loss_pct);
        let take_profit_pct = strategy
            .take_profit
            .or(self.config.risk_config.take_profit_pct);

        // Initialize state
        let mut state: BacktestState<X> = BacktestState::new(self.config.initial_capital);

        // Process each bar
        for (bar_index, candle) in series.candles().iter().enumerate() {
            state.current_bar = bar_index;

            // Skip warmup period
            if bar_index < warmup {
                continue;
            }

            let close = candle.close;
            let high = candle.high;
            let low = candle.low;
            let x = candle.x;

            // Update water marks for all positions
            for pos in &mut state.positions {
                pos.update_water_marks(high, low);
            }

            // Check for exits first (before new entries)
            self.process_exits(
                &mut state,
                bar_index,
                x,
                close,
                high,
                low,
                strategy,
                &result,
                stop_loss_pct,
                take_profit_pct,
            );

            // Check for entries
            self.process_entries(
                &mut state,
                bar_index,
                x,
                candle,
                strategy,
                &result,
                stop_loss_pct,
                take_profit_pct,
            );

            // Update equity and record
            state.update_equity(close);
            state.record_equity_point(x);

            // Check max drawdown limit
            if let Some(max_dd) = self.config.risk_config.max_drawdown_pct {
                if state.max_drawdown_pct >= max_dd {
                    // Close all positions
                    self.close_all_positions(&mut state, x, bar_index, close, ExitReason::RiskLimit);
                    // Could return early or continue with no new positions
                }
            }
        }

        // Close any remaining positions at end of data
        if !state.positions.is_empty() {
            let last_candle = series.last().unwrap();
            self.close_all_positions(
                &mut state,
                last_candle.x,
                series.len() - 1,
                last_candle.close,
                ExitReason::EndOfData,
            );
            state.update_equity(last_candle.close);
        }

        // Build result
        Ok(BacktestResultData::new(
            self.config.initial_capital,
            state.equity,
            state.trades,
            state.equity_curve,
            state.total_commission,
            state.total_slippage,
        ))
    }

    /// Process potential entry signals
    fn process_entries<X: AxisCoordinate>(
        &self,
        state: &mut BacktestState<X>,
        bar_index: usize,
        x: X,
        candle: &trdelnik_core::Candle<X>,
        strategy: &CompiledStrategy,
        result: &trdelnik_graph::ExecutionResult,
        stop_loss_pct: Option<f64>,
        take_profit_pct: Option<f64>,
    ) {
        let close = candle.close;

        // Check position limits
        if state.positions.len() >= self.config.max_positions {
            return;
        }

        // Check long entry
        if let Some(entry_node) = strategy.entry_long {
            let signals = result.get_output_f64(entry_node);
            if let Some(Some(signal)) = signals.get(bar_index) {
                if *signal > 0.5 {
                    // Signal is true
                    if self.config.allow_pyramiding || !state.has_position_side(PositionSide::Long)
                    {
                        self.open_position(
                            state,
                            PositionSide::Long,
                            close,
                            x,
                            bar_index,
                            stop_loss_pct,
                            take_profit_pct,
                        );
                    }
                }
            }
        }

        // Check short entry
        if let Some(entry_node) = strategy.entry_short {
            let signals = result.get_output_f64(entry_node);
            if let Some(Some(signal)) = signals.get(bar_index) {
                if *signal > 0.5 {
                    if self.config.allow_pyramiding || !state.has_position_side(PositionSide::Short)
                    {
                        self.open_position(
                            state,
                            PositionSide::Short,
                            close,
                            x,
                            bar_index,
                            stop_loss_pct,
                            take_profit_pct,
                        );
                    }
                }
            }
        }
    }

    /// Open a new position
    fn open_position<X: AxisCoordinate>(
        &self,
        state: &mut BacktestState<X>,
        side: PositionSide,
        price: f64,
        x: X,
        bar_index: usize,
        stop_loss_pct: Option<f64>,
        take_profit_pct: Option<f64>,
    ) {
        // Calculate fill price with slippage
        let fill_price = self
            .config
            .slippage_model
            .adjusted_price(price, side, true);

        // Calculate stop loss and take profit prices
        let stop_loss = stop_loss_pct.map(|pct| match side {
            PositionSide::Long => fill_price * (1.0 - pct / 100.0),
            PositionSide::Short => fill_price * (1.0 + pct / 100.0),
        });

        let take_profit = take_profit_pct.map(|pct| match side {
            PositionSide::Long => fill_price * (1.0 + pct / 100.0),
            PositionSide::Short => fill_price * (1.0 - pct / 100.0),
        });

        // Calculate position size
        let sizing_ctx =
            SizingContext::new(state.equity, state.cash, fill_price, side, x.to_plot_value(), bar_index)
                .with_open_positions(state.positions.len());
        let sizing_ctx = if let Some(sl) = stop_loss {
            sizing_ctx.with_stop_loss(sl)
        } else {
            sizing_ctx
        };

        let quantity = self.config.position_sizer.calculate_size(&sizing_ctx);

        if quantity <= 0.0 {
            return; // Can't open position with zero size
        }

        // Calculate costs
        let notional = fill_price * quantity;
        let commission = self.config.commission_model.calculate_commission(fill_price, quantity);
        let slippage = self.config.slippage_model.calculate_slippage(fill_price, quantity, side, true);

        // Check if we have enough cash
        if notional + commission > state.cash {
            return; // Insufficient funds
        }

        // Create position
        let mut position = Position::new(side, fill_price, quantity, x, bar_index);
        position.stop_loss = stop_loss;
        position.take_profit = take_profit;
        position.entry_commission = commission;
        position.entry_slippage = slippage;

        // Update state
        state.cash -= notional + commission;
        state.total_commission += commission;
        state.total_slippage += slippage;
        state.positions.push(position);
    }

    /// Process potential exit signals
    fn process_exits<X: AxisCoordinate>(
        &self,
        state: &mut BacktestState<X>,
        bar_index: usize,
        x: X,
        close: f64,
        high: f64,
        low: f64,
        strategy: &CompiledStrategy,
        result: &trdelnik_graph::ExecutionResult,
        _stop_loss_pct: Option<f64>,
        _take_profit_pct: Option<f64>,
    ) {
        if state.positions.is_empty() {
            return;
        }

        let trailing_stop = &self.config.risk_config.trailing_stop;

        // Collect positions to close (can't modify while iterating)
        let mut to_close: Vec<(usize, ExitReason, f64)> = Vec::new();

        for (idx, pos) in state.positions.iter().enumerate() {
            // Check stop loss
            if pos.is_stop_loss_hit(low, high) {
                let exit_price = pos.stop_loss.unwrap();
                to_close.push((idx, ExitReason::StopLoss, exit_price));
                continue;
            }

            // Check take profit
            if pos.is_take_profit_hit(low, high) {
                let exit_price = pos.take_profit.unwrap();
                to_close.push((idx, ExitReason::TakeProfit, exit_price));
                continue;
            }

            // Check trailing stop
            if trailing_stop.enabled {
                let should_check = if let Some(activation) = trailing_stop.activation_pct {
                    pos.unrealized_pnl_pct(close) >= activation
                } else {
                    true
                };

                if should_check && pos.is_trailing_stop_hit(low, high, trailing_stop.trail_pct) {
                    let trail_price = pos.calculate_trailing_stop(trailing_stop.trail_pct);
                    to_close.push((idx, ExitReason::TrailingStop, trail_price));
                    continue;
                }
            }

            // Check signal exits
            for (target, node_id) in &strategy.exit_signals {
                let signals = result.get_output_f64(*node_id);
                if let Some(Some(signal)) = signals.get(bar_index) {
                    if *signal > 0.5 {
                        let should_close = match target {
                            ExitTarget::All => true,
                            ExitTarget::Long => pos.side == PositionSide::Long,
                            ExitTarget::Short => pos.side == PositionSide::Short,
                        };
                        if should_close {
                            to_close.push((idx, ExitReason::Signal, close));
                            break;
                        }
                    }
                }
            }
        }

        // Close positions in reverse order to maintain indices
        to_close.sort_by(|a, b| b.0.cmp(&a.0));
        for (idx, reason, exit_price) in to_close {
            self.close_position(state, idx, x, bar_index, exit_price, reason);
        }
    }

    /// Close a single position
    fn close_position<X: AxisCoordinate>(
        &self,
        state: &mut BacktestState<X>,
        position_idx: usize,
        x: X,
        bar_index: usize,
        price: f64,
        reason: ExitReason,
    ) {
        let pos = state.positions.remove(position_idx);

        // Calculate fill price with slippage
        let fill_price = self
            .config
            .slippage_model
            .adjusted_price(price, pos.side, false);

        // Calculate costs
        let commission = self
            .config
            .commission_model
            .calculate_commission(fill_price, pos.quantity);
        let slippage = self
            .config
            .slippage_model
            .calculate_slippage(fill_price, pos.quantity, pos.side, false);

        // Create trade record
        let trade = Trade::new(
            pos.id,
            pos.side,
            pos.entry_price,
            pos.entry_x,
            pos.entry_bar,
            pos.entry_commission,
            pos.entry_slippage,
            fill_price,
            x,
            bar_index,
            commission,
            slippage,
            reason,
            pos.quantity,
            pos.high_water_mark,
            pos.low_water_mark,
        );

        // Update cash
        let pnl = trade.gross_pnl;
        state.cash += pos.notional_value() + pnl - commission;

        // Update state
        state.total_commission += commission;
        state.total_slippage += slippage;
        state.trades.push(trade);
    }

    /// Close all open positions
    fn close_all_positions<X: AxisCoordinate>(
        &self,
        state: &mut BacktestState<X>,
        x: X,
        bar_index: usize,
        price: f64,
        reason: ExitReason,
    ) {
        while !state.positions.is_empty() {
            self.close_position(state, 0, x, bar_index, price, reason);
        }
    }
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
