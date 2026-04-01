use super::UiI18n;

pub struct SpanishUi;

impl UiI18n for SpanishUi {
    // Initial signal values
    fn no_key_loaded(&self) -> &'static str { "Ninguna clave cargada" }
    fn copy_label(&self) -> &'static str { "Copiar" }
    fn copy_xdr_label(&self) -> &'static str { "Copiar XDR" }
    fn waiting(&self) -> &'static str { "Esperando..." }
    
    // Status messages
    fn err_no_generated_xdr(&self) -> &'static str { "Error: ¡No hay XDR generado!" }
    fn submitting(&self) -> &'static str { "Enviando..." }
    fn calling_faucet(&self) -> &'static str { "🚀 Llamando al faucet..." }
    fn no_loaded_key(&self) -> &'static str { "⚠️ ¡No hay clave cargada!" }
    fn fetching_sequence(&self) -> &'static str { "🔍 Obteniendo número de secuencia..." }
    
    // Console/println messages
    fn clipboard_cleared(&self) -> &'static str { "🔐 Portapapeles limpiado por seguridad." }
    fn save_success(&self) -> &'static str { "✅ ¡Guardado exitosamente en la billetera del sistema!" }
    fn nothing_to_save(&self) -> &'static str { "⚠️ ¡Nada que guardar (la clave está vacía)!" }
    fn loading_started(&self) -> &'static str { "🔍 Carga iniciada..." }
    fn key_loaded_len(&self, len: usize) -> String { format!("📥 Clave cargada, longitud: {}", len) }
    fn ui_updated_with_key(&self) -> &'static str { "✨ Interfaz actualizada con la clave cargada." }
    
    // Format helpers
    fn fmt_success(&self, msg: &str) -> String { format!("✅ ¡ÉXITO! {}", msg) }
    fn fmt_error(&self, e: &str) -> String { format!("❌ {}", e) }
    fn fmt_xdr_ready(&self, net: &str, seq: i64) -> String { format!("✅ ¡XDR listo! [{}] (Sec: {})", net, seq) }
    
    // Button labels
    fn btn_new_key(&self) -> &'static str { "✨ Nueva clave" }
    fn btn_import(&self) -> &'static str { "📥 Importar" }
    fn btn_hide_secret(&self) -> &'static str { "🙈 Ocultar" }
    fn btn_reveal_secret(&self) -> &'static str { "👁 Revelar" }
    fn btn_activate_faucet(&self) -> &'static str { "🚀 Activar (Faucet)" }
    fn btn_save_to_os(&self) -> &'static str { "💾 Guardar en billetera del SO" }
    fn btn_load(&self) -> &'static str { "🔓 Cargar" }
    fn btn_generate_xdr(&self) -> &'static str { "🛠 Generar transacción XDR" }
    fn btn_submit_tx(&self) -> &'static str { "🚀 ENVIAR Transacción" }
    
    // UI labels
    fn lbl_active_address(&self) -> &'static str { "Dirección activa (clave pública):" }
    fn lbl_signed_xdr(&self) -> &'static str { "XDR FIRMADO:" }
    fn lbl_import_ph(&self) -> &'static str { "Importar (S...)" }
    
    // Network labels
    fn net_testnet_label(&self) -> &'static str { "🧪 Testnet ⚠️" }
    fn net_mainnet_label(&self) -> &'static str { "Mainnet" }
    
    // Clipboard
    fn copied(&self) -> &'static str { "¡COPIADO!" }
    fn clipboard_modal_text(&self) -> &'static str { "El contenido se ha copiado al portapapeles. Al cerrar este diálogo, el portapapeles se limpiará por seguridad." }
    fn btn_clear_clipboard(&self) -> &'static str { "🗑️ Limpiar portapapeles" }

    // Tab labels
    fn tab_cyf(&self) -> &'static str { "CYF" }
    fn tab_networking(&self) -> &'static str { "Red" }
    fn tab_info(&self) -> &'static str { "Info" }
    fn tab_settings(&self) -> &'static str { "Ajustes" }

    // Start gate modal
    fn gate_title(&self) -> &'static str { "Bienvenido a Zsozso" }
    fn btn_next(&self) -> &'static str { "Siguiente" }

    // Passkey authentication
    fn authenticating(&self) -> &'static str { "Autenticando..." }
    fn auth_failed(&self) -> &'static str { "Autenticación fallida" }
    fn btn_exit(&self) -> &'static str { "Salir ahora" }
    fn no_prf_key(&self) -> &'static str { "No hay clave de cifrado de passkey disponible. Vuelva a autenticarse." }

    // Info tab
    fn info_public_key_label(&self) -> &'static str { "Tu clave pública" }
    fn info_no_key(&self) -> &'static str { "No hay clave cargada. Genera o importa una en Ajustes." }

