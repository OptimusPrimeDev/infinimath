use num_bigint::{BigUint, RandBigInt};
use num_traits::{One, Zero};
use rand::thread_rng;

/// Determines if the given number is a prime using the Miller-Rabin algorithm.
pub fn is_prime(n: &BigUint, k: u32) -> bool {
    if *n <= BigUint::one() {
        return false;
    }
    if *n <= BigUint::from(3u32) {
        return true;
    }
    if n % BigUint::from(2u32) == BigUint::zero() {
        return false;
    }

    // Write (n - 1) as 2^r * d
    let mut r = 0;
    let mut d = n - BigUint::one();
    while &d % BigUint::from(2u32) == BigUint::zero() {
        d /= BigUint::from(2u32);
        r += 1;
    }

    'outer: for _ in 0..k {
        let a = thread_rng().gen_biguint_below(n);
        let mut x = mod_exp(&a, &d, n);
        if x == BigUint::one() || x == n - BigUint::one() {
            continue;
        }
        for _ in 0..r - 1 {
            x = mod_exp(&x, &BigUint::from(2u32), n);
            if x == n - BigUint::one() {
                continue 'outer;
            }
        }
        return false;
    }
    true
}

/// Performs modular exponentiation. It returns (base^exp) % modulus.
fn mod_exp(base: &BigUint, exp: &BigUint, modulus: &BigUint) -> BigUint {
    let mut result = BigUint::one();
    let mut base = base % modulus;
    let mut exp = exp.clone();
    while exp > BigUint::zero() {
        if &exp % BigUint::from(2u32) == BigUint::one() {
            result = (result * &base) % modulus;
        }
        exp >>= 1;
        base = (&base * &base) % modulus;
    }
    result
}

/// Finds the next prime number starting from the given start number.
pub fn find_next_prime(start: &BigUint, k: u32) -> BigUint {
    let mut candidate = start + BigUint::one();
    while !is_prime(&candidate, k) {
        candidate += BigUint::one();
    }
    candidate
}

/// Initializes the genesis block with the prime number "2".
pub fn initialize_genesis_block() -> BigUint {
    BigUint::from(2u32)
}

/// Retrieves the prime number from the last block or initializes the first prime after genesis.
pub fn get_last_prime_or_initialize(last_block_result: Option<&str>) -> BigUint {
    match last_block_result {
        Some(result) => result.parse::<BigUint>().unwrap_or_else(|_| initialize_genesis_block()),
        None => initialize_genesis_block(),
    }
}