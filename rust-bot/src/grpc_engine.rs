use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::geyser::{
    subscribe_update::UpdateOneof, SubscribeRequest, SubscribeRequestFilterTransactions,
};
use futures::stream::StreamExt;
use log::{info, error};
use std::collections::HashMap;
use crate::parser::{PumpFunParser, ParsedInstruction};
use crate::strategy::RugRidingStrategy;
use crate::filters::TokenFilter;

pub struct GrpcEngine {
    endpoint: String,
}

impl GrpcEngine {
    pub async fn new(endpoint: &str) -> anyhow::Result<Self> {
        info!("🔧 Moteur gRPC initialisé (endpoint: {})", endpoint);
        Ok(Self {
            endpoint: endpoint.to_string(),
        })
    }

    pub async fn subscribe_pump_fun(&mut self, risk_manager: crate::risk_manager::RiskManager) -> anyhow::Result<()> {
        let mut strategy = RugRidingStrategy::new(risk_manager);
        
        loop {
            info!("🔌 Tentative de connexion gRPC à {}...", self.endpoint);

            let client_result = GeyserGrpcClient::build_from_shared(self.endpoint.clone());
            let mut client = match client_result {
                Ok(builder) => {
                    match builder.connect().await {
                        Ok(c) => c,
                        Err(e) => {
                            error!("⚠️ Échec de connexion gRPC : {:?}. Reconnexion dans 2 secondes...", e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                            continue;
                        }
                    }
                },
                Err(e) => {
                    error!("⚠️ Erreur de configuration gRPC : {:?}. Reconnexion dans 2 secondes...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }
            };

            let mut transactions = HashMap::new();
            transactions.insert(
                "pump_fun_filter".to_string(),
                SubscribeRequestFilterTransactions {
                    vote: Some(false),
                    failed: Some(false),
                    signature: None,
                    account_include: vec!["6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string()],
                    account_exclude: vec![],
                    account_required: vec![],
                },
            );

            let request = SubscribeRequest {
                slots: HashMap::new(),
                accounts: HashMap::new(),
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
                    info!("✅ Connecté au stream gRPC Yellowstone ! En écoute sur Pump.fun...");
                    while let Some(message) = stream.next().await {
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
                                                                ParsedInstruction::Create { mint, dev } => {
                                                                    let initial_buy = 2.5;
                                                                    let cluster = vec![];
                                                                    if TokenFilter::is_optimal_rug_ride(initial_buy, 5, true, "TOKEN_NAME") {
                                                                        strategy.on_creation_detected(mint, dev, initial_buy, cluster);
                                                                    }
                                                                },
                                                                ParsedInstruction::Sell { seller, .. } => {
                                                                    strategy.on_dev_sell_detected(seller);
                                                                },
                                                                _ => {}
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        _ => {}
                                    }
                                }
                            },
                            Err(e) => {
                                error!("⚠️ Déconnexion du stream : {:?}. Reconnexion...", e);
                                break;
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("⚠️ Échec de souscription gRPC : {:?}. Reconnexion dans 2 secondes...", e);
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }
}
