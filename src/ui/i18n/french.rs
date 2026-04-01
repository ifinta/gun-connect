use super::UiI18n;

pub struct FrenchUi;

impl UiI18n for FrenchUi {
    // Initial signal values
    fn no_key_loaded(&self) -> &'static str { "Aucune clé chargée" }
    fn copy_label(&self) -> &'static str { "Copier" }
    fn copy_xdr_label(&self) -> &'static str { "Copier XDR" }
    fn waiting(&self) -> &'static str { "En attente..." }
    
    // Status messages
    fn err_no_generated_xdr(&self) -> &'static str { "Erreur : Aucun XDR généré !" }
    fn submitting(&self) -> &'static str { "Envoi en cours..." }
    fn calling_faucet(&self) -> &'static str { "🚀 Appel du faucet..." }
    fn no_loaded_key(&self) -> &'static str { "⚠️ Aucune clé chargée !" }
    fn fetching_sequence(&self) -> &'static str { "🔍 Récupération du numéro de séquence..." }
    
    // Console/println messages
    fn clipboard_cleared(&self) -> &'static str { "🔐 Presse-papiers vidé par mesure de sécurité." }
    fn save_success(&self) -> &'static str { "✅ Sauvegarde réussie dans le portefeuille système !" }
    fn nothing_to_save(&self) -> &'static str { "⚠️ Rien à sauvegarder (la clé est vide) !" }
    fn loading_started(&self) -> &'static str { "🔍 Chargement en cours..." }
    fn key_loaded_len(&self, len: usize) -> String { format!("📥 Clé chargée, longueur : {}", len) }
    fn ui_updated_with_key(&self) -> &'static str { "✨ Interface mise à jour avec la clé chargée." }
    
    // Format helpers
    fn fmt_success(&self, msg: &str) -> String { format!("✅ SUCCÈS ! {}", msg) }
    fn fmt_error(&self, e: &str) -> String { format!("❌ {}", e) }
    fn fmt_xdr_ready(&self, net: &str, seq: i64) -> String { format!("✅ XDR Prêt ! [{}] (Séq : {})", net, seq) }
    
    // Button labels
    fn btn_new_key(&self) -> &'static str { "✨ Nouvelle Clé" }
    fn btn_import(&self) -> &'static str { "📥 Importer" }
    fn btn_hide_secret(&self) -> &'static str { "🙈 Masquer" }
    fn btn_reveal_secret(&self) -> &'static str { "👁 Révéler" }
    fn btn_activate_faucet(&self) -> &'static str { "🚀 Activer (Faucet)" }
    fn btn_save_to_os(&self) -> &'static str { "💾 Sauvegarder dans le portefeuille OS" }
    fn btn_load(&self) -> &'static str { "🔓 Charger" }
    fn btn_generate_xdr(&self) -> &'static str { "🛠 Générer le XDR de transaction" }
    fn btn_submit_tx(&self) -> &'static str { "🚀 ENVOYER la Transaction" }
    
    // UI labels
    fn lbl_active_address(&self) -> &'static str { "Adresse Active (Clé Publique) :" }
    fn lbl_signed_xdr(&self) -> &'static str { "XDR SIGNÉ :" }
    fn lbl_import_ph(&self) -> &'static str { "Importer (S...)" }
    
    // Network labels
    fn net_testnet_label(&self) -> &'static str { "🧪 Testnet ⚠️" }
    fn net_mainnet_label(&self) -> &'static str { "Mainnet" }
    
    // Clipboard
    fn copied(&self) -> &'static str { "COPIÉ !" }
    fn clipboard_modal_text(&self) -> &'static str { "Le contenu a été copié dans le presse-papiers. Lorsque vous fermerez cette fenêtre, le presse-papiers sera vidé par mesure de sécurité." }
    fn btn_clear_clipboard(&self) -> &'static str { "🗑️ Vider le presse-papiers" }

    // Tab labels
    fn tab_cyf(&self) -> &'static str { "CYF" }
    fn tab_networking(&self) -> &'static str { "Réseau" }
    fn tab_info(&self) -> &'static str { "Info" }
    fn tab_settings(&self) -> &'static str { "Paramètres" }

    // Start gate modal
    fn gate_title(&self) -> &'static str { "Bienvenue sur Zsozso" }
    fn btn_next(&self) -> &'static str { "Suivant" }

    // Passkey authentication
    fn authenticating(&self) -> &'static str { "Authentification..." }
    fn auth_failed(&self) -> &'static str { "Échec de l'authentification" }
    fn btn_exit(&self) -> &'static str { "Quitter" }
    fn no_prf_key(&self) -> &'static str { "Aucune clé de chiffrement passkey disponible. Réauthentifiez-vous d'abord." }

    // Info tab
    fn info_public_key_label(&self) -> &'static str { "Votre Clé Publique" }
    fn info_no_key(&self) -> &'static str { "Aucune clé chargée. Générez ou importez-en une dans les Paramètres." }

