use super::UiI18n;
use zsozso_common::{Language, I18nLanguage};
use zsozso_common::ui_i18n::*;

pub struct FrenchUi;

impl I18nLanguage for FrenchUi {
    fn language(&self) -> Language { Language::French }
}

impl CoreI18n for FrenchUi {}
impl AuthI18n for FrenchUi {}
impl KeysI18n for FrenchUi {}
impl ClipboardI18n for FrenchUi {}
impl StoreUiI18n for FrenchUi {}
impl StellarUiI18n for FrenchUi {}
impl SeaUiI18n for FrenchUi {}
impl QrI18n for FrenchUi {}
impl SssI18n for FrenchUi {}
impl LogI18n for FrenchUi {}
impl MlmI18n for FrenchUi {}
impl CyfI18n for FrenchUi {}
impl ZsI18n for FrenchUi {}

impl UiI18n for FrenchUi {
    fn relay_status_connected(&self) -> &'static str { "Connecté" }
    fn relay_status_unreachable(&self) -> &'static str { "Injoignable" }
    fn relay_status_checking(&self) -> &'static str { "Vérification..." }
    fn relay_status_not_configured(&self) -> &'static str { "Aucun relais configuré" }
    fn btn_check_relay(&self) -> &'static str { "Vérifier" }
    fn lbl_relay_status(&self) -> &'static str { "État du Relais" }
    fn btn_find_relays(&self) -> &'static str { "Trouver des Relais" }
    fn lbl_discovered_relays(&self) -> &'static str { "Relais Découverts" }
    fn relay_discovering(&self) -> &'static str { "Recherche..." }
    fn relay_no_results(&self) -> &'static str { "Aucun relais trouvé sur testnet" }
}
