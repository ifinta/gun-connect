use super::UiI18n;

pub struct GermanUi;

impl UiI18n for GermanUi {
    // Initial signal values
    fn no_key_loaded(&self) -> &'static str { "Kein Schlüssel geladen" }
    fn copy_label(&self) -> &'static str { "Kopieren" }
    fn copy_xdr_label(&self) -> &'static str { "XDR kopieren" }
    fn waiting(&self) -> &'static str { "Warten..." }
    
    // Status messages
    fn err_no_generated_xdr(&self) -> &'static str { "Fehler: Kein generiertes XDR!" }
    fn submitting(&self) -> &'static str { "Wird gesendet..." }
    fn calling_faucet(&self) -> &'static str { "🚀 Faucet wird aufgerufen..." }
    fn no_loaded_key(&self) -> &'static str { "⚠️ Kein Schlüssel geladen!" }
    fn fetching_sequence(&self) -> &'static str { "🔍 Sequenznummer wird abgerufen..." }
    
    // Console/println messages
    fn clipboard_cleared(&self) -> &'static str { "🔐 Zwischenablage aus Sicherheitsgründen geleert." }
    fn save_success(&self) -> &'static str { "✅ Erfolgreich in der System-Wallet gespeichert!" }
    fn nothing_to_save(&self) -> &'static str { "⚠️ Nichts zu speichern (Schlüssel ist leer)!" }
    fn loading_started(&self) -> &'static str { "🔍 Laden gestartet..." }
    fn key_loaded_len(&self, len: usize) -> String { format!("📥 Schlüssel geladen, Länge: {}", len) }
    fn ui_updated_with_key(&self) -> &'static str { "✨ Oberfläche erfolgreich mit geladenem Schlüssel aktualisiert." }
    
    // Format helpers
    fn fmt_success(&self, msg: &str) -> String { format!("✅ ERFOLG! {}", msg) }
    fn fmt_error(&self, e: &str) -> String { format!("❌ {}", e) }
    fn fmt_xdr_ready(&self, net: &str, seq: i64) -> String { format!("✅ XDR bereit! [{}] (Seq: {})", net, seq) }
    
    // Button labels
    fn btn_new_key(&self) -> &'static str { "✨ Neuer Schlüssel" }
    fn btn_import(&self) -> &'static str { "📥 Importieren" }
    fn btn_hide_secret(&self) -> &'static str { "🙈 Verbergen" }
    fn btn_reveal_secret(&self) -> &'static str { "👁 Anzeigen" }
    fn btn_activate_faucet(&self) -> &'static str { "🚀 Aktivieren (Faucet)" }
    fn btn_save_to_os(&self) -> &'static str { "💾 In OS-Wallet speichern" }
    fn btn_load(&self) -> &'static str { "🔓 Laden" }
    fn btn_generate_xdr(&self) -> &'static str { "🛠 Transaktions-XDR generieren" }
    fn btn_submit_tx(&self) -> &'static str { "🚀 Transaktion SENDEN" }
    
    // UI labels
    fn lbl_active_address(&self) -> &'static str { "Aktive Adresse (Öffentlicher Schlüssel):" }
    fn lbl_signed_xdr(&self) -> &'static str { "SIGNIERTES XDR:" }
    fn lbl_import_ph(&self) -> &'static str { "Importieren (S...)" }
    
    // Network labels
    fn net_testnet_label(&self) -> &'static str { "🧪 Testnet ⚠️" }
    fn net_mainnet_label(&self) -> &'static str { "Mainnet" }
    
    // Clipboard
    fn copied(&self) -> &'static str { "KOPIERT!" }
    fn clipboard_modal_text(&self) -> &'static str { "Der Inhalt wurde in die Zwischenablage kopiert. Beim Schließen dieses Fensters wird die Zwischenablage aus Sicherheitsgründen geleert." }
    fn btn_clear_clipboard(&self) -> &'static str { "🗑️ Zwischenablage leeren" }

    // Tab labels
    fn tab_cyf(&self) -> &'static str { "CYF" }
    fn tab_networking(&self) -> &'static str { "Netzwerk" }
    fn tab_info(&self) -> &'static str { "Info" }
    fn tab_settings(&self) -> &'static str { "Einstellungen" }

    // Start gate modal
    fn gate_title(&self) -> &'static str { "Willkommen bei Zsozso" }
    fn btn_next(&self) -> &'static str { "Weiter" }

    // Passkey authentication
    fn authenticating(&self) -> &'static str { "Authentifizierung..." }
    fn auth_failed(&self) -> &'static str { "Authentifizierung fehlgeschlagen" }
    fn btn_exit(&self) -> &'static str { "Beenden" }
    fn no_prf_key(&self) -> &'static str { "Kein Passkey-Verschlüsselungsschlüssel verfügbar. Bitte zuerst erneut authentifizieren." }

    // Info tab
    fn info_public_key_label(&self) -> &'static str { "Ihr öffentlicher Schlüssel" }
    fn info_no_key(&self) -> &'static str { "Kein Schlüssel geladen. Generieren oder importieren Sie einen unter Einstellungen." }

