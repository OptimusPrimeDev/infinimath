use imc::blockchain::{Blockchain, Transaction};
use bigdecimal::BigDecimal;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use rand::Rng;
use rand::seq::SliceRandom; // Import SliceRandom for shuffling
use num_traits::Zero; // Import the Zero trait
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Miner {
    pub address: String,
    pub computing_power: u64, // Simulated computing power
}

fn generate_transactions(miners: &Vec<Miner>, balances: &HashMap<String, BigDecimal>) -> Vec<Transaction> {
    let mut transactions = Vec::new();
    let mut rng = rand::thread_rng();

    // Generate a valid transaction
    let sender_index = rng.gen_range(0..miners.len());
    let receiver_index = rng.gen_range(0..miners.len());
    let sender = &miners[sender_index].address;
    let receiver = &miners[receiver_index].address;

    let mut valid_transaction_generated = false;
    if let Some(sender_balance) = balances.get(sender) {
        if sender_balance > &BigDecimal::from_str("0.001").unwrap() {
            let amount = BigDecimal::from_str("0.001").unwrap();
            let fee = BigDecimal::from_str("0.001").unwrap();

            let transaction = Transaction {
                sender: sender.clone(),
                receiver: receiver.clone(),
                amount,
                fee,
                signature: common::wallet::OptionalSerializableSignature(None),
            };

            transactions.push(transaction);
            valid_transaction_generated = true;
        }
    }

    // Generate invalid transactions
    for _ in 0..4 {
        let sender_index = rng.gen_range(0..miners.len());
        let receiver_index = rng.gen_range(0..miners.len());
        let sender = &miners[sender_index].address;
        let receiver = &miners[receiver_index].address;
        let amount = BigDecimal::from(rng.gen_range(1..100));
        let fee = BigDecimal::from(rng.gen_range(1..10));

        let transaction = Transaction {
            sender: sender.clone(),
            receiver: receiver.clone(),
            amount,
            fee,
            signature: common::wallet::OptionalSerializableSignature(None),
        };

        transactions.push(transaction);
    }

    // Ensure at least one valid transaction per block if not already generated
    if !valid_transaction_generated {
        let sender_index = rng.gen_range(0..miners.len());
        let receiver_index = rng.gen_range(0..miners.len());
        let sender = &miners[sender_index].address;
        let receiver = &miners[receiver_index].address;
        let amount = BigDecimal::from_str("0.001").unwrap();
        let fee = BigDecimal::from_str("0.001").unwrap();

        let transaction = Transaction {
            sender: sender.clone(),
            receiver: receiver.clone(),
            amount,
            fee,
            signature: common::wallet::OptionalSerializableSignature(None),
        };

        transactions.push(transaction);
    }

    transactions
}

fn distribute_rewards(blockchain: &Arc<Mutex<Blockchain>>, winner_address: &str, miners: &[Miner], total_reward: BigDecimal) {
    let mut blockchain = blockchain.lock().unwrap();

    // Calculate reward distribution
    let winner_reward = &total_reward * BigDecimal::from_str("0.7").unwrap();
    let remaining_reward = &total_reward * BigDecimal::from_str("0.3").unwrap();

    // Distribute rewards to the winner
    let reward_transaction = Transaction {
        sender: "0".to_string(), // Reward from the system
        receiver: winner_address.to_string(),
        amount: winner_reward,
        fee: BigDecimal::zero(),
        signature: common::wallet::OptionalSerializableSignature(None),
    };
    blockchain.create_transaction(reward_transaction);

    // Calculate total computing power
    let total_computing_power: u64 = miners.iter().map(|m| m.computing_power).sum();

    // Distribute remaining rewards proportionally to computing power
    for miner in miners {
        let miner_share = BigDecimal::from(miner.computing_power) / BigDecimal::from(total_computing_power);
        let miner_reward = &remaining_reward * miner_share;

        if !miner_reward.is_zero() {
            let reward_transaction = Transaction {
                sender: "0".to_string(),
                receiver: miner.address.clone(),
                amount: miner_reward,
                fee: BigDecimal::zero(),
                signature: common::wallet::OptionalSerializableSignature(None),
            };
            blockchain.create_transaction(reward_transaction);
        }
    }
}

fn mine_blocks(blockchain: Arc<Mutex<Blockchain>>, mut miners: Vec<Miner>, duration: Duration, rewards_wallet: String) {
    let start_time = Instant::now();
    let mut rng = rand::thread_rng();

    while start_time.elapsed() < duration {
        // Shuffle the miners list to randomize the mining order
        miners.shuffle(&mut rng);

        for miner in &miners {
            if start_time.elapsed() >= duration {
                break;
            }

            // Add transactions to the blockchain
            let balances = blockchain.lock().unwrap().balances.clone();
            let transactions = generate_transactions(&miners, &balances);
            for transaction in transactions {
                blockchain.lock().unwrap().create_transaction(transaction);
            }

            // Simulate mining based on computing power
            let mining_time = Duration::from_millis(1000 / miner.computing_power);
            thread::sleep(mining_time);

            // Print which miner is attempting to mine
            println!("Mining transactions by {}", miner.address);

            // Miner attempts to mine pending transactions
            let success = blockchain.lock().unwrap().mine_pending_transactions(miner.address.clone());

            // If mining was successful, distribute rewards
            if success {
                let total_reward = BigDecimal::from(50);
                distribute_rewards(&blockchain, &miner.address, &miners, total_reward);
            }
        }
    }

    // Print final results
    println!("Mining duration has ended. Printing final results...");
    print_final_results(&blockchain, &rewards_wallet);
}

fn print_final_results(blockchain: &Arc<Mutex<Blockchain>>, rewards_wallet: &str) {
    // Print the final blockchain status
    println!("Final Blockchain Status:");
    for block in &blockchain.lock().unwrap().blocks {
        println!("{:?}", block);
    }

    // Print final balances
    println!("Final Balances:");
    for (address, balance) in &blockchain.lock().unwrap().balances {
        println!("{}: {}", address, balance);
    }

    // Print rewards wallet balances
    println!("Rewards Wallet Balance: {}", blockchain.lock().unwrap().get_balance(rewards_wallet));
}

fn main() {
    // Initialize blockchain
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));

    // Create miners with different computing power
    let mut rng = rand::thread_rng();
    let mut miners = Vec::new();
    for i in 0..50 {
        let address = format!("Miner{}", i + 1);
        let computing_power = rng.gen_range(1..101); // Randomize computing power between 1 and 100
        miners.push(Miner { address, computing_power });
    }

    // Create rewards wallet
    let rewards_wallet = "RewardsWallet".to_string();

    // Run the mining simulation for 30 minutes
    let duration = Duration::from_secs(2);
    mine_blocks(blockchain.clone(), miners, duration, rewards_wallet.clone());

    // Ensure final results are printed even if the simulation is interrupted
    print_final_results(&blockchain, &rewards_wallet);
}