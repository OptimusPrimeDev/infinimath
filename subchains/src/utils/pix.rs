use num_bigint::BigInt;

const PI_DIGITS: &str = "141592653589793238462643383279502884197";

pub fn initialize_genesis_block() -> BigInt {
    // Set the value of the genesis block to "1" (first digit after the decimal point)
    BigInt::from(1)
}

pub fn get_last_pi_or_initialize(last_block_result: Option<&str>) -> BigInt {
    match last_block_result {
        Some(result) => result.parse::<BigInt>().unwrap_or_else(|_| initialize_genesis_block()),
        None => initialize_genesis_block(),
    }
}

pub fn find_next_pi(last_block_number: usize) -> BigInt {
    // Find the next digit of Pi based on the last block number
    let next_digit = PI_DIGITS.chars().nth(last_block_number).unwrap().to_digit(10).unwrap();
    BigInt::from(next_digit)
}