    // Networking tab / Smart Contract
    fn btn_ping(&self) -> &'static str { "\u{1F3D3} Ping" }
    fn ping_calling(&self) -> &'static str { "\u{1F4E1} Llamando al contrato..." }
    fn ping_success(&self, msg: &str) -> String { format!("\u{2705} {}", msg) }
    fn ping_error(&self, e: &str) -> String { format!("\u{274C} {}", e) }
    fn ping_no_key(&self) -> &'static str { "\u{26A0}\u{FE0F} Cargue una clave primero (pestaña Ajustes)." }

    // QR Scanner
    fn btn_scan_qr(&self) -> &'static str { "\u{1F4F7} Escanear QR" }
    fn scan_scanning(&self) -> &'static str { "\u{1F4F7} Escaneando..." }
    fn scan_success(&self, key: &str) -> String { format!("\u{2705} Escaneado: {}", key) }
    fn scan_error(&self, e: &str) -> String { format!("\u{274C} Error al escanear: {}", e) }

    // Log tab
    fn tab_log(&self) -> &'static str { "Registro" }
    fn log_refresh(&self) -> &'static str { "\u{1F504} Actualizar" }
    fn log_clear(&self) -> &'static str { "\u{1F5D1} Limpiar" }
    fn log_save(&self) -> &'static str { "\u{1F4BE} Guardar" }
    fn log_saving(&self) -> &'static str { "Guardando..." }
    fn log_save_ok(&self) -> &'static str { "\u{2705} Registro guardado" }
    fn log_save_fail(&self, e: &str) -> String { format!("\u{274C} Error al guardar: {}", e) }
    fn log_save_empty(&self) -> &'static str { "\u{26A0}\u{FE0F} El registro está vacío" }

    // GUN DB dump
    fn btn_dump_gun_db(&self) -> &'static str { "\u{1F4E6} Exportar GUN DB" }
    fn log_dumping(&self) -> &'static str { "Exportando GUN DB..." }
    fn log_dump_ok(&self) -> &'static str { "\u{2705} GUN DB exportada" }

    // Update toast
    fn toast_update_available(&self) -> &'static str { "\u{1F680} ¡Una nueva versión de Zsozso está disponible!" }
    fn btn_update_now(&self) -> &'static str { "Actualizar ahora" }

    // Info tab – version
    fn info_version(&self, ver: &str) -> String { format!("Versión: {}", ver) }

    // Network switch modal
    fn network_switch_save_prompt(&self) -> &'static str { "\u{00BF}Tiene una clave secreta cargada. Desea guardarla antes de cambiar de red?" }
    fn btn_save_and_switch(&self) -> &'static str { "\u{1F4BE} Guardar y cambiar" }
    fn btn_switch_and_save(&self) -> &'static str { "\u{1F504} Cambiar y guardar" }
    fn btn_switch_without_saving(&self) -> &'static str { "Cambiar sin guardar" }
    fn btn_cancel(&self) -> &'static str { "Cancelar" }

    // SEA key generation modal
    fn btn_generate_db_secret(&self) -> &'static str { "\u{1F511} Generar secreto DB" }
    fn sea_modal_title(&self) -> &'static str { "Generar claves GunDB SEA" }
    fn sea_modal_placeholder(&self) -> &'static str { "Ingrese la frase secreta..." }
    fn btn_generate_db_keys(&self) -> &'static str { "\u{1F511} Generar claves DB" }
    fn sea_generating(&self) -> &'static str { "\u{1F504} Generando claves..." }
    fn sea_keys_generated(&self) -> &'static str { "\u{2705} Claves SEA generadas y cargadas en memoria." }
    fn sea_generation_error(&self, e: &str) -> String { format!("\u{274C} Error al generar claves: {}", e) }
    fn btn_close(&self) -> &'static str { "Cerrar" }

    // Biometric identification toggle
    fn lbl_biometric(&self) -> &'static str { "Identificación biométrica" }
    fn lbl_biometric_desc(&self) -> &'static str { "Usar autenticación biométrica para proteger su billetera" }
    fn biometric_required_to_save(&self) -> &'static str { "Por favor, active la Identificación biométrica en Ajustes antes de guardar su secreto." }

    // Nickname (Settings)
    fn lbl_nickname(&self) -> &'static str { "Apodo" }
    fn lbl_nickname_ph(&self) -> &'static str { "Ingrese su apodo..." }
    fn btn_save_nickname(&self) -> &'static str { "\u{1F4BE} Guardar" }
    fn nickname_saved(&self) -> &'static str { "\u{2705} ¡Apodo guardado!" }
    fn nickname_save_error(&self, e: &str) -> String { format!("\u{274C} Error al guardar el apodo: {}", e) }

