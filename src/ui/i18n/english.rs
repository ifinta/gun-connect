use super::UiI18n;
use zsozso_common::{Language, I18nLanguage};
use zsozso_common::ui_i18n::*;

pub struct EnglishUi;

impl I18nLanguage for EnglishUi {
    fn language(&self) -> Language { Language::English }
}

impl CoreI18n for EnglishUi {}
impl AuthI18n for EnglishUi {}
impl KeysI18n for EnglishUi {}
impl ClipboardI18n for EnglishUi {}
impl StoreUiI18n for EnglishUi {}
impl StellarUiI18n for EnglishUi {}
impl SeaUiI18n for EnglishUi {}
impl QrI18n for EnglishUi {}
impl SssI18n for EnglishUi {}
impl LogI18n for EnglishUi {}
impl MlmI18n for EnglishUi {}
impl CyfI18n for EnglishUi {}
impl ZsI18n for EnglishUi {}

impl UiI18n for EnglishUi {
    fn relay_status_connected(&self) -> &'static str { "Connected" }
    fn relay_status_unreachable(&self) -> &'static str { "Unreachable" }
    fn relay_status_checking(&self) -> &'static str { "Checking..." }
    fn relay_status_not_configured(&self) -> &'static str { "No relay configured" }
    fn btn_check_relay(&self) -> &'static str { "Check" }
    fn lbl_relay_status(&self) -> &'static str { "Relay Status" }
    fn btn_find_relays(&self) -> &'static str { "Find Relays" }
    fn lbl_discovered_relays(&self) -> &'static str { "Discovered Relays" }
    fn relay_discovering(&self) -> &'static str { "Searching..." }
    fn relay_no_results(&self) -> &'static str { "No relays found on testnet" }
}
