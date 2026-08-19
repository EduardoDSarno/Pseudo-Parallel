use tokio::sync::mpsc::Sender;

use crate::accounts::AccountLookupRequest;

/// Sends a request to the account lookup task and fails if that task has ended.
pub async fn send_account_lookup_request(
    account_lookup_tx: &Sender<AccountLookupRequest>,
    request: AccountLookupRequest,
) {
    account_lookup_tx
        .send(request)
        .await
        .expect("account lookup task should remain active");
}