    // Networking tab / Smart Contract
    fn btn_ping(&self) -> &'static str { "\u{1F3D3} Ping" }
    fn ping_calling(&self) -> &'static str { "\u{1F4E1} Appel du contrat..." }
    fn ping_success(&self, msg: &str) -> String { format!("\u{2705} {}", msg) }
    fn ping_error(&self, e: &str) -> String { format!("\u{274C} {}", e) }
    fn ping_no_key(&self) -> &'static str { "\u{26A0}\u{FE0F} Chargez d'abord une clé (onglet Paramètres)." }

    // QR Scanner
    fn btn_scan_qr(&self) -> &'static str { "\u{1F4F7} Scanner QR" }
    fn scan_scanning(&self) -> &'static str { "\u{1F4F7} Scan en cours..." }
    fn scan_success(&self, key: &str) -> String { format!("\u{2705} Scanné : {}", key) }
    fn scan_error(&self, e: &str) -> String { format!("\u{274C} Échec du scan : {}", e) }

    // Log tab
    fn tab_log(&self) -> &'static str { "Journal" }
    fn log_refresh(&self) -> &'static str { "\u{1F504} Rafraîchir" }
    fn log_clear(&self) -> &'static str { "\u{1F5D1} Effacer" }
    fn log_save(&self) -> &'static str { "\u{1F4BE} Enregistrer" }
    fn log_saving(&self) -> &'static str { "Enregistrement..." }
    fn log_save_ok(&self) -> &'static str { "\u{2705} Journal enregistré" }
    fn log_save_fail(&self, e: &str) -> String { format!("\u{274C} Échec de l'enregistrement\u{00A0}: {}", e) }
    fn log_save_empty(&self) -> &'static str { "\u{26A0}\u{FE0F} Le journal est vide" }

    // GUN DB dump
    fn btn_dump_gun_db(&self) -> &'static str { "\u{1F4E6} Export GUN DB" }
    fn log_dumping(&self) -> &'static str { "Export GUN DB en cours..." }
    fn log_dump_ok(&self) -> &'static str { "\u{2705} GUN DB exportée" }

    // Update toast
    fn toast_update_available(&self) -> &'static str { "\u{1F680} Une nouvelle version de Zsozso est disponible\u{00A0}!" }
    fn btn_update_now(&self) -> &'static str { "Mettre à jour" }

    // Info tab – version
    fn info_version(&self, ver: &str) -> String { format!("Version\u{00A0}: {}", ver) }

    // Network switch modal
    fn network_switch_save_prompt(&self) -> &'static str { "Vous avez une cl\u{00E9} secr\u{00E8}te charg\u{00E9}e. Voulez-vous la sauvegarder avant de changer de r\u{00E9}seau\u{00A0}?" }
    fn btn_save_and_switch(&self) -> &'static str { "\u{1F4BE} Sauvegarder et changer" }
    fn btn_switch_and_save(&self) -> &'static str { "\u{1F504} Changer et sauvegarder" }
    fn btn_switch_without_saving(&self) -> &'static str { "Changer sans sauvegarder" }
    fn btn_cancel(&self) -> &'static str { "Annuler" }

    // SEA key generation modal
    fn btn_generate_db_secret(&self) -> &'static str { "\u{1F511} Générer un secret DB" }
    fn sea_modal_title(&self) -> &'static str { "Générer des clés GunDB SEA" }
    fn sea_modal_placeholder(&self) -> &'static str { "Entrez la phrase secrète..." }
    fn btn_generate_db_keys(&self) -> &'static str { "\u{1F511} Générer les clés DB" }
    fn sea_generating(&self) -> &'static str { "\u{1F504} Génération des clés..." }
    fn sea_keys_generated(&self) -> &'static str { "\u{2705} Clés SEA générées et chargées en mémoire." }
    fn sea_generation_error(&self, e: &str) -> String { format!("\u{274C} Échec de la génération\u{00A0}: {}", e) }
    fn btn_close(&self) -> &'static str { "Fermer" }

    // Biometric identification toggle
    fn lbl_biometric(&self) -> &'static str { "Identification biométrique" }
    fn lbl_biometric_desc(&self) -> &'static str { "Utiliser l'authentification biométrique pour protéger votre portefeuille" }
    fn biometric_required_to_save(&self) -> &'static str { "Veuillez activer l'Identification biométrique dans les Paramètres avant de sauvegarder votre secret." }

    // Nickname (Settings)
    fn lbl_nickname(&self) -> &'static str { "Surnom" }
    fn lbl_nickname_ph(&self) -> &'static str { "Entrez votre surnom..." }
    fn btn_save_nickname(&self) -> &'static str { "\u{1F4BE} Enregistrer" }
    fn nickname_saved(&self) -> &'static str { "\u{2705} Surnom enregistré\u{00A0}!" }
    fn nickname_save_error(&self, e: &str) -> String { format!("\u{274C} Échec de l'enregistrement du surnom\u{00A0}: {}", e) }

