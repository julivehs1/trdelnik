//! # Trdelnik Broker
//!
//! Broker abstraction shared by every order-execution backend in trdelnik
//! (backtest engine, paper trading, future live broker integrations).
//! A `Broker<X>` exposes order submission, position/trade inspection, and
//! a per-bar advance hook. Strategy code talks to the trait — what's on
//! the other side (a simulated state machine, a paper-trading sandbox,
//! or an exchange API) is an implementation detail.
//!
//! Each backend keeps its own `Position`/`Trade` types via associated
//! types, so backtest and live brokers don't have to agree on every
//! field they track.

use std::fmt::Debug;

use trdelnik_core::{AxisCoordinate, Candle};

/// Identifier for a submitted order, returned by [`Broker::place_order`].
///
/// Brokers assign these monotonically. `OrderId` is opaque — callers
/// hold onto it to cancel the order or correlate fills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OrderId(pub u64);

impl OrderId {
    /// Construct an `OrderId` from its raw value.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Raw value.
    pub fn raw(self) -> u64 {
        self.0
    }
}

/// Direction of an order. Mirrors the `PositionSide` used elsewhere but
/// avoids a cross-crate dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    /// Buy / long.
    Long,
    /// Sell / short.
    Short,
}

impl OrderSide {
    /// `+1.0` for long, `-1.0` for short.
    pub fn direction(self) -> f64 {
        match self {
            OrderSide::Long => 1.0,
            OrderSide::Short => -1.0,
        }
    }
}

/// Order type. Backtest currently fills [`OrderKind::Market`] only;
/// limit/stop/stop-limit are part of the API so live brokers and a
/// future, more realistic backtest engine can support them without an
/// API break.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderKind {
    /// Fill at the broker's "current" price (close of the current bar
    /// for backtesting, last quote for paper/live).
    Market,
    /// Fill when the price reaches `price` or better.
    Limit { price: f64 },
    /// Trigger a market order when `price` is touched.
    Stop { price: f64 },
    /// Trigger a limit order at `limit` when `stop` is touched.
    StopLimit { stop: f64, limit: f64 },
}

/// Order ready for submission to a broker.
#[derive(Debug, Clone)]
pub struct Order {
    /// Long or short.
    pub side: OrderSide,
    /// Quantity (shares/contracts/units; semantics are broker-defined).
    pub quantity: f64,
    /// Order type.
    pub kind: OrderKind,
    /// Optional protective stop attached to the resulting position.
    pub stop_loss: Option<f64>,
    /// Optional take-profit attached to the resulting position.
    pub take_profit: Option<f64>,
}

impl Order {
    /// Convenience constructor for a market order with no stops.
    pub fn market(side: OrderSide, quantity: f64) -> Self {
        Self {
            side,
            quantity,
            kind: OrderKind::Market,
            stop_loss: None,
            take_profit: None,
        }
    }

    /// Attach a stop-loss price.
    pub fn with_stop_loss(mut self, price: f64) -> Self {
        self.stop_loss = Some(price);
        self
    }

    /// Attach a take-profit price.
    pub fn with_take_profit(mut self, price: f64) -> Self {
        self.take_profit = Some(price);
        self
    }
}

/// Errors returned by `Broker` operations. Backends extend this with
/// their own variants by implementing `From`.
#[derive(Debug, thiserror::Error)]
pub enum BrokerError {
    /// The order kind is recognised but the backend hasn't implemented it.
    #[error("order kind not supported by this broker: {0:?}")]
    UnsupportedOrderKind(OrderKind),
    /// Quantity must be positive.
    #[error("invalid order quantity: {0}")]
    InvalidQuantity(f64),
    /// Backend ran out of cash to open this position.
    #[error("insufficient funds: required {required}, available {available}")]
    InsufficientFunds {
        /// Cash required to fill the order.
        required: f64,
        /// Cash currently available.
        available: f64,
    },
    /// `OrderId` is not known to this broker.
    #[error("unknown order id: {0:?}")]
    UnknownOrder(OrderId),
    /// Broker has not yet seen a bar — `place_order(Market)` needs a
    /// current price to fill against. Call `on_bar` first.
    #[error("broker has no current bar; call on_bar before submitting market orders")]
    NoCurrentBar,
    /// Catch-all for backend-specific failures.
    #[error("broker error: {0}")]
    Other(String),
}

/// A broker's view of its positions and trades.
///
/// `X` is the chart's X-axis coordinate (timestamp, slot, block, …).
/// Implementations choose their own concrete `Position` and `Trade`
/// types via the associated types.
pub trait Broker<X: AxisCoordinate>: Debug {
    /// Position type carried by this backend.
    type Position;
    /// Closed-trade record produced by this backend.
    type Trade;
    /// Backend-specific error type. Must be convertible from
    /// `BrokerError` so generic code can return the common errors.
    type Error: From<BrokerError> + Debug;

    /// Submit an order. Market orders fill immediately at the broker's
    /// current price; non-market orders are queued and inspected on each
    /// `on_bar` call.
    fn place_order(&mut self, order: Order) -> Result<OrderId, Self::Error>;

    /// Cancel a pending order. Returns `Ok` whether or not the order was
    /// already filled, but errors if the id is unknown.
    fn cancel_order(&mut self, order_id: OrderId) -> Result<(), Self::Error>;

    /// Advance the broker's clock to the given candle. Backends use this
    /// to fire SL/TP, fill pending limit/stop orders, update unrealised
    /// P&L, and record equity.
    fn on_bar(&mut self, candle: &Candle<X>) -> Result<(), Self::Error>;

    /// Currently open positions.
    fn positions(&self) -> &[Self::Position];

    /// Closed trades since the broker was created.
    fn trades(&self) -> &[Self::Trade];

    /// Available cash.
    fn cash(&self) -> f64;

    /// Total equity (cash + open-position market value).
    fn equity(&self) -> f64;
}
