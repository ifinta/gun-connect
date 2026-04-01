use super::i18n::UiI18n;

#[derive(Clone)]
pub enum TxStatus {
    Waiting,
    Submitting,
    CallingFaucet,
    FetchingSequence,
    NoKey,
    NoXdr,
    XdrReady { net: String, seq: i64 },
    Success(String),
    Error(String),
    FaucetSuccess(String),
}

pub fn status_text(status: &TxStatus, i18n: &dyn UiI18n) -> String {
    match status {
        TxStatus::Waiting => i18n.waiting().to_string(),
        TxStatus::Submitting => i18n.submitting().to_string(),
        TxStatus::CallingFaucet => i18n.calling_faucet().to_string(),
        TxStatus::FetchingSequence => i18n.fetching_sequence().to_string(),
        TxStatus::NoKey => i18n.no_loaded_key().to_string(),
        TxStatus::NoXdr => i18n.err_no_generated_xdr().to_string(),
        TxStatus::XdrReady { net, seq } => i18n.fmt_xdr_ready(net, *seq),
        TxStatus::Success(msg) => i18n.fmt_success(msg),
        TxStatus::Error(e) => i18n.fmt_error(e),
        TxStatus::FaucetSuccess(msg) => format!("✅ {}", msg),
    }
}
