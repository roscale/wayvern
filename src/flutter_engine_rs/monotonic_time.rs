use nix::time::{ClockId, clock_gettime};

pub fn monotonic_time() -> u64 {
    let ts = clock_gettime(ClockId::CLOCK_MONOTONIC).unwrap();
    ts.tv_sec() as u64 * 1_000_000_000 + ts.tv_nsec() as u64
}
