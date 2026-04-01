use super::UiI18n;

pub struct EnglishUi;

impl UiI18n for EnglishUi {
    // Initial signal values
    fn no_key_loaded(&self) -> &'static str { "No key loaded" }
    fn copy_label(&self) -> &'static str { "Copy" }
    fn copy_xdr_label(&self) -> &'static str { "Copy XDR" }
    fn waiting(&self) -> &'static str { "Waiting..." }
    
    // Status messages
    fn err_no_generated_xdr(&self) -> &'static str { "Error: No generated XDR!" }
    fn submitting(&self) -> &'static str { "Submitting..." }
    fn calling_faucet(&self) -> &'static str { "🚀 Calling faucet..." }
    fn no_loaded_key(&self) -> &'static str { "⚠️ No loaded key!" }
    fn fetching_sequence(&self) -> &'static str { "🔍 Fetching sequence number..." }
    
    // Console/println messages
    fn clipboard_cleared(&self) -> &'static str { "🔐 Clipboard cleared for security." }
    fn save_success(&self) -> &'static str { "✅ Successfully saved to system wallet!" }
    fn nothing_to_save(&self) -> &'static str { "⚠️ Nothing to save (key is empty)!" }
    fn loading_started(&self) -> &'static str { "🔍 Loading started..." }
    fn key_loaded_len(&self, len: usize) -> String { format!("📥 Key loaded, length: {}", len) }
    fn ui_updated_with_key(&self) -> &'static str { "✨ UI successfully updated with loaded key." }
    
    // Format helpers
    fn fmt_success(&self, msg: &str) -> String { format!("✅ SUCCESS! {}", msg) }
    fn fmt_error(&self, e: &str) -> String { format!("❌ {}", e) }
    fn fmt_xdr_ready(&self, net: &str, seq: i64) -> String { format!("✅ XDR Ready! [{}] (Seq: {})", net, seq) }
    
    // Button labels
    fn btn_new_key(&self) -> &'static str { "✨ New Key" }
    fn btn_import(&self) -> &'static str { "📥 Import" }
    fn btn_hide_secret(&self) -> &'static str { "🙈 Hide" }
    fn btn_reveal_secret(&self) -> &'static str { "👁 Reveal" }
    fn btn_activate_faucet(&self) -> &'static str { "🚀 Activate (Faucet)" }
    fn btn_save_to_os(&self) -> &'static str { "💾 Save to OS Wallet" }
    fn btn_load(&self) -> &'static str { "🔓 Load" }
    fn btn_generate_xdr(&self) -> &'static str { "🛠 Generate Transaction XDR" }
    fn btn_submit_tx(&self) -> &'static str { "🚀 SUBMIT Transaction" }
    
    // UI labels
    fn lbl_active_address(&self) -> &'static str { "Active Address (Public Key):" }
    fn lbl_signed_xdr(&self) -> &'static str { "SIGNED XDR:" }
    fn lbl_import_ph(&self) -> &'static str { "Import (S...)" }
    
    // Network labels
    fn net_testnet_label(&self) -> &'static str { "🧪 Testnet ⚠️" }
    fn net_mainnet_label(&self) -> &'static str { "Mainnet" }
    
    // Clipboard
    fn copied(&self) -> &'static str { "COPIED!" }
    fn clipboard_modal_text(&self) -> &'static str { "The content has been copied to the clipboard. When you close this dialog, the clipboard will be cleared for security." }
    fn btn_clear_clipboard(&self) -> &'static str { "🗑️ Clear clipboard content" }

    // Tab labels
    fn tab_cyf(&self) -> &'static str { "CYF" }
    fn tab_networking(&self) -> &'static str { "Network" }
    fn tab_info(&self) -> &'static str { "Info" }
    fn tab_settings(&self) -> &'static str { "Settings" }

    // Start gate modal
    fn gate_title(&self) -> &'static str { "Welcome to Zsozso" }
    fn btn_next(&self) -> &'static str { "Next" }

    // Passkey authentication
    fn authenticating(&self) -> &'static str { "Authenticating..." }
    fn auth_failed(&self) -> &'static str { "Authentication failed" }
    fn btn_exit(&self) -> &'static str { "Exit now" }
    fn no_prf_key(&self) -> &'static str { "No passkey encryption key available. Re-authenticate first." }

    // Info tab
    fn info_public_key_label(&self) -> &'static str { "Your Public Key" }
    fn info_no_key(&self) -> &'static str { "No key loaded. Generate or import one in Settings." }

