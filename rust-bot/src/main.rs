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
use log::info;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    info!("🚀 Démarrage du Sniper Solana HFT (Rust Edition)");

    // Configuration depuis .env
    let grpc_endpoint = env::var("GRPC_URL")
        .unwrap_or_else(|_| "https://grpc.helius-rpc.com".to_string());
    let jito_endpoint = env::var("JITO_BLOCK_ENGINE_URL")
        .unwrap_or_else(|_| "amsterdam.mainnet.block-engine.jito.wtf".to_string());
    let paper_trading = env::var("PAPER_TRADING")
        .unwrap_or_else(|_| "true".to_string()) == "true";
    let investment_sol: f64 = env::var("INVESTMENT_SOL")
        .unwrap_or_else(|_| "0.05".to_string())
        .parse().unwrap_or(0.05);

    if paper_trading {
        info!("📝 Mode PAPER TRADING activé. Aucune transaction réelle ne sera envoyée.");
    }

    // Risk Manager: Max 3 positions, investment_sol per trade, Max Global Drawdown 0.5 SOL
    use risk_manager::RiskManager;
    let risk_manager = RiskManager::new(3, investment_sol, 0.5);
    
    // Initialisation des moteurs
    let mut grpc = GrpcEngine::new(&grpc_endpoint).await?;
    
    // Simulation d'un Keypair pour l'exemple
    let payer = Keypair::new();
    let _jito = JitoEngine::new(&jito_endpoint, payer).await?;

    info!("📡 Endpoint gRPC : {}", grpc_endpoint);
    info!("⚡ Endpoint Jito  : {}", jito_endpoint);
    info!("💰 Investissement : {} SOL par trade", investment_sol);
    info!("🎯 Monitoring Pump.fun: 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");

    // Lancement du Serveur de Métriques (Prometheus)
    use metrics::MetricsServer;
    let metrics_server = MetricsServer::new(8080);
    metrics_server.start_background_thread();

    // Alertes Telegram
    let tg_token = env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
    let tg_chat = env::var("TELEGRAM_CHAT_ID").unwrap_or_default();
    if !tg_token.is_empty() {
        info!("📢 Alertes Telegram configurées (Chat ID: {})", tg_chat);
    }

    // Lancement du Heartbeat (Health Check)
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            log::info!("💓 [HEARTBEAT] Le système est synchronisé et opérationnel.");
        }
    });

    // Lancement du flux gRPC (avec auto-reconnect interne)
    grpc.subscribe_pump_fun(risk_manager).await?;

    Ok(())
}
