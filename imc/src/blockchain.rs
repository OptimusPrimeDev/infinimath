use sha3::{Sha3_256, Digest as Sha3Digest};
use sha2::Sha256;
use p256::ecdsa::{SigningKey, VerifyingKey};
use p256::ecdsa::signature::{Signer, Verifier};
use p256::FieldBytes; // Ensure this import is present
use common::wallet::{OptionalSerializableSignature, SerializableSignature};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::str::FromStr;
use std::fs::File;
use std::io::{self, Write, Read};
use bigdecimal::{BigDecimal, Zero};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: BigDecimal,
    pub fee: BigDecimal,
    pub signature: OptionalSerializableSignature,
}

impl Transaction {
    pub fn is_well_formed(&self) -> bool {
        !self.sender.is_empty() && !self.receiver.is_empty() && self.amount > BigDecimal::zero() && self.fee >= BigDecimal::zero()
    }

    pub fn sign(&mut self, private_key: &SigningKey) {
        let message = self.hash();
        self.signature = OptionalSerializableSignature(Some(SerializableSignature(private_key.sign(message.as_bytes()))));
    }

    pub fn verify(&self, public_key: &VerifyingKey) -> Result<bool, p256::ecdsa::Error> {
        if let Some(signature) = &self.signature.0 {
            let message = self.hash();
            public_key.verify(message.as_bytes(), &signature.0).map(|_| true)
        } else {
            Ok(false)
        }
    }

    pub fn hash(&self) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(&self.sender);
        hasher.update(&self.receiver);
        hasher.update(&self.amount.to_string());
        hasher.update(&self.fee.to_string());
        format!("{:x}", hasher.finalize())
    }

    pub fn validate(&self, blockchain: &Blockchain) -> bool {
        let sender_balance = blockchain.get_balance(&self.sender);

        if sender_balance < self.amount.clone() + self.fee.clone() {
            println!("Transaction from {} to {} is invalid: insufficient balance.", self.sender, self.receiver);
            return false;
        }

        self.is_well_formed()
    }
}

#[derive(Debug)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub mining_reward: BigDecimal,
    pub balances: HashMap<String, BigDecimal>,
    pub miner_contributions: HashMap<String, u64>,
    pub liquidity_wallet: String,
    pub rewards_wallet: String,
    pub public_keys: HashMap<String, VerifyingKey>,
    pub target_block_time: Duration,
    pub difficulty: usize,
}

impl Blockchain {
    pub fn new() -> Self {
        let target_block_time = Duration::from_secs(450); // 7.5 minutes
        let initial_difficulty = 2; // Initial difficulty level

        let mut balances = HashMap::new();
        balances.insert("System".to_string(), BigDecimal::from_str("1000000000000").unwrap());
        balances.insert("Alice".to_string(), BigDecimal::from_str("1000").unwrap());

        let mut blockchain = Blockchain {
            blocks: vec![],
            pending_transactions: vec![],
            mining_reward: BigDecimal::from_str("40.0").unwrap(),
            balances,
            miner_contributions: HashMap::new(),
            liquidity_wallet: "LiquidityWallet".to_string(),
            rewards_wallet: "RewardsWallet".to_string(),
            public_keys: HashMap::new(),
            target_block_time,
            difficulty: initial_difficulty,
        };

        let mut genesis_block = Block::new(0, 0, "0".to_string(), "Genesis Block".to_string(), 0);
        genesis_block.mine_block(initial_difficulty);
        blockchain.blocks.push(genesis_block);

        blockchain
    }

    pub fn create_transaction(&mut self, transaction: Transaction) {
        if transaction.validate(self) {
            println!("Transaction from {} to {} is valid and added to pending transactions.", transaction.sender, transaction.receiver);
            self.pending_transactions.push(transaction);
        } else {
            println!("Transaction from {} to {} is invalid and not added to pending transactions.", transaction.sender, transaction.receiver);
        }
    }

    pub fn mine_pending_transactions(&mut self, miner_address: String) {
        println!("Mining transactions by {}", miner_address);
        let previous_block = self.blocks.last().unwrap();
        let start_time = SystemTime::now();
        
        let mut block = Block::new(
            previous_block.index + 1,
            start_time.duration_since(UNIX_EPOCH).unwrap().as_millis(),
            previous_block.hash.clone(),
            serde_json::to_string(&self.pending_transactions).unwrap(),
            0,
        );

        block.mine_block(self.difficulty);
        let end_time = SystemTime::now();
        self.blocks.push(block);

        self.adjust_difficulty(start_time, end_time);

        for transaction in &self.pending_transactions {
            let sender_balance = self.balances.entry(transaction.sender.clone()).or_insert(BigDecimal::zero());
            *sender_balance -= &transaction.amount + &transaction.fee;

            let receiver_balance = self.balances.entry(transaction.receiver.clone()).or_insert(BigDecimal::zero());
            *receiver_balance += &transaction.amount;

            let liquidity_fee = &transaction.fee / BigDecimal::from(2);
            let rewards_fee = &transaction.fee / BigDecimal::from(2);

            let liquidity_balance = self.balances.entry(self.liquidity_wallet.clone()).or_insert(BigDecimal::zero());
            *liquidity_balance += liquidity_fee;

            let rewards_balance = self.balances.entry(self.rewards_wallet.clone()).or_insert(BigDecimal::zero());
            *rewards_balance += rewards_fee;
        }

        let miner_contribution = self.miner_contributions.entry(miner_address.clone()).or_insert(0);
        *miner_contribution += 1;

        self.distribute_rewards(miner_address);

        self.pending_transactions.clear();
    }