    // Networking tab / Smart Contract
    fn btn_ping(&self) -> &'static str { "\u{1F3D3} Ping" }
    fn ping_calling(&self) -> &'static str { "\u{1F4E1} Calling contract..." }
    fn ping_success(&self, msg: &str) -> String { format!("\u{2705} {}", msg) }
    fn ping_error(&self, e: &str) -> String { format!("\u{274C} {}", e) }
    fn ping_no_key(&self) -> &'static str { "\u{26A0}\u{FE0F} Load a key first (Settings tab)." }

    // QR Scanner
    fn btn_scan_qr(&self) -> &'static str { "\u{1F4F7} Scan QR" }
    fn scan_scanning(&self) -> &'static str { "\u{1F4F7} Scanning..." }
    fn scan_success(&self, key: &str) -> String { format!("\u{2705} Scanned: {}", key) }
    fn scan_error(&self, e: &str) -> String { format!("\u{274C} Scan failed: {}", e) }

    // Log tab
    fn tab_log(&self) -> &'static str { "Log" }
    fn log_refresh(&self) -> &'static str { "\u{1F504} Refresh" }
    fn log_clear(&self) -> &'static str { "\u{1F5D1} Clear" }
    fn log_save(&self) -> &'static str { "\u{1F4BE} Save" }
    fn log_saving(&self) -> &'static str { "Saving..." }
    fn log_save_ok(&self) -> &'static str { "\u{2705} Log saved" }
    fn log_save_fail(&self, e: &str) -> String { format!("\u{274C} Save failed: {}", e) }
    fn log_save_empty(&self) -> &'static str { "\u{26A0}\u{FE0F} Log is empty" }

    // GUN DB dump
    fn btn_dump_gun_db(&self) -> &'static str { "\u{1F4E6} Dump GUN DB" }
    fn log_dumping(&self) -> &'static str { "Dumping GUN DB..." }
    fn log_dump_ok(&self) -> &'static str { "\u{2705} GUN DB dump saved" }

    // Update toast
    fn toast_update_available(&self) -> &'static str { "\u{1F680} A new version of Zsozso is available!" }
    fn btn_update_now(&self) -> &'static str { "Update Now" }

    // Info tab – version
    fn info_version(&self, ver: &str) -> String { format!("Version: {}", ver) }

    // Network switch modal
    fn network_switch_save_prompt(&self) -> &'static str { "You have a secret key loaded. Would you like to save it before switching networks?" }
    fn btn_save_and_switch(&self) -> &'static str { "\u{1F4BE} Save & Switch" }
    fn btn_switch_and_save(&self) -> &'static str { "\u{1F504} Switch & Save" }
    fn btn_switch_without_saving(&self) -> &'static str { "Switch without saving" }
    fn btn_cancel(&self) -> &'static str { "Cancel" }

    // SEA key generation modal
    fn btn_generate_db_secret(&self) -> &'static str { "\u{1F511} Generate DB Secret" }
    fn sea_modal_title(&self) -> &'static str { "Generate GunDB SEA Keys" }
    fn sea_modal_placeholder(&self) -> &'static str { "Enter secret passphrase..." }
    fn btn_generate_db_keys(&self) -> &'static str { "\u{1F511} Generate DB Keys" }
    fn sea_generating(&self) -> &'static str { "\u{1F504} Generating keys..." }
    fn sea_keys_generated(&self) -> &'static str { "\u{2705} SEA keys generated and loaded into memory." }
    fn sea_generation_error(&self, e: &str) -> String { format!("\u{274C} Key generation failed: {}", e) }
    fn btn_close(&self) -> &'static str { "Close" }

    // Biometric identification toggle
    fn lbl_biometric(&self) -> &'static str { "Biometric Identification" }
    fn lbl_biometric_desc(&self) -> &'static str { "Use biometric authentication to protect your wallet" }
    fn biometric_required_to_save(&self) -> &'static str { "Please enable Biometric Identification in Settings before saving your secret." }

    // Nickname (Settings)
    fn lbl_nickname(&self) -> &'static str { "Nickname" }
    fn lbl_nickname_ph(&self) -> &'static str { "Enter your nickname..." }
    fn btn_save_nickname(&self) -> &'static str { "\u{1F4BE} Save" }
    fn nickname_saved(&self) -> &'static str { "\u{2705} Nickname saved!" }
    fn nickname_save_error(&self, e: &str) -> String { format!("\u{274C} Failed to save nickname: {}", e) }

