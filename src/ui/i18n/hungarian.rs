use super::UiI18n;
use zsozso_common::{Language, I18nLanguage};
use zsozso_common::ui_i18n::*;

pub struct HungarianUi;

impl I18nLanguage for HungarianUi {
    fn language(&self) -> Language { Language::Hungarian }
}

impl CoreI18n for HungarianUi {}
impl AuthI18n for HungarianUi {}
impl KeysI18n for HungarianUi {}
impl ClipboardI18n for HungarianUi {}
impl StoreUiI18n for HungarianUi {}
impl StellarUiI18n for HungarianUi {}
impl SeaUiI18n for HungarianUi {}
impl QrI18n for HungarianUi {}
impl SssI18n for HungarianUi {}
impl LogI18n for HungarianUi {}
impl MlmI18n for HungarianUi {}
impl CyfI18n for HungarianUi {}
impl ZsI18n for HungarianUi {}

impl UiI18n for HungarianUi {
    fn relay_status_connected(&self) -> &'static str { "Csatlakozva" }
    fn relay_status_unreachable(&self) -> &'static str { "Nem elérhető" }
    fn relay_status_checking(&self) -> &'static str { "Ellenőrzés..." }
    fn relay_status_not_configured(&self) -> &'static str { "Nincs relay beállítva" }
    fn btn_check_relay(&self) -> &'static str { "Ellenőrzés" }
    fn lbl_relay_status(&self) -> &'static str { "Relay Állapot" }
    fn btn_find_relays(&self) -> &'static str { "Relay-ek Keresése" }
    fn lbl_discovered_relays(&self) -> &'static str { "Talált Relay-ek" }
    fn relay_discovering(&self) -> &'static str { "Keresés..." }
    fn btn_connect_relay(&self) -> &'static str { "Csatlakozás" }
    fn btn_remove_relay(&self) -> &'static str { "Eltávolítás" }
}
