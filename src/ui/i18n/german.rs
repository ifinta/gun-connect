use super::UiI18n;
use zsozso_common::{Language, I18nLanguage};
use zsozso_common::ui_i18n::*;

pub struct GermanUi;

impl I18nLanguage for GermanUi {
    fn language(&self) -> Language { Language::German }
}

impl CoreI18n for GermanUi {
    fn toast_update_available(&self) -> &'static str { "\u{1F680} Eine neue Version von Gun Connect ist verfügbar!" }
}
impl AuthI18n for GermanUi {
    fn gate_title(&self) -> &'static str { "Willkommen bei Gun Connect" }
}
impl KeysI18n for GermanUi {}
impl ClipboardI18n for GermanUi {}
impl StoreUiI18n for GermanUi {}
impl StellarUiI18n for GermanUi {}
impl SeaUiI18n for GermanUi {}
impl QrI18n for GermanUi {}
impl SssI18n for GermanUi {}
impl LogI18n for GermanUi {}
impl MlmI18n for GermanUi {}
impl CyfI18n for GermanUi {}
impl ZsI18n for GermanUi {}

impl UiI18n for GermanUi {
    fn relay_status_connected(&self) -> &'static str { "Verbunden" }
    fn relay_status_unreachable(&self) -> &'static str { "Nicht erreichbar" }
    fn relay_status_checking(&self) -> &'static str { "Prüfe..." }
    fn relay_status_not_configured(&self) -> &'static str { "Kein Relay konfiguriert" }
    fn btn_check_relay(&self) -> &'static str { "Prüfen" }
    fn lbl_relay_status(&self) -> &'static str { "Relay-Status" }
    fn btn_find_relays(&self) -> &'static str { "Relays Finden" }
    fn lbl_discovered_relays(&self) -> &'static str { "Gefundene Relays" }
    fn relay_discovering(&self) -> &'static str { "Suche..." }
    fn btn_connect_relay(&self) -> &'static str { "Verbinden" }
    fn btn_remove_relay(&self) -> &'static str { "Entfernen" }
    fn btn_forget_relay(&self) -> &'static str { "Vergessen" }
    fn btn_stop_search(&self) -> &'static str { "Stopp" }
}
