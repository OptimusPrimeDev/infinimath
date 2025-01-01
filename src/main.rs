use common::wallet::Wallet;
use imc::blockchain::{Blockchain, Transaction};
use subchains::{SubChain, SubChainBlock};
use subchains::utils::{primex, pix};
use bigdecimal::BigDecimal;
use std::env;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use chrono::Utc;
use sha2::{Sha256, Digest};

fn print_all_blocks(subchain: &SubChain) {
    for block in &subchain.blocks {
        println!("{:?}", block);
    }
}

fn calculate_hash(block: &SubChainBlock) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{:?}{:?}{:?}{:?}{:?}", block.block_number, block.timestamp, block.result, block.prev_block_hash, block.nonce));
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Check if the required argument is provided
    if args.len() < 2 {
        eprintln!("Usage: infinimath <command> [<args>]");
        eprintln!("Commands:");
        eprintln!("  create_wallet");
        eprintln!("  load_wallet");
        eprintln!("  send_transaction <sender> <receiver> <amount> <fee>");
        eprintln!("  mine <miner_address>");
        eprintln!("  balance <address>");
        eprintln!("  is_valid");
        eprintln!("  create_subchain_block");
        eprintln!("  mine_subchain_block <difficulty>");
        eprintln!("  subchain_balance <address>");
        eprintln!("  create_pix_block");
        return;
    }

    let command = &args[1];

    // Initialize a Mutex-wrapped Blockchain to ensure safe concurrent access
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let subchain = Arc::new(Mutex::new(SubChain::new()));

    match command.as_str() {
        "create_wallet" => {
            let wallet = Wallet::new();
            wallet.save_to_file("wallet.dat").expect("Failed to save wallet");
            println!("New wallet created with address: {}", wallet.get_address());
        }
        "load_wallet" => {
            let wallet = Wallet::load_from_file("wallet.dat").expect("Failed to load wallet");
            println!("Wallet loaded with address: {}", wallet.get_address());
        }
        "send_transaction" => {
            if args.len() < 6 {
                eprintln!("Usage: send_transaction <sender> <receiver> <amount> <fee>");
                return;
            }

            let sender = &args[2];
            let receiver = &args[3];
            let amount = BigDecimal::from_str(&args[4]).expect("Invalid amount");
            let fee = BigDecimal::from_str(&args[5]).expect("Invalid fee");

            let wallet = Wallet::load_from_file("wallet.dat").expect("Failed to load wallet");

            let mut transaction = Transaction {
                sender: sender.clone(),
                receiver: receiver.clone(),
                amount,
                fee,
                signature: common::wallet::OptionalSerializableSignature(None),
            };

            transaction.sign(&wallet.private_key);

            blockchain.lock().unwrap().create_transaction(transaction);
            println!("Transaction from {} to {} created", sender, receiver);
        }
        "mine" => {
            if args.len() < 3 {
                eprintln!("Usage: mine <miner_address>");
                return;
            }

            let miner_address = args[2].clone();
            blockchain.lock().unwrap().mine_pending_transactions(miner_address);

            println!("Mining complete. Blockchain status:");
            for block in &blockchain.lock().unwrap().blocks {
                println!("{:?}", block);
            }
        }
        "balance" => {
            if args.len() < 3 {
                eprintln!("Usage: balance <address>");
                return;
            }

            let address = &args[2];
            let balance = blockchain.lock().unwrap().get_balance(address);
            println!("Balance of {}: {}", address, balance);
        }
        "is_valid" => {
            let is_valid = blockchain.lock().unwrap().is_valid();
            println!("Is blockchain valid? {}", is_valid);
        }
        "create_subchain_block" => {
            println!("Creating sub-chain block");

            {
                let mut subchain_lock = subchain.lock().unwrap();

                // Get the last prime from the previous block or initialize the first prime
                let last_prime = primex::get_last_prime_or_initialize(
                    subchain_lock.blocks.last().map(|block| block.result.as_str())
                );

                // Find the next prime number starting from the last prime
                let next_prime = primex::find_next_prime(&last_prime, 5);

                println!("New prime found: {:?}", next_prime);

                let mut subchain_block = SubChainBlock {
                    block_number: subchain_lock.blocks.len() as u64 + 1,
                    timestamp: Utc::now().timestamp() as u64,
                    result: next_prime.to_string(),
                    prev_block_hash: subchain_lock.get_last_block_hash(),
                    nonce: 0,
                    hash: String::new(),
                };

                subchain_block.hash = calculate_hash(&subchain_block);

                println!("Sub-chain block details: {:?}", subchain_block);

                subchain_lock.add_block(subchain_block);
            }

            println!("Sub-chain block successfully added.");
        }
        "mine_subchain_block" => {
            if args.len() < 3 {
                eprintln!("Usage: mine_subchain_block <difficulty>");
                return;
            }

            let difficulty = args[2].parse::<usize>().expect("Invalid difficulty");
            let mut subchain_block = subchain.lock().unwrap().get_latest_block().clone();

            subchain.lock().unwrap().mine_block(&mut subchain_block, difficulty);
            println!("Mined sub-chain block: {:?}", subchain_block);
        }
        "subchain_balance" => {
            if args.len() < 3 {
                eprintln!("Usage: subchain_balance <address>");
                return;
            }

            let address = &args[2];
            let balance = subchain.lock().unwrap().get_balance(address);
            println!("Balance of {} on sub-chain: {}", address, balance);
        }
        "create_pix_block" => {
            println!("Creating PiX block");

            for _ in 0..10 {
                let mut subchain_lock = subchain.lock().unwrap();

                // Get the last Pi value from the previous block or initialize the first value
                let last_block_number = subchain_lock.blocks.len();

                // Find the next Pi decimal value based on the current block number
                let next_pi = pix::find_next_pi(last_block_number);

                println!("New Pi decimal found: {:?}", next_pi);

                let prev_block_hash = if let Some(last_block) = subchain_lock.blocks.last() {
                    last_block.hash.clone()
                } else {
                    String::new()
                };

                let mut subchain_block = SubChainBlock {
                    block_number: subchain_lock.blocks.len() as u64,
                    timestamp: Utc::now().timestamp() as u64,
                    result: next_pi.to_string(),
                    prev_block_hash: prev_block_hash,
                    nonce: 0,
                    hash: String::new(),
                };

                subchain_block.hash = calculate_hash(&subchain_block);

                println!("PiX block details: {:?}", subchain_block);

                subchain_lock.add_block(subchain_block);
            }

            println!("PiX blocks successfully added.");

            // Print all blocks including genesis
            print_all_blocks(&subchain.lock().unwrap());
        }
        _ => {
            eprintln!("Unknown command");
        }
    }
}