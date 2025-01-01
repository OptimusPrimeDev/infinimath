use std::collections::HashMap;
use imc::blockchain::Blockchain;

fn main() {
    let mut blockchain = Blockchain::new();

    // Create a new smart contract
    let create_result = blockchain.create_smart_contract("contract1".to_string(), "Alice".to_string(), "code".to_string());
    println!("{:?}", create_result);

    // Execute a smart contract function to set a state value
    let mut params = HashMap::new();
    params.insert("key".to_string(), "name".to_string());
    params.insert("value".to_string(), "Alice".to_string());

    let execute_result = blockchain.execute_smart_contract("contract1", "set", params.clone());
    println!("{:?}", execute_result);

    // Retrieve the state value
    let mut get_params = HashMap::new();
    get_params.insert("key".to_string(), "name".to_string());

    let get_result = blockchain.execute_smart_contract("contract1", "get", get_params);
    println!("{:?}", get_result);
}