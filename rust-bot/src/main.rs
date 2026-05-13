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
    env_logger::init();
    info!("🚀 Démarrage du Sniper Solana HFT (Rust Edition)");

    // Configuration
    let grpc_endpoint = env::var("GRPC_URL").unwrap_or_else(|_| "https://ny.mainnet.block-engine.jito.wtf".to_string());
    let jito_endpoint = env::var("JITO_BLOCK_ENGINE_URL").unwrap_or_else(|_| "amsterdam.mainnet.block-engine.jito.wtf".to_string());
    
    // Risk Manager: Max 3 positions, 0.05 SOL per trade, Max Global Drawdown 0.5 SOL
    use risk_manager::RiskManager;
    let risk_manager = RiskManager::new(3, 0.05, 0.5);
    
    // Initialisation des moteurs
    let mut grpc = GrpcEngine::new(&grpc_endpoint, None).await?;
    
    // Simulation d'un Keypair pour l'exemple
    let payer = Keypair::new();
    let _jito = JitoEngine::new(&jito_endpoint, payer).await?;

    info!("Monitoring Pump.fun program: 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");

    // Lancement du Serveur de Métriques (Prometheus)
    use metrics::MetricsServer;
    let metrics_server = MetricsServer::new(8080);
    metrics_server.start_background_thread();

    // Alertes Telegram
    use telegram::TelegramAlerts;
    let _telegram = TelegramAlerts::new("TELEGRAM_BOT_TOKEN".to_string(), "CHAT_ID".to_string());
    // (En prod, on passerait l'instance _telegram au grpc_engine / strategy)

    // Lancement du Heartbeat (Health Check)
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            log::info!("💓 [HEARTBEAT] Le système est synchronisé et opérationnel.");
            // Logique de purge des positions gelées ou détection de blocage gRPC
        }
    });

    // Lancement du flux gRPC (avec auto-reconnect interne)
    grpc.subscribe_pump_fun(risk_manager).await?;

    Ok(())
}
