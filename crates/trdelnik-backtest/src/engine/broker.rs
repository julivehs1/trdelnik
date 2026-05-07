//! Backtest implementation of the [`trdelnik_broker::Broker`] trait.
//!
//! `BacktestBroker` owns the simulated state (cash, positions, trades,
//! equity curve) and the execution rules (slippage, commissions, water
//! marks, stop-loss / take-profit / trailing-stop / max-drawdown). It
//! exposes them through the broker trait so strategy/backtester code
//! talks to the same interface a paper or live broker would.

use trdelnik_broker::{Broker, BrokerError, Order, OrderId, OrderKind, OrderSide};
use trdelnik_core::{AxisCoordinate, Candle};

use crate::config::{BacktestConfig, RiskConfig};
use crate::engine::state::BacktestState;
use crate::error::BacktestError;
use crate::models::{ExitReason, Position, PositionSide, Trade};

/// Broker backend that simulates fills against a frozen `CandleSeries`.
///
/// Drive it with [`BacktestBroker::on_bar`] (or via the [`Broker`]
/// trait) once per candle. Submit orders with [`BacktestBroker::place_order`].
/// Stop-loss, take-profit, trailing-stop and max-drawdown checks fire
/// inside `on_bar` — the broker itself enforces them, the same way a
/// live exchange would, so strategy code stays thin.
pub struct BacktestBroker<X: AxisCoordinate> {
    config: BacktestConfig,
    risk: RiskConfig,
    state: BacktestState<X>,
    next_order_id: u64,
    current_bar: Option<BarSnapshot<X>>,
    /// Strategy-level overrides for SL/TP percentages. Set by the
    /// orchestrating backtester before driving the broker.
    pub strategy_stop_loss_pct: Option<f64>,
    /// Strategy-level take-profit override.
    pub strategy_take_profit_pct: Option<f64>,
    /// True after a max-drawdown trip has fired and forced everything flat.
    risk_halt: bool,
}

#[derive(Debug, Clone)]
struct BarSnapshot<X: AxisCoordinate> {
    candle: Candle<X>,
    bar_index: usize,
}

impl<X: AxisCoordinate> BacktestBroker<X> {
    /// Construct a broker with the given configuration.
    pub fn new(config: BacktestConfig) -> Self {
        let state = BacktestState::new(config.initial_capital);
        let risk = config.risk_config.clone();
        Self {
            config,
            risk,
            state,
            next_order_id: 0,
            current_bar: None,
            strategy_stop_loss_pct: None,
            strategy_take_profit_pct: None,
            risk_halt: false,
        }
    }

    /// Whether a max-drawdown halt has been triggered.
    pub fn risk_halt(&self) -> bool {
        self.risk_halt
    }

    /// Total commission paid since construction.
    pub fn total_commission(&self) -> f64 {
        self.state.total_commission
    }

    /// Total slippage paid since construction.
    pub fn total_slippage(&self) -> f64 {
        self.state.total_slippage
    }

    /// Equity curve recorded so far.
    pub fn equity_curve(&self) -> &[crate::engine::EquityPoint<X>] {
        &self.state.equity_curve
    }

    /// Force-close every open position at the current bar's close, with
    /// the given exit reason. Used by the orchestrator for end-of-data
    /// cleanup or signal-driven exits ("exit all when …").
    pub fn close_all(&mut self, reason: ExitReason) -> Result<(), BacktestError> {
        let bar = self
            .current_bar
            .as_ref()
            .ok_or(BrokerError::NoCurrentBar)?
            .clone();
        while !self.state.positions.is_empty() {
            self.close_position_at(0, bar.candle.x, bar.bar_index, bar.candle.close, reason);
        }
        Ok(())
    }

    /// Force-close every open position on the given side. Used to
    /// implement strategy-level "exit long" / "exit short" signals.
    pub fn close_side(
        &mut self,
        side: PositionSide,
        reason: ExitReason,
    ) -> Result<(), BacktestError> {
        let bar = self
            .current_bar
            .as_ref()
            .ok_or(BrokerError::NoCurrentBar)?
            .clone();
        let mut idx = 0;
        while idx < self.state.positions.len() {
            if self.state.positions[idx].side == side {
                self.close_position_at(idx, bar.candle.x, bar.bar_index, bar.candle.close, reason);
            } else {
                idx += 1;
            }
        }
        Ok(())
    }