    // Network tab – hierarchy
    fn lbl_parents(&self) -> &'static str { "Padres" }
    fn lbl_workers(&self) -> &'static str { "Trabajadores" }
    fn btn_new_worker(&self) -> &'static str { "\u{2795} Nuevo" }
    fn network_no_key(&self) -> &'static str { "\u{26A0}\u{FE0F} Cargue una clave primero (pestaña Ajustes)." }
    fn network_add_worker_success(&self, key: &str) -> String { format!("\u{2705} Trabajador añadido: {}", key) }
    fn network_add_worker_error(&self, e: &str) -> String { format!("\u{274C} Error al añadir trabajador: {}", e) }

    // CYF tab
    fn btn_burn(&self) -> &'static str { "\u{1F525} Quemar" }
    fn btn_mint(&self) -> &'static str { "\u{1FA99} Acuñar" }
    fn btn_ok(&self) -> &'static str { "OK" }
    fn cyf_not_implemented(&self, fn_name: &str) -> String { format!("La función {} aún no está implementada.", fn_name) }

    // GUN node address
    fn lbl_gun_address(&self) -> &'static str { "Dirección del nodo GUN" }
    fn lbl_gun_relay_url(&self) -> &'static str { "URL del relé GUN" }
    fn lbl_gun_relay_ph(&self) -> &'static str { "https://your-server.com/gun" }
    fn btn_save_gun_relay(&self) -> &'static str { "Guardar" }

    // SSS (Shamir's Secret Sharing)
    fn sss_modal_title(&self) -> &'static str { "\u{1F512} Partes de recuperación" }
    fn sss_modal_description(&self) -> &'static str { "Distribuye estas partes a tus nodos de confianza. 3 de 7 son suficientes para reconstruir tu secreto." }
    fn sss_share_label(&self, n: usize) -> String { format!("Parte {}", n) }
    fn btn_copy_share(&self) -> &'static str { "\u{1F4CB} Copiar" }
    fn sss_share_copied(&self) -> &'static str { "\u{2705} ¡Copiado!" }

    // ZS (ZSOZSO) tab
    fn tab_zsozso(&self) -> &'static str { "ZS" }
    fn lbl_xlm_balance(&self) -> &'static str { "XLM" }
    fn lbl_zsozso_balance(&self) -> &'static str { "ZSOZSO" }
    fn lbl_locked_zsozso(&self) -> &'static str { "ZSOZSO bloqueado" }
    fn btn_lock(&self) -> &'static str { "\u{1F512} Bloquear" }
    fn btn_unlock(&self) -> &'static str { "\u{1F513} Desbloquear" }
    fn btn_refresh_balances(&self) -> &'static str { "\u{1F504} Actualizar" }
    fn zs_fetching_balances(&self) -> &'static str { "\u{1F504} Obteniendo saldos..." }
    fn zs_mainnet_only(&self) -> &'static str { "ZSOZSO solo está disponible en Mainnet. Cambie a Mainnet en Ajustes." }
    fn zs_no_key(&self) -> &'static str { "Cargue una clave primero en Ajustes." }
    fn zs_locking(&self) -> &'static str { "\u{1F512} Bloqueando ZSOZSO..." }
    fn zs_unlocking(&self) -> &'static str { "\u{1F513} Desbloqueando ZSOZSO..." }
    fn fmt_zs_lock_success(&self, amount: &str) -> String { format!("\u{2705} {} ZSOZSO bloqueado", amount) }
    fn fmt_zs_unlock_success(&self, amount: &str) -> String { format!("\u{2705} {} ZSOZSO desbloqueado", amount) }
    fn fmt_zs_lock_error(&self, err: &str) -> String { format!("\u{274C} Bloqueo fallido: {}", err) }
    fn fmt_zs_unlock_error(&self, err: &str) -> String { format!("\u{274C} Desbloqueo fallido: {}", err) }
    fn zs_invalid_amount(&self) -> &'static str { "Por favor, ingrese un monto válido." }
    fn lbl_amount(&self) -> &'static str { "Monto" }
    fn lbl_mainnet_account(&self) -> &'static str { "Cuenta Mainnet" }
    fn lbl_testnet_account(&self) -> &'static str { "Cuenta Testnet" }
    fn lbl_no_account(&self) -> &'static str { "Sin cuenta" }

    fn lbl_mainnet_keys(&self) -> &'static str { "Clave Mainnet" }
    fn lbl_testnet_keys(&self) -> &'static str { "Clave Testnet" }
    fn lbl_pin_code(&self) -> &'static str { "Código PIN" }
    fn lbl_pin_code_desc(&self) -> &'static str { "Establezca un código PIN para proteger su billetera (solo localhost)" }
    fn lbl_pin_code_ph(&self) -> &'static str { "Ingrese PIN..." }
    fn btn_set_pin(&self) -> &'static str { "Establecer PIN" }
}
