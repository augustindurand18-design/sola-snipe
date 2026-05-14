use log::{info, error};
use serde_json::json;

pub struct TelegramAlerts {
    bot_token: String,
    chat_id: String,
    client: reqwest::Client,
}

impl TelegramAlerts {
    pub fn new(bot_token: String, chat_id: String) -> Self {
        Self { 
            bot_token, 
            chat_id,
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_alert(&self, message: &str) {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        let body = json!({
            "chat_id": self.chat_id,
            "text": message,
            "parse_mode": "HTML"
        });

        match self.client.post(&url).json(&body).send().await {
            Ok(_) => info!("📢 [TELEGRAM] Alerte envoyée avec succès."),
            Err(e) => error!("❌ [TELEGRAM] Erreur d'envoi : {:?}", e),
        }
    }

    pub async fn alert_entry(&self, mint: &str, amount_sol: f64) {
        let msg = format!(
            "🟢 <b>[ENTRÉE HFT]</b>\n\nToken: <code>{}</code>\nMontant: <b>{} SOL</b>\nStatut: TX Pré-forgée en cache ⚡",
            mint, amount_sol
        );
        self.send_alert(&msg).await;
    }

    pub async fn alert_take_profit(&self, mint: &str, pnl_pct: f64, pnl_sol: f64) {
        let msg = format!(
            "🎯 <b>[TAKE-PROFIT]</b>\n\nToken: <code>{}</code>\nProfit: <b>+{:.2}%</b>\nGain: <b>+{:.4} SOL</b> 💰",
            mint, pnl_pct, pnl_sol
        );
        self.send_alert(&msg).await;
    }

    pub async fn alert_stop_loss(&self, mint: &str, pnl_pct: f64, pnl_sol: f64) {
        let msg = format!(
            "🩸 <b>[STOP-LOSS]</b>\n\nToken: <code>{}</code>\nPerte: <b>{:.2}%</b>\nMontant: <b>{:.4} SOL</b>",
            mint, pnl_pct, pnl_sol
        );
        self.send_alert(&msg).await;
    }

    pub async fn alert_leapfrog(&self, mint: &str, dev_wallet: &str) {
        let msg = format!(
            "🧨 <b>[FRONT-RUN DÉFENSIF]</b>\n\nToken: <code>{}</code>\nLe Dev (<code>{}</code>) a dump !\n⚡ <b>Jito Bundle envoyé (Leapfrog)</b>.",
            mint, dev_wallet
        );
        self.send_alert(&msg).await;
    }
}
