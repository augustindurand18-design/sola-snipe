use log::info;
use std::sync::atomic::{AtomicU64, Ordering};

static TOTAL_TRADES: AtomicU64 = AtomicU64::new(0);
static TOTAL_WINS: AtomicU64 = AtomicU64::new(0);

pub struct MetricsServer {
    pub port: u16,
}

impl MetricsServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn start_background_thread(&self) {
        let port = self.port;
        tokio::spawn(async move {
            info!("📊 [METRICS] Serveur démarré sur http://localhost:{}/metrics", port);
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                let trades = TOTAL_TRADES.load(Ordering::Relaxed);
                let wins = TOTAL_WINS.load(Ordering::Relaxed);
                let wr = if trades > 0 { (wins as f64 / trades as f64) * 100.0 } else { 0.0 };
                info!("📊 [METRICS] Trades: {} | Wins: {} | WinRate: {:.1}%", trades, wins, wr);
            }
        });
    }

    pub fn record_latency(ms: f64) {
        info!("⏱️ [LATENCY] {:.2}ms", ms);
    }

    pub fn record_trade(won: bool, pnl_sol: f64) {
        TOTAL_TRADES.fetch_add(1, Ordering::Relaxed);
        if won { TOTAL_WINS.fetch_add(1, Ordering::Relaxed); }
        info!("📊 [TRADE] {} | PnL: {:.4} SOL", if won { "WIN" } else { "LOSS" }, pnl_sol);
    }
}
