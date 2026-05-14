use solana_sdk::{
    signature::Keypair,
    transaction::VersionedTransaction,
};
use log::{info, warn};
use std::sync::Arc;

pub struct JitoEngine {
    payer: Arc<Keypair>,
    endpoint: String,
}

impl JitoEngine {
    pub async fn new(endpoint: &str, payer: Keypair) -> anyhow::Result<Self> {
        warn!("⚙️ JITO BLOCK ENGINE: Mode Simulation / Paper Trading Activé");
        Ok(Self {
            payer: Arc::new(payer),
            endpoint: endpoint.to_string(),
        })
    }

    pub async fn send_bundle(&self, transactions: Vec<VersionedTransaction>) -> anyhow::Result<()> {
        info!("📦 Simulation Jito : Envoi d'un Bundle de {} transactions vers {}", transactions.len(), self.endpoint);
        Ok(())
    }
}