    fn adjust_difficulty(&mut self, start_time: SystemTime, end_time: SystemTime) {
        let block_time = end_time.duration_since(start_time).unwrap();
        let adjustment_factor = 0.1; // Adjust this factor to control sensitivity

        if block_time < self.target_block_time {
            // Decrease difficulty if block time is less than target
            self.difficulty = (self.difficulty as f64 * (1.0 + adjustment_factor)).ceil() as usize;
        } else if block_time > self.target_block_time {
            // Increase difficulty if block time is greater than target
            self.difficulty = (self.difficulty as f64 * (1.0 - adjustment_factor)).max(1.0) as usize;
        }

        println!("Adjusted difficulty to: {}", self.difficulty);
    }

    pub fn distribute_rewards(&mut self, winner: String) {
        let total_contributions: u64 = self.miner_contributions.values().sum();
        if total_contributions == 0 {
            return;
        }

        // Winner gets 70% of the mining reward
        let winner_reward = &self.mining_reward * BigDecimal::from_str("0.7").unwrap();
        let winner_balance = self.balances.entry(winner.clone()).or_insert(BigDecimal::zero());
        *winner_balance += winner_reward;

        // Everyone, including the winner, gets a share of the remaining 30% based on their contributions
        let remaining_reward = &self.mining_reward * BigDecimal::from_str("0.3").unwrap();
        for (miner, contribution) in &self.miner_contributions {
            let reward = &remaining_reward * BigDecimal::from(*contribution as i64) / BigDecimal::from(total_contributions as i64);
            let miner_balance = self.balances.entry(miner.clone()).or_insert(BigDecimal::zero());
            *miner_balance += reward;
        }

        self.miner_contributions.clear();
    }

    pub fn get_balance(&self, address: &str) -> BigDecimal {
        self.balances.get(address).cloned().unwrap_or(BigDecimal::zero())
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

            let transactions: Vec<Transaction> = match serde_json::from_str(&current_block.data) {
                Ok(txs) => txs,
                Err(_) => return false,
            };
            for transaction in &transactions {
                let public_key = match self.get_public_key(&transaction.sender) {
                    Ok(key) => key,
                    Err(e) => {
                        println!("Failed to get public key for {}: {:?}", transaction.sender, e);
                        return false;
                    }
                };
                match transaction.verify(&public_key) {
                    Ok(true) => (),
                    Ok(false) => {
                        println!("Transaction verification failed for transaction from {} to {}", transaction.sender, transaction.receiver);
                        return false;
                    }
                    Err(e) => {
                        println!("Transaction verification error for transaction from {} to {}: {:?}", transaction.sender, transaction.receiver, e);
                        return false;
                    }
                }
                if !transaction.validate(self) {
                    println!("Transaction validation failed for transaction from {} to {}", transaction.sender, transaction.receiver);
                    return false;
                }
            }
        }
        true
    }

    fn get_public_key(&self, address: &str) -> Result<VerifyingKey, p256::ecdsa::Error> {
        self.public_keys.get(address).cloned().ok_or_else(|| p256::ecdsa::Error::new())
    }

    pub fn store_public_key(&mut self, address: &str, public_key: VerifyingKey) {
        self.public_keys.insert(address.to_string(), public_key);
    }

    pub fn save_key_to_file(key_path: &str, key_data: &[u8]) -> io::Result<()> {
        let mut file = File::create(key_path)?;
        file.write_all(key_data)?;
        Ok(())
    }

    pub fn load_key_from_file(key_path: &str) -> io::Result<FieldBytes> {
        let mut file = File::open(key_path)?;
        let mut key_data = FieldBytes::default();
        file.read_exact(&mut key_data)?;
        Ok(key_data)
    }
}

// Dummy Block struct for demonstration purposes
#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u128,
    pub previous_hash: String,
    pub hash: String,
    pub data: String,
    pub nonce: u64,
}

impl Block {
    pub fn new(index: u64, timestamp: u128, previous_hash: String, data: String, nonce: u64) -> Self {
        let mut block = Block {
            index,
            timestamp,
            previous_hash,
            hash: String::new(),
            data,
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
        hasher.update(&self.data);
        hasher.update(self.nonce.to_string());
        format!("{:x}", hasher.finalize())
    }

    pub fn mine_block(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        while &self.hash[..difficulty] != target {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        println!("Block mined with hash: {}", self.hash);
    }
}