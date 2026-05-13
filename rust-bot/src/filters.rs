use log::{info, warn};

pub struct TokenFilter;

impl TokenFilter {
    /// Évalue si un lancement présente les caractéristiques parfaites d'un "Bon Rug Pull"
    /// (Un jeton artificiellement manipulé mais suffisamment bien fait pour attirer la masse).
    pub fn is_optimal_rug_ride(
        dev_buy_sol: f64,
        cluster_wallets_count: usize,
        has_socials: bool,
        name: &str,
    ) -> bool {
        // 1. LE "SWEET SPOT" DU CAPITAL (Goldilocks Zone)
        // - < 1.0 SOL : Trop faible. Ne créera pas de bougie verte assez grosse pour déclencher le FOMO des autres bots.
        // - > 7.0 SOL : Danger de mort. Le dev possède 70%+ de la supply. S'il vend, la courbe s'effondre à zéro. Il peut "sniper" son propre jeton et dumper à la seconde 2.
        if dev_buy_sol < 1.5 || dev_buy_sol > 6.0 {
            warn!("Filtre Rejeté (Capital) : Dev Buy = {} SOL", dev_buy_sol);
            return false;
        }

        // 2. LE FILTRE DE MANIPULATION (Sybil Attack / Volume Spoofing)
        // Les pros ne lancent jamais avec un seul wallet. Ils achètent avec 5 à 15 sous-wallets dans le même bloc
        // pour faire croire à la blockchain qu'il y a de l'engouement organique. C'est CE signal qu'on veut.
        if cluster_wallets_count < 4 {
            warn!("Filtre Rejeté (Cluster) : Seulement {} acheteurs dans le bloc T0.", cluster_wallets_count);
            return false;
        }

        // 3. L'ILLUSION DE LÉGITIMITÉ (Socials)
        // Un scam *sans* Twitter/Telegram va dumper en 30 secondes chrono (Trop risqué même en HFT).
        // Un scam *avec* des liens sociaux va essayer de survivre 3 à 10 minutes pour attirer les humains.
        // C'est notre fenêtre de tir pour prendre les +30% et partir.
        if !has_socials {
            warn!("Filtre Rejeté (Socials) : Aucun lien social détecté (Scam low-effort).");
            return false;
        }

        // 4. LE FILTRE MÉTIER (Noms génériques)
        // Éviter les jetons générés par défaut par des scripts bâclés
        if name.to_lowercase().contains("test") || name.len() < 3 {
            return false;
        }

        info!("🟢 [FILTRE VALIDÉ] Ce jeton est un candidat parfait pour le Rug-Riding !");
        true
    }

    /// Filtre de Sécurité Absolue (Hardware-level)
    /// Vérifie le buffer binaire de la Mint pour s'assurer que le dev ne peut ni
    /// "Freeze" notre portefeuille, ni imprimer (Mint) une infinité de nouveaux jetons.
    pub fn is_mint_safe(mint_account_data: &[u8]) -> bool {
        // En SPL Token, le buffer d'une Mint fait exactement 82 bytes.
        // Optionnellement, on pourrait utiliser `spl_token::state::Mint::unpack`,
        // mais pour du Zéro-Latence HFT, on lit directement les octets (Zero-Copy).
        
        if mint_account_data.len() != 82 {
            warn!("🛡️ [DANGER] Format de Mint suspect (Non-Standard).");
            return false;
        }

        // Offset 0 (4 bytes) : Tag (Option) pour la Mint Authority
        let has_mint_authority = mint_account_data[0] != 0;
        
        // Offset 46 (4 bytes) : Tag (Option) pour la Freeze Authority
        let has_freeze_authority = mint_account_data[46] != 0;

        if has_mint_authority {
            warn!("🛡️ [DANGER DE MORT] Le développeur a conservé la Mint Authority. Il peut imprimer des jetons à l'infini et dumper le prix à zéro.");
            return false;
        }

        if has_freeze_authority {
            warn!("🛡️ [DANGER DE MORT] Le développeur a conservé la Freeze Authority. Il peut bloquer notre vente pendant qu'il rug-pull.");
            return false;
        }

        info!("🛡️ [SÉCURITÉ VÉRIFIÉE] Le contrat du jeton est immuable. Risque de blocage = 0%.");
        true
    }

    /// Filtre On-Chain : Maturité du Wallet
    /// Interroge le RPC (ou l'indexeur) pour vérifier l'âge du portefeuille du développeur.
    pub fn is_wallet_mature(first_signature_timestamp: u64) -> bool {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let age_in_seconds = current_time.saturating_sub(first_signature_timestamp);
        
        // Un wallet créé il y a moins de 2 heures est statistiquement un "Burner Wallet" de scammer
        if age_in_seconds < 7200 {
            warn!("⚠️ [FILTRE MATURITÉ] Rejeté : Le wallet du Dev est trop récent (créé il y a {} secondes). Scammer probable.", age_in_seconds);
            return false;
        }
        true
    }

    /// Filtre On-Chain : Clustering Tri-angulaire
    /// Identifie si le wallet initial a été financé par le même "Parent Wallet" que de précédents Rug Pulls.
    pub fn check_sybil_funding_source(funding_wallet: &str, known_scam_hubs: &[&str]) -> bool {
        if known_scam_hubs.contains(&funding_wallet) {
            warn!("⚠️ [FILTRE CLUSTERING] Rejeté : Le funding vient d'un hub de scams connu ({})", funding_wallet);
            return false;
        }
        true
    }
}
