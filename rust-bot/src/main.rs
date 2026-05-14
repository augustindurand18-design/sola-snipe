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
    info!("🚀 Démarrage du Sniper Solana HFT (Rust V3 - Full Integration)");

    // Configuration depuis .env
    let grpc_endpoint = env::var("GRPC_URL").unwrap_or_else(|_| "https://grpc.helius-rpc.com".to_string());
    let jito_endpoint = env::var("JITO_BLOCK_ENGINE_URL").unwrap_or_else(|_| "amsterdam.mainnet.block-engine.jito.wtf".to_string());
    let investment_sol: f64 = env::var("INVESTMENT_SOL").unwrap_or_else(|_| "0.05".to_string()).parse().unwrap_or(0.05);

    // Telegram
    let tg_token = env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
    let tg_chat = env::var("TELEGRAM_CHAT_ID").unwrap_or_default();
    if tg_token.is_empty() || tg_chat.is_empty() {
        error!("❌ TELEGRAM_BOT_TOKEN ou CHAT_ID manquant dans .env !");
        return Err(anyhow::anyhow!("Configuration Telegram incomplète"));
    }
    let telegram = Arc::new(TelegramAlerts::new(tg_token, tg_chat));

    // Jito Engine
    let payer = Keypair::new();
    let jito = Arc::new(JitoEngine::new(&jito_endpoint, payer).await?);

    // Risk Manager
    use risk_manager::RiskManager;
    let risk_manager = RiskManager::new(3, investment_sol, 0.5);

    // gRPC Engine (avec Telegram + Jito intégrés)
    let mut grpc = GrpcEngine::new(&grpc_endpoint, telegram.clone(), jito.clone()).await?;

    info!("📡 gRPC    : {}", grpc_endpoint);
    info!("⚡ Jito    : {}", jito_endpoint);
    info!("💰 Trade   : {} SOL", investment_sol);
    info!("📢 Telegram: Connecté");

    // Metrics
    use metrics::MetricsServer;
    let metrics_server = MetricsServer::new(8080);
    metrics_server.start_background_thread();

    // Heartbeat
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            log::info!("💓 [HEARTBEAT] Bot opérationnel.");
        }
    });

    // Lancement du Sniper
    grpc.subscribe_pump_fun(risk_manager).await?;

    Ok(())
}
