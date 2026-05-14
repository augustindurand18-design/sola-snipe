use solana_sdk::{
    signature::{Keypair, Signer},
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
        // En V3 finale (avec abonnement payant), nous utiliserons le crate `jito-protos`
        // via Tonic gRPC. Pour la phase de simulation et de test, le Block Engine
        // est mocké pour éviter les erreurs de compilation liées à la restructuration du repo Jito.
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
