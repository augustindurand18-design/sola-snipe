use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct CreateInstruction {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct BuyInstruction {
    pub amount: u64,
    pub max_sol_cost: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct SellInstruction {
    pub amount: u64,
    pub min_sol_output: u64,
}

#[derive(BorshDeserialize, Debug)]
#[allow(dead_code)]
pub struct BondingCurveAccount {
    pub discriminator: u64,
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub token_total_supply: u64,
    pub complete: bool,
}

impl BondingCurveAccount {
    pub fn get_price_sol(&self) -> f64 {
        if self.virtual_token_reserves == 0 { return 0.0; }
        (self.virtual_sol_reserves as f64 / 1e9) / (self.virtual_token_reserves as f64 / 1e6)
    }
}

pub enum ParsedInstruction {
    Create { mint: Pubkey, dev: Pubkey, name: String },
    Buy { mint: Pubkey, buyer: Pubkey, sol_amount: u64 },
    Sell { mint: Pubkey, seller: Pubkey, token_amount: u64 },
    Unknown,
}

pub struct PumpFunParser;

impl PumpFunParser {
    pub const CREATE_DISCRIMINATOR: [u8; 8] = [24, 30, 200, 40, 5, 28, 7, 119];
    pub const BUY_DISCRIMINATOR: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
    pub const SELL_DISCRIMINATOR: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 210];

    pub fn parse_instruction(data: &[u8], accounts: &[String]) -> ParsedInstruction {
        if data.len() < 8 || accounts.len() < 7 { return ParsedInstruction::Unknown; }
        let discriminator = &data[..8];

        if discriminator == Self::CREATE_DISCRIMINATOR {
            if accounts.len() >= 8 {
                let mint = Pubkey::from_str(&accounts[0]).unwrap_or_default();
                let dev = Pubkey::from_str(&accounts[7]).unwrap_or_default();
                let name = CreateInstruction::try_from_slice(&data[8..])
                    .map(|c| c.name)
                    .unwrap_or_else(|_| "UNKNOWN".to_string());
                return ParsedInstruction::Create { mint, dev, name };
            }
        } else if discriminator == Self::BUY_DISCRIMINATOR {
            if let Ok(buy_data) = BuyInstruction::try_from_slice(&data[8..]) {
                let mint = Pubkey::from_str(&accounts[2]).unwrap_or_default();
                let buyer = Pubkey::from_str(&accounts[6]).unwrap_or_default();
                return ParsedInstruction::Buy { mint, buyer, sol_amount: buy_data.max_sol_cost };
            }
        } else if discriminator == Self::SELL_DISCRIMINATOR {
            if let Ok(sell_data) = SellInstruction::try_from_slice(&data[8..]) {
                let mint = Pubkey::from_str(&accounts[2]).unwrap_or_default();
                let seller = Pubkey::from_str(&accounts[6]).unwrap_or_default();
                return ParsedInstruction::Sell { mint, seller, token_amount: sell_data.amount };
            }
        }
        ParsedInstruction::Unknown
    }

    pub fn parse_bonding_curve(data: &[u8]) -> anyhow::Result<BondingCurveAccount> {
        Ok(BondingCurveAccount::try_from_slice(data)?)
    }
}