    /// Drop the broker, returning its terminal state for result building.
    pub fn into_state(self) -> BacktestState<X> {
        self.state
    }

    fn allocate_order_id(&mut self) -> OrderId {
        let id = OrderId::new(self.next_order_id);
        self.next_order_id += 1;
        id
    }

    fn fill_market_order(&mut self, order: Order) -> Result<OrderId, BacktestError> {
        if self.risk_halt {
            // Drawdown limit tripped — silently reject new entries; the
            // backtester decides whether to stop running.
            return Err(BacktestError::RiskLimitExceeded(
                "max drawdown reached".to_string(),
            ));
        }

        let bar = self
            .current_bar
            .as_ref()
            .ok_or(BrokerError::NoCurrentBar)?
            .clone();

        if order.quantity <= 0.0 {
            return Err(BrokerError::InvalidQuantity(order.quantity).into());
        }

        if self.state.positions.len() >= self.config.max_positions {
            // Position limit reached — quietly drop. The backtester is
            // expected to check this itself for cleaner messages, but we
            // also guard here.
            return Err(BacktestError::Execution(
                "max positions reached".to_string(),
            ));
        }

        let side = into_position_side(order.side);

        if !self.config.allow_pyramiding && self.state.has_position_side(side) {
            return Err(BacktestError::Execution(
                "pyramiding disabled and side already open".to_string(),
            ));
        }

        // Slippage-adjusted entry fill at the bar's close.
        let fill_price = self
            .config
            .slippage_model
            .adjusted_price(bar.candle.close, side, true);

        let stop_loss_pct = self.strategy_stop_loss_pct.or(self.risk.stop_loss_pct);
        let take_profit_pct = self.strategy_take_profit_pct.or(self.risk.take_profit_pct);

        let stop_loss = order
            .stop_loss
            .or_else(|| stop_loss_pct.map(|pct| sl_from_pct(side, fill_price, pct)));
        let take_profit = order
            .take_profit
            .or_else(|| take_profit_pct.map(|pct| tp_from_pct(side, fill_price, pct)));

        // Position sizing: caller provided quantity, but if it's 0 it
        // already errored above. We honour the caller's size here.
        let quantity = order.quantity;

        let notional = fill_price * quantity;
        let commission = self
            .config
            .commission_model
            .calculate_commission(fill_price, quantity);
        let slippage = self
            .config
            .slippage_model
            .calculate_slippage(fill_price, quantity, side, true);

        if notional + commission > self.state.cash {
            return Err(BrokerError::InsufficientFunds {
                required: notional + commission,
                available: self.state.cash,
            }
            .into());
        }

        let mut position = Position::new(side, fill_price, quantity, bar.candle.x, bar.bar_index);
        position.stop_loss = stop_loss;
        position.take_profit = take_profit;
        position.entry_commission = commission;
        position.entry_slippage = slippage;

        self.state.cash -= notional + commission;
        self.state.total_commission += commission;
        self.state.total_slippage += slippage;
        self.state.positions.push(position);

        Ok(self.allocate_order_id())
    }

    fn close_position_at(
        &mut self,
        position_idx: usize,
        x: X,
        bar_index: usize,
        price: f64,
        reason: ExitReason,
    ) {
        let pos = self.state.positions.remove(position_idx);

        let fill_price = self
            .config
            .slippage_model
            .adjusted_price(price, pos.side, false);

        let commission = self
            .config
            .commission_model
            .calculate_commission(fill_price, pos.quantity);
        let slippage = self
            .config
            .slippage_model
            .calculate_slippage(fill_price, pos.quantity, pos.side, false);

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

        let pnl = trade.gross_pnl;
        self.state.cash += pos.notional_value() + pnl - commission;
        self.state.total_commission += commission;
        self.state.total_slippage += slippage;
        self.state.trades.push(trade);
    }

