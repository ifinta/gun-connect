use super::UiI18n;
use zsozso_common::{Language, I18nLanguage};
use zsozso_common::ui_i18n::*;

pub struct SpanishUi;

impl I18nLanguage for SpanishUi {
    fn language(&self) -> Language { Language::Spanish }
}

impl CoreI18n for SpanishUi {
    fn toast_update_available(&self) -> &'static str { "\u{1F680} ¡Una nueva versión de Gun Connect está disponible!" }
}
impl AuthI18n for SpanishUi {
    fn gate_title(&self) -> &'static str { "Bienvenido a Gun Connect" }
}
impl KeysI18n for SpanishUi {}
impl ClipboardI18n for SpanishUi {}
impl StoreUiI18n for SpanishUi {}
impl StellarUiI18n for SpanishUi {}
impl SeaUiI18n for SpanishUi {}
impl QrI18n for SpanishUi {}
impl SssI18n for SpanishUi {}
impl LogI18n for SpanishUi {}
impl MlmI18n for SpanishUi {}
impl CyfI18n for SpanishUi {}
impl ZsI18n for SpanishUi {}

impl UiI18n for SpanishUi {
    fn relay_status_connected(&self) -> &'static str { "Conectado" }
    fn relay_status_unreachable(&self) -> &'static str { "Inalcanzable" }
    fn relay_status_checking(&self) -> &'static str { "Verificando..." }
    fn relay_status_not_configured(&self) -> &'static str { "Sin relé configurado" }
    fn btn_check_relay(&self) -> &'static str { "Verificar" }
    fn lbl_relay_status(&self) -> &'static str { "Estado del Relé" }
    fn btn_find_relays(&self) -> &'static str { "Buscar Relés" }
    fn lbl_discovered_relays(&self) -> &'static str { "Relés Descubiertos" }
    fn relay_discovering(&self) -> &'static str { "Buscando..." }
    fn btn_connect_relay(&self) -> &'static str { "Conectar" }
    fn btn_remove_relay(&self) -> &'static str { "Eliminar" }
}
