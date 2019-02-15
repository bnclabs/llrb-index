// Performance measurement for Llrb instances. Measure:
// * Initial load, 1M and 10M and 100M
// * Set operation, overwrite 1M data set, 10M and 100M data set
// * Delete operation 10M -> 1M and 100M -> 10M
// * Get operation all 1M, all 10M, all 100M
//
// For each data set measure latency and throughput
// * For every 10th operation measure the latency.
// * Compute the total time taken to complete all the operation for set.
// * Tally, latency * num_op = throughtput
//
// Repeat the above process for:
// * {key=i64, value=i64}
// * {key=String(32), value=String(256)}
//
// Precreate the set before applying the operation.

fn main() {
    unreachable!()
}
