use crate::subchain_block::SubChainBlock;
use sha2::{Sha256, Digest};

pub fn calculate_subchain_hash(block: &SubChainBlock) -> String {
    let block_data = serde_json::to_string(&block).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(block_data);
    format!("{:x}", hasher.finalize())
}

pub fn mine_subchain_block(block: &mut SubChainBlock, difficulty: usize) -> u64 {
    let target = "0".repeat(difficulty);
    let mut nonce = 0;
    loop {
        block.nonce = nonce;
        block.hash = calculate_subchain_hash(&block);
        if block.hash.starts_with(&target) {
            break;
        }
        nonce += 1;
    }
    nonce
}

// Example utility module for a specific sub-chain algorithm
pub mod subchain1_utils {
    pub fn specific_algorithm(input: &str) -> String {
        // Perform specific mathematical computation
        format!("Result of computation for input: {}", input)
    }
}