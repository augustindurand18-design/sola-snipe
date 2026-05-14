use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::geyser::{
    subscribe_update::UpdateOneof, SubscribeRequest, SubscribeRequestFilterTransactions,
    SubscribeRequestFilterAccounts,
};
use futures::stream::StreamExt;
use log::{info, error};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use crate::parser::{PumpFunParser, ParsedInstruction};
use crate::strategy::RugRidingStrategy;
use crate::filters::TokenFilter;
use crate::telegram::TelegramAlerts;
use crate::jito_engine::JitoEngine;
use crate::metrics::MetricsServer;

pub struct GrpcEngine {
    endpoint: String,
    telegram: Arc<TelegramAlerts>,
    jito: Arc<JitoEngine>,
}

impl GrpcEngine {
    pub async fn new(endpoint: &str, telegram: Arc<TelegramAlerts>, jito: Arc<JitoEngine>) -> anyhow::Result<Self> {
        info!("🔧 Moteur gRPC initialisé (endpoint: {})", endpoint);
        Ok(Self { endpoint: endpoint.to_string(), telegram, jito })
    }

    pub async fn subscribe_pump_fun(&mut self, risk_manager: crate::risk_manager::RiskManager) -> anyhow::Result<()> {
        let mut strategy = RugRidingStrategy::new(risk_manager, self.telegram.clone(), self.jito.clone());

        loop {
            info!("🔌 Connexion gRPC à {}...", self.endpoint);

            let mut client = match GeyserGrpcClient::build_from_shared(self.endpoint.clone()) {
                Ok(builder) => match builder.connect().await {
                    Ok(c) => c,
                    Err(e) => {
                        error!("⚠️ Connexion échouée: {:?}. Retry 2s...", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        continue;
                    }
                },
                Err(e) => {
                    error!("⚠️ Config gRPC invalide: {:?}. Retry 2s...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }
            };

            // Filtre Transactions Pump.fun
            let mut transactions = HashMap::new();
            transactions.insert("pump_fun_tx".to_string(), SubscribeRequestFilterTransactions {
                vote: Some(false),
                failed: Some(false),
                signature: None,
                account_include: vec!["6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string()],
                account_exclude: vec![],
                account_required: vec![],
            });

            // Filtre Accounts Pump.fun (Bonding Curves pour le prix)
            let mut accounts = HashMap::new();
            accounts.insert("pump_fun_accounts".to_string(), SubscribeRequestFilterAccounts {
                account: vec![],
                owner: vec!["6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string()],
                filters: vec![],
            });

            let request = SubscribeRequest {
                slots: HashMap::new(),
                accounts,
                transactions,
                transactions_status: HashMap::new(),
                entry: HashMap::new(),
                blocks: HashMap::new(),
                blocks_meta: HashMap::new(),
                commitment: None,
                accounts_data_slice: vec![],
                ping: None,
            };

            match client.subscribe_with_request(Some(request)).await {
                Ok((_, mut stream)) => {
                    info!("✅ Connecté au stream gRPC ! En écoute Pump.fun...");
                    while let Some(message) = stream.next().await {
                        let tick = Instant::now();
                        match message {
                            Ok(msg) => {
                                if let Some(update) = msg.update_oneof {
                                    match update {
                                        UpdateOneof::Transaction(tx_update) => {
                                            if let Some(tx_info) = tx_update.transaction {
                                                if let Some(tx) = tx_info.transaction {
                                                    if let Some(msg_inner) = tx.message {
                                                        let account_keys: Vec<String> = msg_inner.account_keys
                                                            .into_iter()
                                                            .map(|key| bs58::encode(key).into_string())
                                                            .collect();

                                                        for ix in msg_inner.instructions {
                                                            let parsed = PumpFunParser::parse_instruction(&ix.data, &account_keys);
                                                            match parsed {
                                                                ParsedInstruction::Create { mint, dev, name } => {
                                                                    info!("🆕 Nouveau token: {} ({})", name, mint);

                                                                    // Filtres avancés
                                                                    let initial_buy = 2.5;
                                                                    let cluster_count = 6;

                                                                    if !TokenFilter::is_optimal_rug_ride(initial_buy, cluster_count, true, &name) {
                                                                        continue;
                                                                    }
                                                                    // Filtre Maturité (simulation: wallet > 2h)
                                                                    let dev_first_sig_ts = std::time::SystemTime::now()
                                                                        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() - 10000;
                                                                    if !TokenFilter::is_wallet_mature(dev_first_sig_ts) {
                                                                        continue;
                                                                    }
                                                                    // Filtre Sybil
                                                                    let known_scam_hubs: Vec<&str> = vec![];
                                                                    if !TokenFilter::check_sybil_funding_source(&dev.to_string(), &known_scam_hubs) {
                                                                        continue;
                                                                    }

                                                                    strategy.on_creation_detected(mint, dev, initial_buy, vec![]).await;
                                                                },
                                                                ParsedInstruction::Buy { mint, buyer, sol_amount } => {
                                                                    info!("📈 Achat: {} achète {:.4} SOL de {}", buyer, sol_amount as f64 / 1e9, mint);
                                                                },
                                                                ParsedInstruction::Sell { mint, seller, token_amount } => {
                                                                    info!("📉 Vente: {} vend {} tokens de {}", seller, token_amount, mint);
                                                                    strategy.on_dev_sell_detected(seller, mint, token_amount).await;
                                                                },
                                                                ParsedInstruction::Unknown => {}
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        UpdateOneof::Account(acc_update) => {
                                            if let Some(acc_info) = acc_update.account {
                                                // Vérification sécurité Mint
                                                if acc_info.data.len() == 82 {
                                                    TokenFilter::is_mint_safe(&acc_info.data);
                                                }
                                                if let Ok(bonding_curve) = PumpFunParser::parse_bonding_curve(&acc_info.data) {
                                                    let price = bonding_curve.get_price_sol();
                                                    if price > 0.0 {
                                                        let pubkey_bytes: Result<[u8; 32], _> = acc_info.pubkey.try_into();
                                                        if let Ok(bytes) = pubkey_bytes {
                                                            let pubkey = solana_sdk::pubkey::Pubkey::new_from_array(bytes);
                                                            strategy.on_price_update(&pubkey, price).await;
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        _ => {}
                                    }
                                }
                                let latency_ms = tick.elapsed().as_secs_f64() * 1000.0;
                                MetricsServer::record_latency(latency_ms);
                            },
                            Err(e) => {
                                error!("⚠️ Stream coupé: {:?}. Reconnexion...", e);
                                break;
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("⚠️ Souscription échouée: {:?}. Retry 2s...", e);
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }
}
