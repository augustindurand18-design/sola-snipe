use log::{info, error};

pub struct TelegramAlerts {
    bot_token: String,
    chat_id: String,
}

impl TelegramAlerts {
    pub fn new(bot_token: String, chat_id: String) -> Self {
        Self { bot_token, chat_id }
    }

    pub fn send_alert(&self, message: &str) {
        // En production: Utiliser reqwest pour envoyer l'appel HTTP POST
        // https://api.telegram.org/bot<token>/sendMessage
        info!("📢 [TELEGRAM] Envoi de l'alerte : {}", message);
    }

    pub fn alert_entry(&self, mint: &str, amount_sol: f64) {
        self.send_alert(&format!("🟢 [ENTRÉE HFT]\nToken: {}\nMontant: {} SOL\nStatut: TX Pré-forgée", mint, amount_sol));
    }

    pub fn alert_take_profit(&self, mint: &str, pnl_pct: f64, pnl_sol: f64) {
        self.send_alert(&format!("🎯 [TAKE-PROFIT]\nToken: {}\nProfit: +{:.2}%\nGain: +{:.4} SOL", mint, pnl_pct, pnl_sol));
    }

    pub fn alert_stop_loss(&self, mint: &str, pnl_pct: f64, pnl_sol: f64) {
        self.send_alert(&format!("🩸 [STOP-LOSS]\nToken: {}\nPerte: {:.2}%\nPerte: {:.4} SOL", mint, pnl_pct, pnl_sol));
    }

    pub fn alert_leapfrog(&self, mint: &str, dev_wallet: &str) {
        self.send_alert(&format!("🧨 [FRONT-RUN DÉFENSIF]\nToken: {}\nLe Dev ({}) a dump !\n⚡ Jito Bundle envoyé pour Front-Run.", mint, dev_wallet));
    }
}