    /// Run SL/TP/trailing-stop checks and close any positions that hit.
    fn run_protective_exits(&mut self, candle: &Candle<X>, bar_index: usize) {
        let trailing = self.risk.trailing_stop.clone();
        let mut to_close: Vec<(usize, ExitReason, f64)> = Vec::new();

        for (idx, pos) in self.state.positions.iter().enumerate() {
            if pos.is_stop_loss_hit(candle.low, candle.high) {
                let exit_price = pos.stop_loss.unwrap();
                to_close.push((idx, ExitReason::StopLoss, exit_price));
                continue;
            }
            if pos.is_take_profit_hit(candle.low, candle.high) {
                let exit_price = pos.take_profit.unwrap();
                to_close.push((idx, ExitReason::TakeProfit, exit_price));
                continue;
            }
            if trailing.enabled {
                let activated = trailing
                    .activation_pct
                    .map(|pct| pos.unrealized_pnl_pct(candle.close) >= pct)
                    .unwrap_or(true);
                if activated && pos.is_trailing_stop_hit(candle.low, candle.high, trailing.trail_pct)
                {
                    let trail_price = pos.calculate_trailing_stop(trailing.trail_pct);
                    to_close.push((idx, ExitReason::TrailingStop, trail_price));
                    continue;
                }
            }
        }

        // Close in reverse so indices stay valid.
        to_close.sort_by(|a, b| b.0.cmp(&a.0));
        for (idx, reason, price) in to_close {
            self.close_position_at(idx, candle.x, bar_index, price, reason);
        }
    }
}

fn into_position_side(side: OrderSide) -> PositionSide {
    match side {
        OrderSide::Long => PositionSide::Long,
        OrderSide::Short => PositionSide::Short,
    }
}

fn sl_from_pct(side: PositionSide, fill_price: f64, pct: f64) -> f64 {
    match side {
        PositionSide::Long => fill_price * (1.0 - pct / 100.0),
        PositionSide::Short => fill_price * (1.0 + pct / 100.0),
    }
}

fn tp_from_pct(side: PositionSide, fill_price: f64, pct: f64) -> f64 {
    match side {
        PositionSide::Long => fill_price * (1.0 + pct / 100.0),
        PositionSide::Short => fill_price * (1.0 - pct / 100.0),
    }
}

impl<X: AxisCoordinate> std::fmt::Debug for BacktestBroker<X> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BacktestBroker")
            .field("cash", &self.state.cash)
            .field("equity", &self.state.equity)
            .field("open_positions", &self.state.positions.len())
            .field("trades", &self.state.trades.len())
            .field("risk_halt", &self.risk_halt)
            .finish()
    }
}

impl<X: AxisCoordinate> Broker<X> for BacktestBroker<X> {
    type Position = Position<X>;
    type Trade = Trade<X>;
    type Error = BacktestError;

    fn place_order(&mut self, order: Order) -> Result<OrderId, BacktestError> {
        match order.kind {
            OrderKind::Market => self.fill_market_order(order),
            other => Err(BrokerError::UnsupportedOrderKind(other).into()),
        }
    }

    fn cancel_order(&mut self, _order_id: OrderId) -> Result<(), BacktestError> {
        // Market orders fill immediately — there is nothing to cancel.
        // Limit/stop will need a real implementation when those land.
        Ok(())
    }

    fn on_bar(&mut self, candle: &Candle<X>) -> Result<(), BacktestError> {
        let bar_index = self
            .current_bar
            .as_ref()
            .map(|b| b.bar_index + 1)
            .unwrap_or(0);
        self.current_bar = Some(BarSnapshot {
            candle: candle.clone(),
            bar_index,
        });
        self.state.current_bar = bar_index;

        // Update water marks for all positions.
        for pos in &mut self.state.positions {
            pos.update_water_marks(candle.high, candle.low);
        }

        // Protective exits (SL/TP/trailing) before equity update so the
        // recorded equity reflects post-exit state.
        self.run_protective_exits(candle, bar_index);

        // Update equity and record curve point.
        self.state.update_equity(candle.close);
        self.state.record_equity_point(candle.x);

        // Max-drawdown check — if breached, force flat and halt.
        if let Some(max_dd) = self.risk.max_drawdown_pct {
            if !self.risk_halt && self.state.max_drawdown_pct >= max_dd {
                self.risk_halt = true;
                while !self.state.positions.is_empty() {
                    self.close_position_at(
                        0,
                        candle.x,
                        bar_index,
                        candle.close,
                        ExitReason::RiskLimit,
                    );
                }
            }
        }

        Ok(())
    }

    fn positions(&self) -> &[Position<X>] {
        &self.state.positions
    }

    fn trades(&self) -> &[Trade<X>] {
        &self.state.trades
    }

    fn cash(&self) -> f64 {
        self.state.cash
    }

    fn equity(&self) -> f64 {
        self.state.equity
    }
}
