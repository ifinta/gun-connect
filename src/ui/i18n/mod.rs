mod english;
mod french;
mod german;
mod hungarian;
mod spanish;

use zsozso_common::Language;
use english::EnglishUi;
use french::FrenchUi;
use german::GermanUi;
use hungarian::HungarianUi;
use spanish::SpanishUi;

pub use zsozso_common::ui_i18n::{
    CoreI18n, AuthI18n, KeysI18n, ClipboardI18n, StoreUiI18n,
    StellarUiI18n, SeaUiI18n, QrI18n, SssI18n, LogI18n,
    MlmI18n, CyfI18n, ZsI18n,
};

/// Trait for UI-related internationalized strings.
/// Composes grouped traits from zsozso-common + app-specific relay methods.
pub trait UiI18n: CoreI18n + AuthI18n + KeysI18n + ClipboardI18n + StoreUiI18n
    + StellarUiI18n + SeaUiI18n + QrI18n + SssI18n + LogI18n
    + MlmI18n + CyfI18n + ZsI18n
{
    // Relay status (gun-connect specific)
    fn relay_status_connected(&self) -> &'static str;
    fn relay_status_unreachable(&self) -> &'static str;
    fn relay_status_checking(&self) -> &'static str;
    fn relay_status_not_configured(&self) -> &'static str;
    fn btn_check_relay(&self) -> &'static str;
    fn lbl_relay_status(&self) -> &'static str;
    fn btn_find_relays(&self) -> &'static str;
    fn lbl_discovered_relays(&self) -> &'static str;
    fn relay_discovering(&self) -> &'static str;
    fn relay_no_results(&self) -> &'static str;
}

/// Factory function to get the appropriate UiI18n implementation
pub fn ui_i18n(lang: Language) -> Box<dyn UiI18n> {
    match lang {
        Language::English => Box::new(EnglishUi),
        Language::French => Box::new(FrenchUi),
        Language::German => Box::new(GermanUi),
        Language::Hungarian => Box::new(HungarianUi),
        Language::Spanish => Box::new(SpanishUi),
        _ => Box::new(EnglishUi),
    }
}