    // Network tab – hierarchy
    fn lbl_parents(&self) -> &'static str { "Parents" }
    fn lbl_workers(&self) -> &'static str { "Travailleurs" }
    fn btn_new_worker(&self) -> &'static str { "\u{2795} Nouveau" }
    fn network_no_key(&self) -> &'static str { "\u{26A0}\u{FE0F} Chargez d'abord une clé (onglet Paramètres)." }
    fn network_add_worker_success(&self, key: &str) -> String { format!("\u{2705} Travailleur ajouté\u{00A0}: {}", key) }
    fn network_add_worker_error(&self, e: &str) -> String { format!("\u{274C} Échec de l'ajout du travailleur\u{00A0}: {}", e) }

    // CYF tab
    fn btn_burn(&self) -> &'static str { "\u{1F525} Brûler" }
    fn btn_mint(&self) -> &'static str { "\u{1FA99} Émettre" }
    fn btn_ok(&self) -> &'static str { "OK" }
    fn cyf_not_implemented(&self, fn_name: &str) -> String { format!("La fonction {} n'est pas encore implémentée.", fn_name) }

    // GUN node address
    fn lbl_gun_address(&self) -> &'static str { "Adresse du nœud GUN" }
    fn lbl_gun_relay_url(&self) -> &'static str { "URL du relais GUN" }
    fn lbl_gun_relay_ph(&self) -> &'static str { "https://your-server.com/gun" }
    fn btn_save_gun_relay(&self) -> &'static str { "Sauvegarder" }

    // SSS (Shamir's Secret Sharing)
    fn sss_modal_title(&self) -> &'static str { "\u{1F512} Parts de récupération" }
    fn sss_modal_description(&self) -> &'static str { "Distribuez ces parts à vos nœuds de confiance. 3 parts sur 7 suffisent pour reconstituer votre secret." }
    fn sss_share_label(&self, n: usize) -> String { format!("Part {}", n) }
    fn btn_copy_share(&self) -> &'static str { "\u{1F4CB} Copier" }
    fn sss_share_copied(&self) -> &'static str { "\u{2705} Copié !" }

    // ZS (ZSOZSO) tab
    fn tab_zsozso(&self) -> &'static str { "ZS" }
    fn lbl_xlm_balance(&self) -> &'static str { "XLM" }
    fn lbl_zsozso_balance(&self) -> &'static str { "ZSOZSO" }
    fn lbl_locked_zsozso(&self) -> &'static str { "ZSOZSO verrouillé" }
    fn btn_lock(&self) -> &'static str { "\u{1F512} Verrouiller" }
    fn btn_unlock(&self) -> &'static str { "\u{1F513} Déverrouiller" }
    fn btn_refresh_balances(&self) -> &'static str { "\u{1F504} Rafraîchir" }
    fn zs_fetching_balances(&self) -> &'static str { "\u{1F504} Récupération des soldes..." }
    fn zs_mainnet_only(&self) -> &'static str { "ZSOZSO n'est disponible que sur Mainnet. Basculez vers Mainnet dans les Paramètres." }
    fn zs_no_key(&self) -> &'static str { "Chargez d'abord une clé dans les Paramètres." }
    fn zs_locking(&self) -> &'static str { "\u{1F512} Verrouillage de ZSOZSO..." }
    fn zs_unlocking(&self) -> &'static str { "\u{1F513} Déverrouillage de ZSOZSO..." }
    fn fmt_zs_lock_success(&self, amount: &str) -> String { format!("\u{2705} {} ZSOZSO verrouillé", amount) }
    fn fmt_zs_unlock_success(&self, amount: &str) -> String { format!("\u{2705} {} ZSOZSO déverrouillé", amount) }
    fn fmt_zs_lock_error(&self, err: &str) -> String { format!("\u{274C} Échec du verrouillage : {}", err) }
    fn fmt_zs_unlock_error(&self, err: &str) -> String { format!("\u{274C} Échec du déverrouillage : {}", err) }
    fn zs_invalid_amount(&self) -> &'static str { "Veuillez entrer un montant valide." }
    fn lbl_amount(&self) -> &'static str { "Montant" }
    fn lbl_mainnet_account(&self) -> &'static str { "Compte Mainnet" }
    fn lbl_testnet_account(&self) -> &'static str { "Compte Testnet" }
    fn lbl_no_account(&self) -> &'static str { "Aucun compte" }

    fn lbl_mainnet_keys(&self) -> &'static str { "Clé Mainnet" }
    fn lbl_testnet_keys(&self) -> &'static str { "Clé Testnet" }
    fn lbl_pin_code(&self) -> &'static str { "Code PIN" }
    fn lbl_pin_code_desc(&self) -> &'static str { "Définissez un code PIN pour protéger votre portefeuille (localhost uniquement)" }
    fn lbl_pin_code_ph(&self) -> &'static str { "Entrez le PIN..." }
    fn btn_set_pin(&self) -> &'static str { "Définir le PIN" }
}