    // Networking tab / Smart Contract
    fn btn_ping(&self) -> &'static str { "\u{1F3D3} Ping" }
    fn ping_calling(&self) -> &'static str { "\u{1F4E1} Vertrag wird aufgerufen..." }
    fn ping_success(&self, msg: &str) -> String { format!("\u{2705} {}", msg) }
    fn ping_error(&self, e: &str) -> String { format!("\u{274C} {}", e) }
    fn ping_no_key(&self) -> &'static str { "\u{26A0}\u{FE0F} Laden Sie zuerst einen Schlüssel (Tab Einstellungen)." }

    // QR Scanner
    fn btn_scan_qr(&self) -> &'static str { "\u{1F4F7} QR scannen" }
    fn scan_scanning(&self) -> &'static str { "\u{1F4F7} Wird gescannt..." }
    fn scan_success(&self, key: &str) -> String { format!("\u{2705} Gescannt: {}", key) }
    fn scan_error(&self, e: &str) -> String { format!("\u{274C} Scan fehlgeschlagen: {}", e) }

    // Log tab
    fn tab_log(&self) -> &'static str { "Protokoll" }
    fn log_refresh(&self) -> &'static str { "\u{1F504} Aktualisieren" }
    fn log_clear(&self) -> &'static str { "\u{1F5D1} Löschen" }
    fn log_save(&self) -> &'static str { "\u{1F4BE} Speichern" }
    fn log_saving(&self) -> &'static str { "Wird gespeichert..." }
    fn log_save_ok(&self) -> &'static str { "\u{2705} Protokoll gespeichert" }
    fn log_save_fail(&self, e: &str) -> String { format!("\u{274C} Speichern fehlgeschlagen: {}", e) }
    fn log_save_empty(&self) -> &'static str { "\u{26A0}\u{FE0F} Protokoll ist leer" }

    // GUN DB dump
    fn btn_dump_gun_db(&self) -> &'static str { "\u{1F4E6} GUN DB exportieren" }
    fn log_dumping(&self) -> &'static str { "GUN DB wird exportiert..." }
    fn log_dump_ok(&self) -> &'static str { "\u{2705} GUN DB exportiert" }

    // Update toast
    fn toast_update_available(&self) -> &'static str { "\u{1F680} Eine neue Version von Zsozso ist verfügbar!" }
    fn btn_update_now(&self) -> &'static str { "Jetzt aktualisieren" }

    // Info tab – version
    fn info_version(&self, ver: &str) -> String { format!("Version: {}", ver) }

    // Network switch modal
    fn network_switch_save_prompt(&self) -> &'static str { "Sie haben einen geheimen Schl\u{00FC}ssel geladen. M\u{00F6}chten Sie ihn vor dem Netzwerkwechsel speichern?" }
    fn btn_save_and_switch(&self) -> &'static str { "\u{1F4BE} Speichern und wechseln" }
    fn btn_switch_and_save(&self) -> &'static str { "\u{1F504} Wechseln und speichern" }
    fn btn_switch_without_saving(&self) -> &'static str { "Wechseln ohne zu speichern" }
    fn btn_cancel(&self) -> &'static str { "Abbrechen" }

    // SEA key generation modal
    fn btn_generate_db_secret(&self) -> &'static str { "\u{1F511} DB-Geheimnis generieren" }
    fn sea_modal_title(&self) -> &'static str { "GunDB SEA-Schlüssel generieren" }
    fn sea_modal_placeholder(&self) -> &'static str { "Geheime Passphrase eingeben..." }
    fn btn_generate_db_keys(&self) -> &'static str { "\u{1F511} DB-Schlüssel generieren" }
    fn sea_generating(&self) -> &'static str { "\u{1F504} Schlüssel werden generiert..." }
    fn sea_keys_generated(&self) -> &'static str { "\u{2705} SEA-Schlüssel generiert und im Speicher geladen." }
    fn sea_generation_error(&self, e: &str) -> String { format!("\u{274C} Schlüsselgenerierung fehlgeschlagen: {}", e) }
    fn btn_close(&self) -> &'static str { "Schließen" }

    // Biometric identification toggle
    fn lbl_biometric(&self) -> &'static str { "Biometrische Identifikation" }
    fn lbl_biometric_desc(&self) -> &'static str { "Biometrische Authentifizierung zum Schutz Ihrer Wallet verwenden" }
    fn biometric_required_to_save(&self) -> &'static str { "Bitte aktivieren Sie die Biometrische Identifikation in den Einstellungen, bevor Sie Ihr Geheimnis speichern." }

    // Nickname (Settings)
    fn lbl_nickname(&self) -> &'static str { "Spitzname" }
    fn lbl_nickname_ph(&self) -> &'static str { "Spitzname eingeben..." }
    fn btn_save_nickname(&self) -> &'static str { "\u{1F4BE} Speichern" }
    fn nickname_saved(&self) -> &'static str { "\u{2705} Spitzname gespeichert!" }
    fn nickname_save_error(&self, e: &str) -> String { format!("\u{274C} Spitzname konnte nicht gespeichert werden: {}", e) }

