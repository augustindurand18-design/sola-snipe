use yellowstone_grpc_client::GeyserServiceClient;
use yellowstone_grpc_proto::geyser::{
    subscribe_update::UpdateOneof, SubscribeRequest, SubscribeRequestFilterTransactions,
};
use futures::stream::StreamExt;
use log::{info, warn, error};
use std::collections::HashMap;
use crate::parser::{PumpFunParser, ParsedInstruction};
use crate::strategy::RugRidingStrategy;
use crate::filters::TokenFilter;

pub struct GrpcEngine {
    client: GeyserServiceClient,
}

impl GrpcEngine {
    pub async fn new(endpoint: &str, x_token: Option<&str>) -> anyhow::Result<Self> {
        let mut client = GeyserServiceClient::connect(endpoint, x_token, None).await?;
        Ok(Self { client })
    }

    pub async fn subscribe_pump_fun(&mut self, mut risk_manager: crate::risk_manager::RiskManager) -> anyhow::Result<()> {
        let mut strategy = RugRidingStrategy::new(risk_manager);
        
        loop {
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
                entry: HashMap::new(),
                blocks: HashMap::new(),
                blocks_meta: HashMap::new(),
                commitment: None,
                accounts_data_slice: vec![],
                ping: None,
            };

            match self.client.subscribe_with_request(Some(request)).await {
                Ok((_, mut stream)) => {
                    info!("🔌 Connecté au stream gRPC Yellowstone");
                    while let Some(message) = stream.next().await {
            match message {
                Ok(msg) => {
                    if let Some(update) = msg.update_oneof {
                        match update {
                            UpdateOneof::Transaction(tx_update) => {
                                if let Some(tx) = tx_update.transaction {
                                    if let Some(message) = tx.message {
                                        // On récupère les adresses (account_keys)
                                        let account_keys: Vec<String> = message.account_keys
                                            .into_iter()
                                            .map(|key| bs58::encode(key).into_string())
                                            .collect();

                                        for ix in message.instructions {
                                            // Parseur Binaire Zéro-JSON
                                            let parsed = PumpFunParser::parse_instruction(&ix.data, &account_keys);
                                            
                                            match parsed {
                                                ParsedInstruction::Create { mint, dev } => {
                                                    // Simulation : on récupère le sol investi dans la meta et les cluster buyers
                                                    let initial_buy = 2.5; // Exemple simulé
                                                    let cluster = vec![]; // Exemple simulé
                                                    
                                                    // APPLICATION DU FILTRE
                                                    // Note: En production, "has_socials" et "name" sont extraits de l'instruction Create
                                                    if TokenFilter::is_optimal_rug_ride(initial_buy, 5, true, "TOKEN_NAME") {
                                                        strategy.on_creation_detected(mint, dev, initial_buy, cluster);
                                                    }
                                                },
                                                ParsedInstruction::Sell { mint, seller, .. } => {
                                                    strategy.on_dev_sell_detected(seller);
                                                },
                                                ParsedInstruction::Buy { .. } => {
                                                    // Calcul de l'impact sur le prix ou tracking d'autres wallets
                                                },
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                            UpdateOneof::Account(acc_update) => {
                                if let Some(acc) = acc_update.account {
                                    use std::str::FromStr;
                                    let mint = solana_sdk::pubkey::Pubkey::from_str(&acc.pubkey).unwrap_or_default();
                                    
                                    if let Ok(bonding_curve) = PumpFunParser::parse_bonding_curve(&acc.data) {
                                        let price = bonding_curve.get_price_sol();
                                        strategy.on_price_update(&mint, price);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    error!("⚠️ Erreur du stream interne : {:?}", e);
                    break; // Sort de la boucle interne pour relancer la souscription
                }
            }
        } // Fin du while
        } // Fin du Ok
        Err(e) => {
            error!("⚠️ Échec de connexion gRPC : {:?}. Reconnexion dans 1 seconde...", e);
        }
        } // Fin du match
        
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    } // Fin de la boucle externe (loop)
}
