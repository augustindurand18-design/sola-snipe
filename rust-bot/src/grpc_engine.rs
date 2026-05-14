use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::geyser::{
    subscribe_update::UpdateOneof, SubscribeRequest, SubscribeRequestFilterTransactions,
    SubscribeRequestFilterAccounts,
};
use futures::stream::StreamExt;
use log::{info, warn, error};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use crate::parser::{PumpFunParser, ParsedInstruction};
use crate::strategy::RugRidingStrategy;
use crate::filters::TokenFilter;
use crate::telegram::TelegramAlerts;
use crate::jito_engine::JitoEngine;
use crate::metrics::MetricsServer;
use solana_client::pubsub_client::PubsubClient;

pub struct GrpcEngine {
    endpoint: String,
    telegram: Arc<TelegramAlerts>,
    jito: Arc<JitoEngine>,
}

impl GrpcEngine {
    pub async fn new(endpoint: &str, telegram: Arc<TelegramAlerts>, jito: Arc<JitoEngine>) -> anyhow::Result<Self> {
        info!("🔧 Moteur Hybride initialisé (endpoint: {})", endpoint);
        Ok(Self { endpoint: endpoint.to_string(), telegram, jito })
    }

    pub async fn subscribe_pump_fun(&mut self, risk_manager: crate::risk_manager::RiskManager) -> anyhow::Result<()> {
        let mut strategy = RugRidingStrategy::new(risk_manager, self.telegram.clone(), self.jito.clone());

        // Détection automatique du mode (gRPC vs RPC/WSS)
        if self.endpoint.starts_with("http") && !self.endpoint.contains("grpc") {
            warn!("⚠️ Mode RPC/WebSocket détecté (Mode Test Gratuit).");
            self.run_websocket_fallback(&mut strategy).await
        } else {
            info!("🚀 Mode gRPC Yellowstone détecté (Mode Payant/HFT).");
            self.run_grpc_loop(&mut strategy).await
        }
    }

    async fn run_websocket_fallback(&self, strategy: &mut RugRidingStrategy) -> anyhow::Result<()> {
        let wss_url = self.endpoint
            .replace("https://", "wss://")
            .replace("http://", "ws://");
        
        loop {
            info!("🔌 Connexion WebSocket à {}...", wss_url);
            match PubsubClient::program_subscribe(
                &wss_url,
                &solana_sdk::pubkey::Pubkey::from_str("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P").unwrap(),
                None,
            ) {
                Ok((_unsub, receiver)) => {
                    info!("✅ WebSocket connecté ! En écoute sur Pump.fun (Fallback Mode)...");
                    while let Ok(response) = receiver.recv() {
                        let tick = Instant::now();
                        if let Some(data) = response.value.account.data.decode() {
                            if let Ok(bonding_curve) = PumpFunParser::parse_bonding_curve(&data) {
                                let price = bonding_curve.get_price_sol();
                                if price > 0.0 {
                                    let pubkey = solana_sdk::pubkey::Pubkey::from_str(&response.value.pubkey).unwrap_or_default();
                                    strategy.on_price_update(&pubkey, price).await;
                                }
                            }
                        }
                        MetricsServer::record_latency(tick.elapsed().as_secs_f64() * 1000.0);
                    }
                }
                Err(e) => {
                    error!("⚠️ Erreur WebSocket : {:?}. Reconnexion dans 5s...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn run_grpc_loop(&self, strategy: &mut RugRidingStrategy) -> anyhow::Result<()> {
        loop {
            // ... (Code gRPC existant optimisé) ...
            let mut client = match GeyserGrpcClient::build_from_shared(self.endpoint.clone()) {
                Ok(builder) => match builder.connect().await {
                    Ok(c) => c,
                    Err(_) => {
                        error!("⚠️ Échec gRPC. Vérifiez votre abonnement. Retry 5s...");
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }
                },
                Err(_) => break Err(anyhow::anyhow!("URL gRPC invalide")),
            };

            let mut transactions = HashMap::new();
            transactions.insert("pump_fun_tx".to_string(), SubscribeRequestFilterTransactions {
                vote: Some(false), failed: Some(false), signature: None,
                account_include: vec!["6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string()],
                account_exclude: vec![], account_required: vec![],
            });

            let mut accounts = HashMap::new();
            accounts.insert("pump_fun_acc".to_string(), SubscribeRequestFilterAccounts {
                account: vec![], owner: vec!["6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string()], filters: vec![],
            });

            let request = SubscribeRequest {
                slots: HashMap::new(), accounts, transactions, transactions_status: HashMap::new(),
                entry: HashMap::new(), blocks: HashMap::new(), blocks_meta: HashMap::new(),
                commitment: None, accounts_data_slice: vec![], ping: None,
            };

            if let Ok((_, mut stream)) = client.subscribe_with_request(Some(request)).await {
                while let Some(Ok(msg)) = stream.next().await {
                    let tick = Instant::now();
                    if let Some(UpdateOneof::Transaction(tx_update)) = msg.update_oneof {
                        // ... traitement transaction ...
                        if let Some(tx) = tx_update.transaction.and_then(|t| t.transaction) {
                            if let Some(msg_inner) = tx.message {
                                let keys: Vec<String> = msg_inner.account_keys.into_iter().map(|k| bs58::encode(k).into_string()).collect();
                                for ix in msg_inner.instructions {
                                    match PumpFunParser::parse_instruction(&ix.data, &keys) {
                                        ParsedInstruction::Create { mint, dev, name } => {
                                            if TokenFilter::is_optimal_rug_ride(2.5, 6, true, &name) {
                                                strategy.on_creation_detected(mint, dev, 2.5, vec![]).await;
                                            }
                                        },
                                        ParsedInstruction::Sell { mint, seller, token_amount } => {
                                            strategy.on_dev_sell_detected(seller, mint, token_amount).await;
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    MetricsServer::record_latency(tick.elapsed().as_secs_f64() * 1000.0);
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }
}

use std::str::FromStr;
