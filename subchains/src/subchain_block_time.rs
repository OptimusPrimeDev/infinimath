pub fn calculate_block_time(miners: u64) -> f64 {
    let mut block_time = 10.0; // start at 10 seconds
    if miners > 50 {
        let reduction_factor = (miners - 50) as f64 * 0.01;
        block_time *= 1.0 - reduction_factor;
    }
    block_time.max(0.5) // minimum block time of 0.5 seconds
}