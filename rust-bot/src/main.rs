mod grpc_engine;
mod parser;
mod jito_engine;
mod strategy;
mod filters;
mod risk_manager;
mod telegram;
mod metrics;

use grpc_engine::GrpcEngine;
use jito_engine::JitoEngine;
use solana_sdk::signature::Keypair;
use log::{info, error};
use std::env;
use std::sync::Arc;
use telegram::TelegramAlerts;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    info!("🚀 Démarrage du Sniper Solana HFT (Full Integration)");

    // Configuration depuis .env
    let grpc_endpoint = env::var("GRPC_URL")
        .unwrap_or_else(|_| "https://grpc.helius-rpc.com".to_string());
    let jito_endpoint = env::var("JITO_BLOCK_ENGINE_URL")
        .unwrap_or_else(|_| "amsterdam.mainnet.block-engine.jito.wtf".to_string());
    let investment_sol: f64 = env::var("INVESTMENT_SOL")
        .unwrap_or_else(|_| "0.05".to_string())
        .parse().unwrap_or(0.05);

    // Initialisation Telegram (Arc pour partage thread-safe)
    let tg_token = env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
    let tg_chat = env::var("TELEGRAM_CHAT_ID").unwrap_or_default();
    
    if tg_token.is_empty() || tg_chat.is_empty() {
        error!("❌ [CONFIG] TELEGRAM_BOT_TOKEN ou CHAT_ID manquant dans le .env !");
        return Err(anyhow::anyhow!("Configuration Telegram incomplète"));
    }

    let telegram = Arc::new(TelegramAlerts::new(tg_token, tg_chat));

    // Risk Manager
    use risk_manager::RiskManager;
    let risk_manager = RiskManager::new(3, investment_sol, 0.5);
    
    // Initialisation des moteurs
    let mut grpc = GrpcEngine::new(&grpc_endpoint, telegram.clone()).await?;
    
    let payer = Keypair::new();
    let _jito = JitoEngine::new(&jito_endpoint, payer).await?;

    info!("📡 Flux gRPC : {}", grpc_endpoint);
    info!("📢 Alertes Telegram connectées.");
    info!("📊 Metrics activées sur port 8080.");

    // Lancement du Serveur de Métriques
    use metrics::MetricsServer;
    let metrics_server = MetricsServer::new(8080);
    metrics_server.start_background_thread();

    // Heartbeat
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            log::info!("💓 [HEARTBEAT] Bot Rust opérationnel.");
        }
    });

    // Lancement du Sniper
    grpc.subscribe_pump_fun(risk_manager).await?;

    Ok(())
}
