use std::collections::{HashMap, hash_map::Entry};

use hypersdk::Address;
use tokio::time::Instant;

use crate::{coin::Coin, config::ADDRESS_REFRESH_STATE_COOLDOWN};

/// Converts a typed Hyperliquid address for display or text storage.
pub fn format_address(address: Address) -> String {
    address.to_string()
}

/// Requests an authoritative account lookup for one coin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountLookupRequest {
    pub address: Address,
    pub coin: Coin,
}

/// AddressRefreshAction tells the application what to do after an address appears in a trade.
/// It represents an action, not the permanent state of the address.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddressRefreshAction {
    /// Request now flag is set when an accout can be requested immiediatly
    /// This happens when, the address is discovered for the first time or
    /// the address is known, but its cooldown has already expired.
    RequestNow,

    /// Account can not be request yet because it is on a CoolDown, the instant is the
    /// exact time when the cooldown will finish
    ScheduleAt(Instant),
    /// No new work needs to be created.
    /// means the address is still inside its cooldown or
    /// a trailing refresh is already scheduled.
    Nothing,
}

pub struct AddressRefreshState {
    last_requested_at: Instant,
    needs_refresh: bool,
}

impl AddressRefreshState {
    pub fn new() -> Self {
        Self {
            last_requested_at: Instant::now(),
            needs_refresh: false,
        }
    }

    /// Decides whether current activity should request an account immediately,
    /// schedule one trailing refresh, or reuse an existing scheduled refresh.
    pub fn refresh(&mut self) -> AddressRefreshAction {
        self.refresh_at(Instant::now())
    }

    fn refresh_at(&mut self, now: Instant) -> AddressRefreshAction {
        if now.duration_since(self.last_requested_at) < ADDRESS_REFRESH_STATE_COOLDOWN {
            if self.needs_refresh {
                return AddressRefreshAction::Nothing;
            }

            self.needs_refresh = true;
            return AddressRefreshAction::ScheduleAt(
                self.last_requested_at + ADDRESS_REFRESH_STATE_COOLDOWN,
            );
        }

        self.last_requested_at = now;
        self.needs_refresh = false;
        AddressRefreshAction::RequestNow
    }

    /// Returns true when a scheduled trailing refresh is still needed and due.
    /// Taking it starts a new cooldown.
    pub fn take_due_refresh(&mut self) -> bool {
        self.take_due_refresh_at(Instant::now())
    }

    fn take_due_refresh_at(&mut self, now: Instant) -> bool {
        if !self.needs_refresh
            || now.duration_since(self.last_requested_at) < ADDRESS_REFRESH_STATE_COOLDOWN
        {
            return false;
        }

        self.last_requested_at = now;
        self.needs_refresh = false;
        true
    }
}

/// Owns the refresh state for every address discovered from market trades.
pub struct AddressRefreshRegistry {
    addresses: HashMap<Address, AddressRefreshState>,
}

impl AddressRefreshRegistry {
    pub fn new() -> Self {
        Self {
            addresses: HashMap::new(),
        }
    }

    /// Records activity for an address and returns the refresh action to take.
    pub fn register_activity(&mut self, address: Address) -> AddressRefreshAction {
        match self.addresses.entry(address) {
            // A new address always receives its initial lookup.
            Entry::Vacant(entry) => {
                entry.insert(AddressRefreshState::new());
                AddressRefreshAction::RequestNow
            }
            // A known address may be requested now, scheduled once, or already
            // represented in the delayed queue.
            Entry::Occupied(mut entry) => entry.get_mut().refresh(),
        }
    }

    /// Takes a scheduled refresh only when the address still exists and is due.
    pub fn take_due_refresh(&mut self, address: &Address) -> bool {
        self.addresses
            .get_mut(address)
            .is_some_and(AddressRefreshState::take_due_refresh)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{AddressRefreshAction, AddressRefreshState};
    use crate::config::ADDRESS_REFRESH_STATE_COOLDOWN;

    #[test]
    fn takes_one_trailing_refresh_after_the_cooldown() {
        let started_at = tokio::time::Instant::now();
        let mut state = AddressRefreshState {
            last_requested_at: started_at,
            needs_refresh: false,
        };
        let just_before_cooldown =
            started_at + ADDRESS_REFRESH_STATE_COOLDOWN - Duration::from_millis(1);
        let cooldown_finished = started_at + ADDRESS_REFRESH_STATE_COOLDOWN;

        assert_eq!(
            state.refresh_at(just_before_cooldown),
            AddressRefreshAction::ScheduleAt(cooldown_finished)
        );
        assert_eq!(
            state.refresh_at(just_before_cooldown),
            AddressRefreshAction::Nothing
        );
        assert!(!state.take_due_refresh_at(just_before_cooldown));
        assert!(state.take_due_refresh_at(cooldown_finished));
        assert!(!state.take_due_refresh_at(cooldown_finished + ADDRESS_REFRESH_STATE_COOLDOWN));
    }
}
