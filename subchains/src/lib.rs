use sha2::{Sha256, Digest}; // Import the SHA-256 hashing library
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u128,
    pub previous_hash: String,
    pub hash: String,
    pub result: String,
    pub nonce: u64,
}

impl Block {
    pub fn new(index: u64, timestamp: u128, previous_hash: String, result: String, nonce: u64) -> Self {
        let mut block = Block {
            index,
            timestamp,
            previous_hash: previous_hash.clone(),
            hash: String::new(),
            result,
            nonce,
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_string());
        hasher.update(self.timestamp.to_string());
        hasher.update(&self.previous_hash);
        hasher.update(&self.result);
        hasher.update(self.nonce.to_string());
        format!("{:x}", hasher.finalize())
    }

    pub fn mine_block(&mut self, difficulty: usize) {
        let target = vec![0; difficulty]; // Difficulty target (leading zeroes)
        while &self.hash[..difficulty] != target.iter().map(|_| "0").collect::<String>() {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        println!("Block mined: {}", self.hash);
    }
}

#[derive(Debug)]
pub struct Subchain {
    pub blocks: Vec<Block>,
    pub mining_reward: f64,
}

impl Subchain {
    pub fn new() -> Self {
        Subchain {
            blocks: vec![Block::new(0, 0, String::from("0"), String::from("Genesis Result"), 0)],
            mining_reward: 2.0,
        }
    }

    pub fn add_block(&mut self, result: String, difficulty: usize) {
        let previous_block = self.blocks.last().unwrap();
        let mut new_block = Block::new(
            previous_block.index + 1,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
            previous_block.hash.clone(),
            result,
            0,
        );
        new_block.mine_block(difficulty);
        self.blocks.push(new_block);
    }

    pub fn is_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let current_block = &self.blocks[i];
            let previous_block = &self.blocks[i - 1];

            if current_block.hash != current_block.calculate_hash() {
                return false;
            }

            if current_block.previous_hash != previous_block.hash {
                return false;
            }
        }
        true
    }
}