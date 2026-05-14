use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use log::info;
use std::sync::Arc;

pub struct JitoEngine {
    payer: Arc<Keypair>,
    endpoint: String,
}

impl JitoEngine {
    pub async fn new(endpoint: &str, payer: Keypair) -> anyhow::Result<Self> {
        info!("⚙️ JITO BLOCK ENGINE initialisé (endpoint: {}, payer: {})", endpoint, payer.pubkey());
        Ok(Self {
            payer: Arc::new(payer),
            endpoint: endpoint.to_string(),
        })
    }

    pub async fn send_bundle(&self, transactions: Vec<VersionedTransaction>) -> anyhow::Result<()> {
        info!("📦 [JITO] Bundle {} TX → {} (payer: {})", transactions.len(), self.endpoint, self.payer.pubkey());
        Ok(())
    }
}
