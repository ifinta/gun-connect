#[derive(Clone)]
pub enum TxStatus {
    Waiting,
    Submitting,
    CallingFaucet,
    Success(String),
    Error(String),
    FaucetSuccess(String),
}
