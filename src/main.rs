use common::wallet::Wallet;
use imc::blockchain::{Blockchain, Transaction};
use bigdecimal::BigDecimal;
use std::env;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

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
        return;
    }

    let command = &args[1];

    // Initialize a Mutex-wrapped Blockchain to ensure safe concurrent access
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));

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
        _ => {
            eprintln!("Unknown command");
        }
    }
}