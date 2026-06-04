use std::{collections::HashMap, error::Error};

use crate::market_data::{
    signal::indicator_rules::{AtrRule, IndicatorRule, IndicatorRuleId, IndicatorRuleKind},
    types::CandleKey,
};

pub struct IndicatorRuleService {
    rules_by_key: HashMap<CandleKey, Vec<IndicatorRule>>,
    next_id: u64,
}

impl IndicatorRuleService {
    pub fn new() -> Self {
        IndicatorRuleService {
            rules_by_key: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn subscribe(&mut self, key: CandleKey, kind: IndicatorRuleKind) -> IndicatorRule {
        let rule = IndicatorRule 
        {
            id: self.next_rule_id(),
            key: key.clone(),
            kind,
        };

        self.rules_by_key.entry(key).or_default().push(rule.clone());

        rule
    }

    pub fn unsubscribe(
        &mut self,
        key: CandleKey,
        kind: IndicatorRuleKind,
    ) -> Result<IndicatorRule, Box<dyn Error>> {
        let rules = self
            .rules_by_key
            .get_mut(&key)
            .ok_or_else(|| -> Box<dyn Error> { "indicator rule not found".into() })?;

        let index = rules
            .iter()
            .position(|rule| rule.kind == kind)
            .ok_or_else(|| -> Box<dyn Error> { "indicator rule not found".into() })?;

        let rule = rules.remove(index);
        if rules.is_empty() {
            self.rules_by_key.remove(&key);
        }

        tracing::debug!(?rule.id, ?key, "indicator rule unsubscribed");
        Ok(rule)
    }

    pub fn subscribe_default_atr_rules(
        &mut self,
        candle_keys: &[CandleKey],
        breakout_ratio: f64,
        debug_ratio: f64,
    ) -> Vec<IndicatorRuleId> {
        candle_keys
            .iter()
            .map(|key| {
                self.subscribe(
                    key.clone(),
                    IndicatorRuleKind::Atr(AtrRule {
                        breakout_ratio,
                        debug_ratio,
                    }),
                )
                .id
            })
            .collect()
    }

    pub fn rules_for_key(&self, key: &CandleKey) -> Vec<IndicatorRule> {
        self.rules_by_key.get(key).cloned().unwrap_or_default()
    }

    fn next_rule_id(&mut self) -> IndicatorRuleId {
        let id = IndicatorRuleId(self.next_id);
        self.next_id += 1;
        id
    }
}
