use log::{info, warn, error};
use std::time::Instant;

pub struct RiskManager {
    pub max_concurrent_positions: usize,
    pub trade_size_sol: f64,
    pub global_pnl_sol: f64,
    pub max_drawdown_sol: f64, // PnL négatif max toléré avant arrêt total (ex: 0.5 SOL)
    pub is_halted: bool,
    pub start_time: Instant,
}

impl RiskManager {
    pub fn new(max_pos: usize, size_sol: f64, max_dd_sol: f64) -> Self {
        Self {
            max_concurrent_positions: max_pos,
            trade_size_sol: size_sol,
            global_pnl_sol: 0.0,
            max_drawdown_sol: max_dd_sol,
            is_halted: false,
            start_time: Instant::now(),
        }
    }

    pub fn register_trade_result(&mut self, pnl_sol: f64) {
        if self.is_halted { return; }
        
        self.global_pnl_sol += pnl_sol;
        info!("💰 Global PnL mis à jour : {:.4} SOL", self.global_pnl_sol);

        // Si le PnL global chute en dessous du seuil critique
        if self.global_pnl_sol <= -self.max_drawdown_sol {
            error!("🛑 CIRCUIT BREAKER DÉCLENCHÉ ! Perte globale maximale atteinte ({:.4} SOL).", self.global_pnl_sol);
            error!("🛑 Désactivation complète des nouvelles entrées (Kill-Switch).");
            self.is_halted = true;
        }
    }

    pub fn can_open_position(&self, current_positions_count: usize) -> bool {
        if self.is_halted {
            warn!("⚠️ Entrée bloquée : Circuit Breaker actif.");
            return false;
        }
        if current_positions_count >= self.max_concurrent_positions {
            warn!("⚠️ Entrée bloquée : Capacité max atteinte ({} positions).", self.max_concurrent_positions);
            return false;
        }
        true
    }
}
