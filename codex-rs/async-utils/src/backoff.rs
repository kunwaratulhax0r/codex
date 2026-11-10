//! Shared exponential retry delays with jitter.

use std::time::Duration;

use rand::Rng;

const INITIAL_DELAY_MS: u64 = 200;
const BACKOFF_FACTOR: f64 = 2.0;
/// Upper bound on the computed delay. Without this, doubling without a cap
/// reaches `f64::INFINITY` by attempt ~1025 (and already reaches multi-year
/// delays by attempt ~20), which saturates the final `u64` cast to its max
/// value and makes the caller sleep for hundreds of millions of years
/// instead of retrying or giving up.
const MAX_DELAY_MS: u64 = 60_000;
/// Exponent cap. `2^32` already dwarfs `MAX_DELAY_MS`, so this only guards
/// the `u64 -> i32` cast below from wrapping on an attempt count far beyond
/// any caller's actual retry budget.
const MAX_BACKOFF_EXPONENT: u64 = 32;

/// Return a retry delay starting at 200 ms and doubling on each subsequent attempt,
/// capped at 60 seconds, with up to 10% jitter. Attempts zero and one both use the
/// initial delay.
pub fn backoff(attempt: u64) -> Duration {
    let exponent = attempt.saturating_sub(1).min(MAX_BACKOFF_EXPONENT) as i32;
    let exp = BACKOFF_FACTOR.powi(exponent);
    let base_ms = (INITIAL_DELAY_MS as f64 * exp).min(MAX_DELAY_MS as f64);
    let jitter = rand::rng().random_range(0.9..1.1);
    let delay_ms = (base_ms * jitter).min(MAX_DELAY_MS as f64);
    Duration::from_millis(delay_ms as u64)
}

#[cfg(test)]
#[path = "backoff_tests.rs"]
mod tests;
