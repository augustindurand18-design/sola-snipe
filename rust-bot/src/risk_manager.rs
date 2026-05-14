use log::{info, warn, error};
use std::time::Instant;

pub struct RiskManager {
    pub max_concurrent_positions: usize,
    pub trade_size_sol: f64,
    pub global_pnl_sol: f64,
    pub max_drawdown_sol: f64,
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
        let uptime = self.start_time.elapsed().as_secs();
        info!("💰 PnL: {:.4} SOL | Uptime: {}s", self.global_pnl_sol, uptime);

        if self.global_pnl_sol <= -self.max_drawdown_sol {
            error!("🛑 CIRCUIT BREAKER ! Perte max atteinte ({:.4} SOL).", self.global_pnl_sol);
            self.is_halted = true;
        }
    }

    pub fn can_open_position(&self, current_positions_count: usize) -> bool {
        if self.is_halted {
            warn!("⚠️ Entrée bloquée : Circuit Breaker actif (Uptime: {}s).", self.start_time.elapsed().as_secs());
            return false;
        }
        if current_positions_count >= self.max_concurrent_positions {
            warn!("⚠️ Entrée bloquée : Max {} positions.", self.max_concurrent_positions);
            return false;
        }
        true
    }
}
