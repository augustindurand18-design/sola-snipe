use std::collections::{HashSet, HashMap};
use solana_sdk::pubkey::Pubkey;
use log::{info, warn};
use std::sync::Arc;
use crate::risk_manager::RiskManager;
use crate::telegram::TelegramAlerts;

pub struct Position {
    pub entry_price_sol: f64,
    pub token_amount: u64,
    pub sol_invested: f64,
    pub pre_forged_sell_tx: Vec<u8>,
}

pub struct RugRidingStrategy {
    pub positions: HashMap<Pubkey, Position>,
    pub watchlist: HashSet<Pubkey>, 
    pub take_profit_pct: f64,
    pub stop_loss_pct: f64,
    pub risk_manager: RiskManager,
    pub telegram: Arc<TelegramAlerts>,
}

impl RugRidingStrategy {
    pub fn new(risk_manager: RiskManager, telegram: Arc<TelegramAlerts>) -> Self {
        Self {
            positions: HashMap::new(),
            watchlist: HashSet::new(),
            take_profit_pct: 35.0, 
            stop_loss_pct: 20.0,   
            risk_manager,
            telegram,
        }
    }

    pub async fn on_creation_detected(&mut self, mint: Pubkey, dev: Pubkey, initial_dev_buy_sol: f64, _cluster: Vec<Pubkey>) {
        if self.positions.contains_key(&mint) || !self.risk_manager.can_open_position(self.positions.len()) {
            return;
        }

        info!("🚨 [RUG-RIDE OPPORTUNITY] Dev: {} a injecté {} SOL sur {}", dev, initial_dev_buy_sol, mint);
        
        self.watchlist.insert(dev);
        // Simulation d'entrée
        let mock_entry_price = 0.000000035; 
        let pre_forged_sell_tx = vec![0; 256]; 
        
        self.positions.insert(mint, Position {
            entry_price_sol: mock_entry_price,
            token_amount: 1_000_000, 
            sol_invested: self.risk_manager.trade_size_sol,
            pre_forged_sell_tx,
        });

        // Alerte Telegram
        self.telegram.alert_entry(&mint.to_string(), self.risk_manager.trade_size_sol).await;
    }

    pub async fn on_dev_sell_detected(&mut self, seller: Pubkey) {
        if self.watchlist.contains(&seller) {
            warn!("🧨 [FRONT-RUN DÉFENSIF] MEMBRE DE LA WATCHLIST VEND !");
            
            // Alerte Telegram avant liquidation
            for (mint, _) in &self.positions {
                self.telegram.alert_leapfrog(&mint.to_string(), &seller.to_string()).await;
            }

            self.risk_manager.register_trade_result(-0.01); 
            self.positions.clear();
            self.watchlist.clear();
        }
    }

    pub async fn on_price_update(&mut self, mint: &Pubkey, current_price_sol: f64) {
        let mut to_remove = false;
        let mut pnl_sol_realized = 0.0;
        let mut pnl_pct = 0.0;

        if let Some(pos) = self.positions.get(mint) {
            pnl_pct = ((current_price_sol - pos.entry_price_sol) / pos.entry_price_sol) * 100.0;
            pnl_sol_realized = pos.sol_invested * (pnl_pct / 100.0);

            if pnl_pct >= self.take_profit_pct {
                self.telegram.alert_take_profit(&mint.to_string(), pnl_pct, pnl_sol_realized).await;
                to_remove = true;
            } else if pnl_pct <= -self.stop_loss_pct {
                self.telegram.alert_stop_loss(&mint.to_string(), pnl_pct, pnl_sol_realized).await;
                to_remove = true;
            }
        }

        if to_remove {
            self.positions.remove(mint);
            self.risk_manager.register_trade_result(pnl_sol_realized);
        }
    }
}
