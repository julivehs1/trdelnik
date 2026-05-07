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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BacktestConfig, RiskConfig, TrailingStopConfig};
    use crate::models::PositionSide;
    use trdelnik_core::Timestamp;

    /// Bar with explicit OHLC. Volume is fixed because nothing here cares.
    fn bar(t: i64, open: f64, high: f64, low: f64, close: f64) -> Candle<Timestamp> {
        Candle::new(Timestamp(t), open, high, low, close, 1_000.0)
    }

    /// Flat candle at price `p` — useful when only the close matters.
    fn flat(t: i64, p: f64) -> Candle<Timestamp> {
        bar(t, p, p, p, p)
    }

    fn fresh_broker(cfg: BacktestConfig) -> BacktestBroker<Timestamp> {
        BacktestBroker::<Timestamp>::new(cfg)
    }

    fn default_broker() -> BacktestBroker<Timestamp> {
        let mut cfg = BacktestConfig::default();
        cfg.initial_capital = 10_000.0;
        cfg.max_positions = 4;
        cfg.allow_pyramiding = true;
        fresh_broker(cfg)
    }

    // ----- Order acceptance / validation -----

    #[test]
    fn place_order_before_on_bar_errors() {
        let mut broker = default_broker();
        let err = broker
            .place_order(Order::market(OrderSide::Long, 1.0))
            .unwrap_err();
        assert!(matches!(
            err,
            BacktestError::Broker(BrokerError::NoCurrentBar)
        ));
    }

    #[test]
    fn quantity_zero_or_negative_is_rejected() {
        let mut broker = default_broker();
        broker.on_bar(&flat(0, 100.0)).unwrap();

        let zero = broker.place_order(Order::market(OrderSide::Long, 0.0));
        assert!(matches!(
            zero,
            Err(BacktestError::Broker(BrokerError::InvalidQuantity(_)))
        ));
        let neg = broker.place_order(Order::market(OrderSide::Long, -5.0));
        assert!(matches!(
            neg,
            Err(BacktestError::Broker(BrokerError::InvalidQuantity(_)))
        ));
    }

    #[test]
    fn insufficient_funds_rejected() {
        let mut broker = default_broker();
        broker.on_bar(&flat(0, 100.0)).unwrap();
        // 200 * 100 = 20_000 > initial_capital 10_000
        let err = broker
            .place_order(Order::market(OrderSide::Long, 200.0))
            .unwrap_err();
        assert!(matches!(
            err,
            BacktestError::Broker(BrokerError::InsufficientFunds { .. })
        ));
        assert_eq!(broker.positions().len(), 0);
    }

    #[test]
    fn max_positions_blocks_new_entries() {
        let mut cfg = BacktestConfig::default();
        cfg.initial_capital = 10_000.0;
        cfg.allow_pyramiding = true;
        cfg.max_positions = 1;
        let mut broker = fresh_broker(cfg);
        broker.on_bar(&flat(0, 50.0)).unwrap();

        broker
            .place_order(Order::market(OrderSide::Long, 1.0))
            .unwrap();
        let err = broker
            .place_order(Order::market(OrderSide::Long, 1.0))
            .unwrap_err();
        assert!(matches!(err, BacktestError::Execution(_)));
        assert_eq!(broker.positions().len(), 1);
    }

    #[test]
    fn pyramiding_disabled_blocks_second_same_side() {
        let mut cfg = BacktestConfig::default();
        cfg.initial_capital = 10_000.0;
        cfg.allow_pyramiding = false;
        cfg.max_positions = 4;
        let mut broker = fresh_broker(cfg);
        broker.on_bar(&flat(0, 50.0)).unwrap();

        broker
            .place_order(Order::market(OrderSide::Long, 1.0))
            .unwrap();
        let err = broker
            .place_order(Order::market(OrderSide::Long, 1.0))
            .unwrap_err();
        assert!(matches!(err, BacktestError::Execution(_)));
        // Opposite side is still allowed.
        broker
            .place_order(Order::market(OrderSide::Short, 1.0))
            .unwrap();
        assert_eq!(broker.positions().len(), 2);
    }

    // ----- Order lifecycle / cash bookkeeping -----

    #[test]
    fn long_round_trip_at_same_price_returns_initial_cash() {
        let mut broker = default_broker();
        broker.on_bar(&flat(0, 100.0)).unwrap();
        broker
            .place_order(Order::market(OrderSide::Long, 10.0))
            .unwrap();
        assert_eq!(broker.cash(), 10_000.0 - 1_000.0);

        broker.close_all(ExitReason::Signal).unwrap();
        // Same price, zero costs → cash should be exactly back to start.
        assert!((broker.cash() - 10_000.0).abs() < 1e-9);
        assert_eq!(broker.trades().len(), 1);
        assert_eq!(broker.trades()[0].exit_reason, ExitReason::Signal);
    }

    #[test]
    fn long_profitable_close_credits_correct_pnl() {
        let mut broker = default_broker();
        broker.on_bar(&flat(0, 100.0)).unwrap();
        broker
            .place_order(Order::market(OrderSide::Long, 10.0))
            .unwrap();

        // Move forward and close at 110 → +100 PnL.
        broker.on_bar(&flat(1, 110.0)).unwrap();
        broker.close_all(ExitReason::Signal).unwrap();

        assert!((broker.cash() - 10_100.0).abs() < 1e-9);
        let trade = &broker.trades()[0];
        assert!((trade.gross_pnl - 100.0).abs() < 1e-9);
        assert_eq!(trade.side, PositionSide::Long);
    }

    #[test]
    fn short_profits_when_price_drops() {
        let mut broker = default_broker();
        broker.on_bar(&flat(0, 100.0)).unwrap();
        broker
            .place_order(Order::market(OrderSide::Short, 10.0))
            .unwrap();
        broker.on_bar(&flat(1, 90.0)).unwrap();
        broker.close_all(ExitReason::Signal).unwrap();

        let trade = &broker.trades()[0];
        assert_eq!(trade.side, PositionSide::Short);
        // Short: (100 - 90) * 10 = +100
        assert!((trade.gross_pnl - 100.0).abs() < 1e-9);
    }

    // ----- Protective exits: SL / TP -----

    #[test]
    fn stop_loss_long_fires_when_low_pierces() {
        let mut broker = default_broker();
        broker.on_bar(&flat(0, 100.0)).unwrap();
        let order = Order::market(OrderSide::Long, 5.0).with_stop_loss(95.0);
        broker.place_order(order).unwrap();

        // Bar dips to 94 → SL hit.
        broker.on_bar(&bar(1, 99.0, 99.5, 94.0, 96.0)).unwrap();
        assert_eq!(broker.positions().len(), 0);
        let trade = &broker.trades()[0];
        assert_eq!(trade.exit_reason, ExitReason::StopLoss);
        // Exit must be at the SL price, not the bar close.
        assert!((trade.exit_price - 95.0).abs() < 1e-9);
    }

    #[test]
    fn take_profit_long_fires_when_high_pierces() {
        let mut broker = default_broker();
        broker.on_bar(&flat(0, 100.0)).unwrap();
        let order = Order::market(OrderSide::Long, 5.0).with_take_profit(110.0);
        broker.place_order(order).unwrap();

        broker.on_bar(&bar(1, 101.0, 112.0, 100.5, 108.0)).unwrap();
        let trade = &broker.trades()[0];
        assert_eq!(trade.exit_reason, ExitReason::TakeProfit);
        assert!((trade.exit_price - 110.0).abs() < 1e-9);
    }

    #[test]
    fn stop_loss_short_fires_when_high_pierces() {
        let mut broker = default_broker();
        broker.on_bar(&flat(0, 100.0)).unwrap();
        let order = Order::market(OrderSide::Short, 5.0).with_stop_loss(105.0);
        broker.place_order(order).unwrap();

        broker.on_bar(&bar(1, 101.0, 107.0, 100.0, 103.0)).unwrap();
        let trade = &broker.trades()[0];
        assert_eq!(trade.exit_reason, ExitReason::StopLoss);
        assert!((trade.exit_price - 105.0).abs() < 1e-9);
    }

    #[test]
    fn risk_config_pct_stops_apply_when_no_per_order_stop() {
        let mut cfg = BacktestConfig::default();
        cfg.initial_capital = 10_000.0;
        cfg.max_positions = 4;
        cfg.allow_pyramiding = true;
        cfg.risk_config = RiskConfig::new()
            .with_stop_loss(2.0)
            .with_take_profit(6.0);
        let mut broker = fresh_broker(cfg);
        broker.on_bar(&flat(0, 100.0)).unwrap();
        broker
            .place_order(Order::market(OrderSide::Long, 5.0))
            .unwrap();

        // Long entry 100 → SL = 98, TP = 106. Drop to 97 → SL hit at 98.
        broker.on_bar(&bar(1, 99.0, 99.5, 97.0, 98.5)).unwrap();
        let trade = &broker.trades()[0];
        assert_eq!(trade.exit_reason, ExitReason::StopLoss);
        assert!((trade.exit_price - 98.0).abs() < 1e-9);
    }

    // ----- Trailing stop -----

    #[test]
    fn trailing_stop_long_fires_after_run_up() {
        let mut cfg = BacktestConfig::default();
        cfg.initial_capital = 10_000.0;
        cfg.max_positions = 4;
        cfg.allow_pyramiding = true;
        cfg.risk_config = RiskConfig::new()
            .with_trailing_stop(TrailingStopConfig::new(5.0));
        let mut broker = fresh_broker(cfg);
        broker.on_bar(&flat(0, 100.0)).unwrap();
        broker
            .place_order(Order::market(OrderSide::Long, 5.0))
            .unwrap();

        // Run up to high 110 — water mark = 110, trail price = 104.5.
        // Bar low must stay above 104.5 or the trailing fires intra-bar.
        broker.on_bar(&bar(1, 100.0, 110.0, 105.0, 109.0)).unwrap();
        assert_eq!(broker.positions().len(), 1, "should still be open");

        // Next bar dips below 104.5 — trips trailing.
        broker.on_bar(&bar(2, 109.0, 109.5, 104.0, 105.0)).unwrap();
        let trade = &broker.trades()[0];
        assert_eq!(trade.exit_reason, ExitReason::TrailingStop);
    }

    #[test]
    fn trailing_stop_with_activation_threshold_holds_off() {
        let mut cfg = BacktestConfig::default();
        cfg.initial_capital = 10_000.0;
        cfg.max_positions = 4;
        cfg.allow_pyramiding = true;
        // 5% trail, only active after +5% profit.
        cfg.risk_config = RiskConfig::new()
            .with_trailing_stop(TrailingStopConfig::new(5.0).with_activation(5.0));
        let mut broker = fresh_broker(cfg);
        broker.on_bar(&flat(0, 100.0)).unwrap();
        broker
            .place_order(Order::market(OrderSide::Long, 5.0))
            .unwrap();

        // High 103 (+3%, below activation) then drop to 98. Trailing
        // must NOT fire — activation threshold not yet crossed.
        broker.on_bar(&bar(1, 100.0, 103.0, 99.0, 102.0)).unwrap();
        broker.on_bar(&bar(2, 102.0, 102.5, 98.0, 99.0)).unwrap();
        assert_eq!(
            broker.positions().len(),
            1,
            "trailing stop fired before activation threshold"
        );
        assert!(broker.trades().is_empty());
    }

    // ----- close_all / close_side -----

    #[test]
    fn close_side_only_closes_matching_positions() {
        let mut broker = default_broker();
        broker.on_bar(&flat(0, 100.0)).unwrap();
        broker
            .place_order(Order::market(OrderSide::Long, 1.0))
            .unwrap();
        broker
            .place_order(Order::market(OrderSide::Short, 1.0))
            .unwrap();
        assert_eq!(broker.positions().len(), 2);

        broker
            .close_side(PositionSide::Long, ExitReason::Signal)
            .unwrap();

        assert_eq!(broker.positions().len(), 1);
        assert_eq!(broker.positions()[0].side, PositionSide::Short);
        assert_eq!(broker.trades().len(), 1);
        assert_eq!(broker.trades()[0].side, PositionSide::Long);
    }

    #[test]
    fn close_all_uses_current_bar_close_price() {
        let mut broker = default_broker();
        broker.on_bar(&flat(0, 100.0)).unwrap();
        broker
            .place_order(Order::market(OrderSide::Long, 5.0))
            .unwrap();
        // Move to bar 1 with close 105, then close_all.
        broker.on_bar(&bar(1, 100.5, 106.0, 100.0, 105.0)).unwrap();
        broker.close_all(ExitReason::EndOfData).unwrap();

        let trade = &broker.trades()[0];
        assert!((trade.exit_price - 105.0).abs() < 1e-9);
        assert_eq!(trade.exit_reason, ExitReason::EndOfData);
    }

    #[test]
    fn close_all_before_any_bar_errors() {
        let mut broker = default_broker();
        let err = broker.close_all(ExitReason::Signal).unwrap_err();
        assert!(matches!(
            err,
            BacktestError::Broker(BrokerError::NoCurrentBar)
        ));
    }

    // ----- Max-drawdown halt -----

    #[test]
    fn max_drawdown_halt_force_closes_and_blocks_new_orders() {
        let mut cfg = BacktestConfig::default();
        cfg.initial_capital = 10_000.0;
        cfg.max_positions = 4;
        cfg.allow_pyramiding = true;
        // 5% drawdown trips the halt.
        cfg.risk_config = RiskConfig::new().with_max_drawdown(5.0);
        let mut broker = fresh_broker(cfg);

        broker.on_bar(&flat(0, 100.0)).unwrap();
        // Buy 50 units (notional 5_000) — half of equity.
        broker
            .place_order(Order::market(OrderSide::Long, 50.0))
            .unwrap();

        // Big drop — equity sinks well below the 5% threshold.
        broker.on_bar(&flat(1, 80.0)).unwrap();
        assert!(broker.risk_halt(), "expected halt to trip");
        assert_eq!(
            broker.positions().len(),
            0,
            "halt should force-close every position"
        );
        assert_eq!(broker.trades().len(), 1);
        assert_eq!(broker.trades()[0].exit_reason, ExitReason::RiskLimit);

        // After the halt, new entries are rejected.
        let err = broker
            .place_order(Order::market(OrderSide::Long, 1.0))
            .unwrap_err();
        assert!(matches!(err, BacktestError::RiskLimitExceeded(_)));
    }

    // ----- Equity curve -----

    #[test]
    fn equity_curve_records_every_bar() {
        let mut broker = default_broker();
        for i in 0..5 {
            broker.on_bar(&flat(i, 100.0 + i as f64)).unwrap();
        }
        assert_eq!(broker.equity_curve().len(), 5);
    }

    // ----- Order kinds -----

    #[test]
    fn limit_order_is_unsupported() {
        let mut broker = default_broker();
        broker.on_bar(&flat(0, 100.0)).unwrap();
        let order = Order {
            side: OrderSide::Long,
            quantity: 1.0,
            kind: OrderKind::Limit { price: 99.0 },
            stop_loss: None,
            take_profit: None,
        };
        let err = broker.place_order(order).unwrap_err();
        assert!(matches!(
            err,
            BacktestError::Broker(BrokerError::UnsupportedOrderKind(_))
        ));
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
