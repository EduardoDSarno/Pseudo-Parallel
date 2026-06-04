use crate::market_data::types::CandleKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IndicatorRuleId(pub u64);

#[derive(Debug, Clone)]
pub struct IndicatorRule {
    pub id: IndicatorRuleId,
    pub key: CandleKey,
    pub kind: IndicatorRuleKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IndicatorRuleKind {
    Atr(AtrRule),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtrRule {
    pub breakout_ratio: f64,
    pub debug_ratio: f64,
}

/* Struct used for subscriptions */
#[derive(Debug, Clone)]
pub struct Indicator {
    pub(crate) key: CandleKey,
    pub(crate) kind: IndicatorRuleKind,
}

impl Indicator {
    pub fn new(key: CandleKey, kind: IndicatorRuleKind) -> Self {
        Indicator { key, kind }
    }
}
