use super::MAX_DELAY_MS;
use super::backoff;
use std::time::Duration;

#[test]
fn early_attempts_stay_small_and_grow() {
    // Attempt 0 and 1 both use the initial delay (200ms +/- 10% jitter).
    for attempt in [0, 1] {
        let delay = backoff(attempt);
        assert!(
            delay >= Duration::from_millis(180) && delay <= Duration::from_millis(220),
            "attempt {attempt}: unexpected delay {delay:?}"
        );
    }
    // Attempt 2 should be roughly double attempt 1 (within jitter bounds).
    let delay_at_2 = backoff(2);
    assert!(
        delay_at_2 >= Duration::from_millis(360) && delay_at_2 <= Duration::from_millis(440),
        "attempt 2: unexpected delay {delay_at_2:?}"
    );
}

#[test]
fn large_attempt_counts_saturate_at_max_delay_instead_of_overflowing() {
    // Before the fix, attempt=30 alone already computed a multi-year delay
    // (`2.0_f64.powi(29) * 200ms` ~= 3.4 years), and by ~attempt 58 the
    // exponent overflowed f64 to infinity, saturating the final `as u64`
    // cast to `u64::MAX` milliseconds (~584 million years) instead of a
    // sane bounded delay.
    for attempt in [30, 58, 100, 1_000, 1_000_000, u64::MAX] {
        let delay = backoff(attempt);
        assert!(
            delay <= Duration::from_millis(MAX_DELAY_MS),
            "attempt {attempt}: delay {delay:?} exceeds the {MAX_DELAY_MS}ms cap"
        );
        // Never silently collapses to zero either -- a capped delay should
        // still meaningfully throttle the retry loop.
        assert!(
            delay >= Duration::from_millis(MAX_DELAY_MS * 9 / 10),
            "attempt {attempt}: delay {delay:?} is suspiciously far below the cap"
        );
    }
}

#[test]
fn delay_never_exceeds_cap_across_a_full_attempt_sweep() {
    for attempt in 0..200 {
        let delay = backoff(attempt);
        assert!(
            delay <= Duration::from_millis(MAX_DELAY_MS),
            "attempt {attempt}: delay {delay:?} exceeds the {MAX_DELAY_MS}ms cap"
        );
    }
}
