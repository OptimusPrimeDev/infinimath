use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubChainBlock {
    pub block_number: u64,
    pub timestamp: u64,
    pub result: String,
    pub prev_block_hash: String,
    pub nonce: u64,
    pub hash: String,
}