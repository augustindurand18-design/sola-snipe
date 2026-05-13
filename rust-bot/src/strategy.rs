use std::collections::{HashSet, HashMap};
use solana_sdk::pubkey::Pubkey;
use log::{info, warn, error};
use crate::risk_manager::RiskManager;

pub struct Position {
    pub entry_price_sol: f64,
    pub token_amount: u64,
    pub sol_invested: f64,
    pub pre_forged_sell_tx: Vec<u8>, // Cache de la transaction prête à être signée
}

pub struct RugRidingStrategy {
    pub positions: HashMap<Pubkey, Position>,
    pub watchlist: HashSet<Pubkey>, 
    pub take_profit_pct: f64,
    pub stop_loss_pct: f64,
    pub risk_manager: RiskManager,
}

impl RugRidingStrategy {
    pub fn new(risk_manager: RiskManager) -> Self {
        Self {
            positions: HashMap::new(),
            watchlist: HashSet::new(),
            take_profit_pct: 35.0, 
            stop_loss_pct: 20.0,   
            risk_manager,
        }
    }

    pub fn on_creation_detected(&mut self, mint: Pubkey, dev: Pubkey, initial_dev_buy_sol: f64, cluster: Vec<Pubkey>) {
        if self.positions.contains_key(&mint) {
            return; 
        }

        if !self.risk_manager.can_open_position(self.positions.len()) {
            return;
        }

        if initial_dev_buy_sol >= 1.0 {
            info!("🚨 [RUG-RIDE OPPORTUNITY] Dev: {} a injecté {} SOL sur {}", dev, initial_dev_buy_sol, mint);
            
            self.watchlist.insert(dev);
            for buyer in cluster {
                self.watchlist.insert(buyer);
            }
            info!("👁️ Watchlist mise à jour : {} adresses traquées (O(1)).", self.watchlist.len());
            
            let mock_entry_price = 0.000000035; 
            
            // 2. PRÉ-FORGEAGE (Lazy Signing)
            // On prépare la transaction de vente (100%) immédiatement en mémoire.
            // Au moment critique, il ne restera plus qu'à rajouter le blockhash et signer.
            let pre_forged_sell_tx = vec![0; 256]; // Simulation d'une VersionedTransaction
            
            self.positions.insert(mint, Position {
                entry_price_sol: mock_entry_price,
                token_amount: 1_000_000, 
                sol_invested: self.risk_manager.trade_size_sol,
                pre_forged_sell_tx,
            });
            info!("✅ Position ouverte sur {} pour {} SOL (TX de Sortie Pré-forgée en Cache)", mint, self.risk_manager.trade_size_sol);
        }
    }

    pub fn on_dev_sell_detected(&mut self, seller: Pubkey) {
        if self.watchlist.contains(&seller) {
            warn!("🧨 [FRONT-RUN DÉFENSIF] MEMBRE DE LA WATCHLIST VEND !");
            warn!("⚡ DÉCLENCHEMENT IMMÉDIAT DU JITO BUNDLE (LEAPFROG) SUR TOUTES LES POSITIONS COMPROMISES");
            
            // 3. Tip Jito Dynamique
            // En production, nous interrogerions un oracle de mempool pour connaître le "Tip Base Fee"
            // Ici on simule une dynamique défensive : Tip de base + Prime d'urgence
            let base_network_tip = 0.005; // Extrait d'un tracker réseau
            let panic_multiplier = 4.0; // Prime pour garantir le passage en tête absolue
            let dynamic_jito_tip_sol = base_network_tip * panic_multiplier;
            
            let compute_unit_price = 3_000_000; // Priority fee extrême local
            info!("🛠️ TX : Sell 100% | Tip Dynamique: {:.4} SOL | Compute: {}", dynamic_jito_tip_sol, compute_unit_price);
            
            // Liquidation d'urgence
            // Simulation d'un PnL neutre ou légère perte sur sortie défensive
            self.risk_manager.register_trade_result(-0.01); 
            self.positions.clear();
            self.watchlist.clear();
        }
    }

    pub fn on_price_update(&mut self, mint: &Pubkey, current_price_sol: f64) {
        let mut to_remove = false;
        let mut pnl_sol_realized = 0.0;

        if let Some(pos) = self.positions.get(mint) {
            let pnl_pct = ((current_price_sol - pos.entry_price_sol) / pos.entry_price_sol) * 100.0;
            pnl_sol_realized = pos.sol_invested * (pnl_pct / 100.0);

            if pnl_pct >= self.take_profit_pct {
                info!("🎯 TAKE-PROFIT HFT (+{:.2}%) sur {}! Profit: +{:.4} SOL", pnl_pct, mint, pnl_sol_realized);
                to_remove = true;
            } else if pnl_pct <= -self.stop_loss_pct {
                error!("🩸 STOP-LOSS (-{:.2}%) sur {}! Perte: {:.4} SOL", pnl_pct.abs(), mint, pnl_sol_realized);
                to_remove = true;
            }
        }

        if to_remove {
            self.positions.remove(mint);
            self.risk_manager.register_trade_result(pnl_sol_realized);
            // On pourrait affiner en vidant la watchlist uniquement pour ce jeton
        }
    }
}
