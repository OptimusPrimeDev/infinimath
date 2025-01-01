use crate::subchain_block::SubChainBlock;
use crate::subchain_pow::{calculate_subchain_hash, mine_subchain_block};
use std::collections::HashMap;
use chrono::Utc;

pub struct SubChain {
    pub blocks: Vec<SubChainBlock>,
    pub balances: HashMap<String, f64>,
}

impl SubChain {
    pub fn new() -> Self {
        let genesis_block = SubChainBlock {
            block_number: 0,
            timestamp: Utc::now().timestamp() as u64,
            result: String::from("Genesis Block"),
            prev_block_hash: String::new(),
            nonce: 0,
            hash: String::new(),
        };

        let mut subchain = SubChain {
            blocks: vec![genesis_block],
            balances: HashMap::new(),
        };

        subchain.blocks[0].hash = calculate_subchain_hash(&subchain.blocks[0]);
        subchain
    }

    pub fn add_block(&mut self, block: SubChainBlock) {
        self.blocks.push(block);
    }

    pub fn get_latest_block(&self) -> &SubChainBlock {
        self.blocks.last().unwrap()
    }

    pub fn get_last_block_hash(&self) -> String {
        self.get_latest_block().hash.clone()
    }

    pub fn mine_block(&mut self, block: &mut SubChainBlock, difficulty: usize) {
        mine_subchain_block(block, difficulty);
        self.add_block(block.clone());
    }

    pub fn get_balance(&self, address: &str) -> f64 {
        *self.balances.get(address).unwrap_or(&0.0)
    }
}