    // Network tab – hierarchy
    fn lbl_parents(&self) -> &'static str { "Eltern" }
    fn lbl_workers(&self) -> &'static str { "Arbeiter" }
    fn btn_new_worker(&self) -> &'static str { "\u{2795} Neu" }
    fn network_no_key(&self) -> &'static str { "\u{26A0}\u{FE0F} Laden Sie zuerst einen Schlüssel (Tab Einstellungen)." }
    fn network_add_worker_success(&self, key: &str) -> String { format!("\u{2705} Arbeiter hinzugefügt: {}", key) }
    fn network_add_worker_error(&self, e: &str) -> String { format!("\u{274C} Arbeiter konnte nicht hinzugefügt werden: {}", e) }

    // CYF tab
    fn btn_burn(&self) -> &'static str { "\u{1F525} Verbrennen" }
    fn btn_mint(&self) -> &'static str { "\u{1FA99} Prägen" }
    fn btn_ok(&self) -> &'static str { "OK" }
    fn cyf_not_implemented(&self, fn_name: &str) -> String { format!("Die Funktion {} ist noch nicht implementiert.", fn_name) }

    // GUN node address
    fn lbl_gun_address(&self) -> &'static str { "GUN-Knotenadresse" }
    fn lbl_gun_relay_url(&self) -> &'static str { "GUN Relay-URL" }
    fn lbl_gun_relay_ph(&self) -> &'static str { "https://your-server.com/gun" }
    fn btn_save_gun_relay(&self) -> &'static str { "Speichern" }

    // SSS (Shamir's Secret Sharing)
    fn sss_modal_title(&self) -> &'static str { "\u{1F512} Wiederherstellungsanteile" }
    fn sss_modal_description(&self) -> &'static str { "Verteile diese Anteile an deine vertrauenswürdigen Knoten. 3 von 7 genügen zur Wiederherstellung deines Geheimnisses." }
    fn sss_share_label(&self, n: usize) -> String { format!("Anteil {}", n) }
    fn btn_copy_share(&self) -> &'static str { "\u{1F4CB} Kopieren" }
    fn sss_share_copied(&self) -> &'static str { "\u{2705} Kopiert!" }

    // ZS (ZSOZSO) tab
    fn tab_zsozso(&self) -> &'static str { "ZS" }
    fn lbl_xlm_balance(&self) -> &'static str { "XLM" }
    fn lbl_zsozso_balance(&self) -> &'static str { "ZSOZSO" }
    fn lbl_locked_zsozso(&self) -> &'static str { "Gesperrtes ZSOZSO" }
    fn btn_lock(&self) -> &'static str { "\u{1F512} Sperren" }
    fn btn_unlock(&self) -> &'static str { "\u{1F513} Entsperren" }
    fn btn_refresh_balances(&self) -> &'static str { "\u{1F504} Aktualisieren" }
    fn zs_fetching_balances(&self) -> &'static str { "\u{1F504} Salden werden abgerufen..." }
    fn zs_mainnet_only(&self) -> &'static str { "ZSOZSO ist nur auf Mainnet verfügbar. Wechseln Sie zu Mainnet in den Einstellungen." }
    fn zs_no_key(&self) -> &'static str { "Laden Sie zuerst einen Schlüssel in den Einstellungen." }
    fn zs_locking(&self) -> &'static str { "\u{1F512} ZSOZSO wird gesperrt..." }
    fn zs_unlocking(&self) -> &'static str { "\u{1F513} ZSOZSO wird entsperrt..." }
    fn fmt_zs_lock_success(&self, amount: &str) -> String { format!("\u{2705} {} ZSOZSO gesperrt", amount) }
    fn fmt_zs_unlock_success(&self, amount: &str) -> String { format!("\u{2705} {} ZSOZSO entsperrt", amount) }
    fn fmt_zs_lock_error(&self, err: &str) -> String { format!("\u{274C} Sperrung fehlgeschlagen: {}", err) }
    fn fmt_zs_unlock_error(&self, err: &str) -> String { format!("\u{274C} Entsperrung fehlgeschlagen: {}", err) }
    fn zs_invalid_amount(&self) -> &'static str { "Bitte geben Sie einen gültigen Betrag ein." }
    fn lbl_amount(&self) -> &'static str { "Betrag" }
    fn lbl_mainnet_account(&self) -> &'static str { "Mainnet-Konto" }
    fn lbl_testnet_account(&self) -> &'static str { "Testnet-Konto" }
    fn lbl_no_account(&self) -> &'static str { "Kein Konto" }

    fn lbl_mainnet_keys(&self) -> &'static str { "Mainnet-Schlüssel" }
    fn lbl_testnet_keys(&self) -> &'static str { "Testnet-Schlüssel" }
    fn lbl_pin_code(&self) -> &'static str { "PIN-Code" }
    fn lbl_pin_code_desc(&self) -> &'static str { "Legen Sie einen PIN-Code fest, um Ihre Wallet zu schützen (nur localhost)" }
    fn lbl_pin_code_ph(&self) -> &'static str { "PIN eingeben..." }
    fn btn_set_pin(&self) -> &'static str { "PIN festlegen" }
}