    // Network tab – hierarchy
    fn lbl_parents(&self) -> &'static str { "Parents" }
    fn lbl_workers(&self) -> &'static str { "Workers" }
    fn btn_new_worker(&self) -> &'static str { "\u{2795} New" }
    fn network_no_key(&self) -> &'static str { "\u{26A0}\u{FE0F} Load a key first (Settings tab)." }
    fn network_add_worker_success(&self, key: &str) -> String { format!("\u{2705} Worker added: {}", key) }
    fn network_add_worker_error(&self, e: &str) -> String { format!("\u{274C} Failed to add worker: {}", e) }

    // CYF tab
    fn btn_burn(&self) -> &'static str { "\u{1F525} Burn" }
    fn btn_mint(&self) -> &'static str { "\u{1FA99} Mint" }
    fn btn_ok(&self) -> &'static str { "OK" }
    fn cyf_not_implemented(&self, fn_name: &str) -> String { format!("The {} function is not yet implemented.", fn_name) }

    // GUN node address
    fn lbl_gun_address(&self) -> &'static str { "GUN Node Address" }
    fn lbl_gun_relay_url(&self) -> &'static str { "GUN Relay URL" }
    fn lbl_gun_relay_ph(&self) -> &'static str { "https://your-server.com/gun" }
    fn btn_save_gun_relay(&self) -> &'static str { "Save" }

    // SSS (Shamir's Secret Sharing)
    fn sss_modal_title(&self) -> &'static str { "\u{1F512} Recovery Shares" }
    fn sss_modal_description(&self) -> &'static str { "Distribute these shares to your trusted nodes. Any 3 of 7 can reconstruct your brain-secret." }
    fn sss_share_label(&self, n: usize) -> String { format!("Share {}", n) }
    fn btn_copy_share(&self) -> &'static str { "\u{1F4CB} Copy" }
    fn sss_share_copied(&self) -> &'static str { "\u{2705} Copied!" }

    // ZS (ZSOZSO) tab
    fn tab_zsozso(&self) -> &'static str { "ZS" }
    fn lbl_xlm_balance(&self) -> &'static str { "XLM" }
    fn lbl_zsozso_balance(&self) -> &'static str { "ZSOZSO" }
    fn lbl_locked_zsozso(&self) -> &'static str { "Locked ZSOZSO" }
    fn btn_lock(&self) -> &'static str { "\u{1F512} Lock" }
    fn btn_unlock(&self) -> &'static str { "\u{1F513} Unlock" }
    fn btn_refresh_balances(&self) -> &'static str { "\u{1F504} Refresh" }
    fn zs_fetching_balances(&self) -> &'static str { "\u{1F504} Fetching balances..." }
    fn zs_mainnet_only(&self) -> &'static str { "ZSOZSO is only available on Mainnet. Switch to Mainnet in Settings." }
    fn zs_no_key(&self) -> &'static str { "Load a key first in Settings." }
    fn zs_locking(&self) -> &'static str { "\u{1F512} Locking ZSOZSO..." }
    fn zs_unlocking(&self) -> &'static str { "\u{1F513} Unlocking ZSOZSO..." }
    fn fmt_zs_lock_success(&self, amount: &str) -> String { format!("\u{2705} Locked {} ZSOZSO", amount) }
    fn fmt_zs_unlock_success(&self, amount: &str) -> String { format!("\u{2705} Unlocked {} ZSOZSO", amount) }
    fn fmt_zs_lock_error(&self, err: &str) -> String { format!("\u{274C} Lock failed: {}", err) }
    fn fmt_zs_unlock_error(&self, err: &str) -> String { format!("\u{274C} Unlock failed: {}", err) }
    fn zs_invalid_amount(&self) -> &'static str { "Please enter a valid amount." }
    fn lbl_amount(&self) -> &'static str { "Amount" }
    fn lbl_mainnet_account(&self) -> &'static str { "Mainnet Account" }
    fn lbl_testnet_account(&self) -> &'static str { "Testnet Account" }
    fn lbl_no_account(&self) -> &'static str { "No account" }

    fn lbl_mainnet_keys(&self) -> &'static str { "Mainnet Key" }
    fn lbl_testnet_keys(&self) -> &'static str { "Testnet Key" }
    fn lbl_pin_code(&self) -> &'static str { "PIN Code" }
    fn lbl_pin_code_desc(&self) -> &'static str { "Set a PIN code to protect your wallet (localhost only)" }
    fn lbl_pin_code_ph(&self) -> &'static str { "Enter PIN..." }
    fn btn_set_pin(&self) -> &'static str { "Set PIN" }
}
