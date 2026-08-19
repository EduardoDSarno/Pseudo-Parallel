use std::{collections::BTreeMap, future};

use tokio::time::{Instant, sleep_until};

use crate::accounts::AccountLookupRequest;

/// Keeps future account refreshes ordered by their execution deadline.
pub struct AccountRefreshScheduler {
    scheduled_refreshes: BTreeMap<Instant, Vec<AccountLookupRequest>>,
}

impl AccountRefreshScheduler {
    pub fn new() -> Self {
        Self {
            scheduled_refreshes: BTreeMap::new(),
        }
    }

    /// Adds a request to the group scheduled for this exact deadline.
    pub fn schedule(&mut self, deadline: Instant, request: AccountLookupRequest) {
        self.scheduled_refreshes
            .entry(deadline)
            .or_default()
            .push(request);
    }

    /// Returns the earliest scheduled deadline without removing it.
    pub fn next_deadline(&self) -> Option<Instant> {
        self.scheduled_refreshes
            .first_key_value()
            .map(|(deadline, _)| *deadline)
    }

    /// Waits for the earliest deadline, or forever when nothing is scheduled.
    pub async fn wait_until(next_refresh_at: Option<Instant>) {
        match next_refresh_at {
            Some(deadline) => sleep_until(deadline).await,
            None => future::pending().await,
        }
    }

    /// Removes and returns every request whose deadline has passed.
    pub fn take_due(&mut self, now: Instant) -> Vec<AccountLookupRequest> {
        let mut due_requests = Vec::new();

        while self
            .scheduled_refreshes
            .first_key_value()
            .is_some_and(|(deadline, _)| *deadline <= now)
        {
            let (_, scheduled_requests) = self
                .scheduled_refreshes
                .pop_first()
                .expect("the earliest scheduled refresh should exist");

            due_requests.extend(scheduled_requests);
        }

        due_requests
    }
}
