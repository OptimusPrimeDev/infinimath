use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use imc::blockchain::{Blockchain, Block};
use rand::Rng;
use std::fs::File;
use std::io::Write;

#[derive(Clone)]
struct Miner {
    id: String,
    computing_power: u64, // Number of hashes per second
    total_imc: f64,       // Total IMC earned
    blocks_mined: u64,    // Total blocks mined
}

impl Miner {
    fn new(id: &str, computing_power: u64) -> Self {
        Miner {
            id: id.to_string(),
            computing_power,
            total_imc: 0.0,
            blocks_mined: 0,
        }
    }

    fn mine(&mut self, blockchain: Arc<Mutex<Blockchain>>, duration: Duration, miners: Arc<Mutex<Vec<Miner>>>) {
        let start_time = Instant::now();
        while start_time.elapsed() < duration {
            let mut blockchain = blockchain.lock().unwrap();
            let previous_block = blockchain.blocks.last().unwrap();
            let mut block = Block::new(
                previous_block.index + 1,
                start_time.elapsed().as_millis() as u128,
                previous_block.hash.clone(),
                format!("Miner {}'s block", self.id),
                0,
            );

            // Simulate mining with computing power
            for _ in 0..self.computing_power {
                block.nonce += 1;
                block.hash = block.calculate_hash();
                if &block.hash[..blockchain.difficulty] == &"0".repeat(blockchain.difficulty) {
                    println!("Miner {} mined a block!", self.id);
                    blockchain.blocks.push(block);
                    self.blocks_mined += 1;
                    self.distribute_rewards(miners.clone());
                    return; // Exit the mining loop after finding a valid block
                }
            }
        }
    }

    fn distribute_rewards(&self, miners: Arc<Mutex<Vec<Miner>>>) {
        let reward = 40.0;
        let winner_share = reward * 0.3;
        let distributed_share = reward * 0.7;

        // Calculate total computing power for distribution
        let total_computing_power: u64 = {
            let miners = miners.lock().unwrap();
            miners.iter().map(|m| m.computing_power).sum()
        };

        // Distribute winner's share
        {
            let mut miners = miners.lock().unwrap();
            for miner in miners.iter_mut() {
                if miner.id == self.id {
                    miner.total_imc += winner_share;
                    break;
                }
            }
        }

        // Distribute the remaining share based on computing power
        {
            let mut miners = miners.lock().unwrap();
            for miner in miners.iter_mut() {
                let miner_share = distributed_share * (miner.computing_power as f64 / total_computing_power as f64);
                miner.total_imc += miner_share;
            }
        }
    }
}

fn main() {
    let blockchain = Arc::new(Mutex::new({
        let mut bc = Blockchain::new();
        bc.difficulty = 5; // Increase difficulty to make mining more challenging
        bc
    }));
    let mut rng = rand::thread_rng(); // Create a random number generator

    // Create 1000 miners with random computing power
    let miners = Arc::new(Mutex::new(
        (1..=1000)
            .map(|i| Miner::new(&format!("Miner{}", i), rng.gen_range(100..=500))) // Random computing power between 100 and 500
            .collect::<Vec<_>>(),
    ));

    let duration = Duration::new(30 * 60, 0); // 30 minutes

    // Create threads for each miner
    let mut handles = vec![];
    for i in 0..1000 {
        let blockchain = Arc::clone(&blockchain);
        let miners = Arc::clone(&miners);
        let handle = thread::spawn(move || {
            let mut miner = miners.lock().unwrap()[i].clone();
            miner.mine(blockchain, duration, miners);
        });
        handles.push(handle);
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // Print each miner's total IMC, computing power, and blocks mined
    let miners = miners.lock().unwrap();
    for miner in miners.iter() {
        println!("{} earned {:.2} IMC with computing power of {} and mined {} blocks", miner.id, miner.total_imc, miner.computing_power, miner.blocks_mined);
    }

    // Save results to a CSV file
    let mut file = File::create("miners_results.csv").unwrap();
    writeln!(file, "Miner ID,Total IMC,Computing Power,Blocks Mined").unwrap();
    for miner in miners.iter() {
        writeln!(file, "{},{:.2},{},{}", miner.id, miner.total_imc, miner.computing_power, miner.blocks_mined).unwrap();
    }

    // Print final blockchain status
    let blockchain = blockchain.lock().unwrap();
    println!("Final Blockchain Status:");
    for block in &blockchain.blocks {
        println!("{:?}", block);
    }
}