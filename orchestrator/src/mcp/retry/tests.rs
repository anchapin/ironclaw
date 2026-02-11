use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn test_retry_config_default() {
    let config = RetryConfig::default();
    assert_eq!(config.max_attempts, 3);
    assert_eq!(config.base_delay, Duration::from_millis(100));
    assert_eq!(config.max_delay, Duration::from_secs(5));
    assert_eq!(config.jitter, 0.1);
}

#[test]
fn test_retry_config_builder() {
    let config = RetryConfig::new()
        .max_attempts(5)
        .base_delay(Duration::from_millis(50))
        .max_delay(Duration::from_secs(10))
        .jitter(0.2);

    assert_eq!(config.max_attempts, 5);
    assert_eq!(config.base_delay, Duration::from_millis(50));
    assert_eq!(config.max_delay, Duration::from_secs(10));
    assert_eq!(config.jitter, 0.2);
}

#[test]
fn test_calculate_delay_exponential() {
    let config = RetryConfig::new()
        .base_delay(Duration::from_millis(100))
        .max_delay(Duration::from_secs(10));

    // Attempt 0: base_delay * 2^0 = 100ms
    let delay0 = config.calculate_delay(0);
    assert!(delay0 >= Duration::from_millis(90) && delay0 <= Duration::from_millis(110));

    // Attempt 1: base_delay * 2^1 = 200ms
    let delay1 = config.calculate_delay(1);
    assert!(delay1 >= Duration::from_millis(180) && delay1 <= Duration::from_millis(220));

    // Attempt 2: base_delay * 2^2 = 400ms
    let delay2 = config.calculate_delay(2);
    assert!(delay2 >= Duration::from_millis(360) && delay2 <= Duration::from_millis(440));

    // Large attempt should cap at max_delay
    let delay_large = config.calculate_delay(10);
    assert_eq!(delay_large, Duration::from_secs(10));
}

#[test]
fn test_calculate_delay_with_jitter() {
    // Run multiple times to check jitter variation
    let config = RetryConfig::new()
        .base_delay(Duration::from_millis(100))
        .jitter(0.2); // 20% jitter

    let delays: Vec<_> = (0..10).map(|_| config.calculate_delay(1)).collect();

    // With jitter, we should see variation
    let min_delay = *delays.iter().min().unwrap();
    let max_delay = *delays.iter().max().unwrap();

    // Should have some variation (at least 10ms)
    assert!(max_delay - min_delay >= Duration::from_millis(10));
}

#[test]
fn test_should_retry_status() {
    // Retryable status codes
    assert!(should_retry_status(408)); // Request Timeout
    assert!(should_retry_status(429)); // Too Many Requests
    assert!(should_retry_status(500)); // Internal Server Error
    assert!(should_retry_status(503)); // Service Unavailable

    // Non-retryable status codes
    assert!(!should_retry_status(400)); // Bad Request
    assert!(!should_retry_status(404)); // Not Found
    assert!(!should_retry_status(501)); // Not Implemented
    assert!(!should_retry_status(505)); // HTTP Version Not Supported
}

#[test]
fn test_should_retry_error() {
    let config = RetryConfig::default();

    // Network errors should be retried
    let network_err = anyhow::anyhow!("Connection refused");
    assert!(config.should_retry_error(&network_err));

    // Timeout errors should be retried
    let timeout_err = anyhow::anyhow!("Request timed out");
    assert!(config.should_retry_error(&timeout_err));

    // Auth errors should NOT be retried
    let auth_err = anyhow::anyhow!("Unauthorized");
    assert!(!config.should_retry_error(&auth_err));

    // Invalid data errors should NOT be retried
    let invalid_err = anyhow::anyhow!("Invalid JSON");
    assert!(!config.should_retry_error(&invalid_err));
}

#[tokio::test]
async fn test_retry_with_backoff_success() {
    let config = RetryConfig::default()
        .max_attempts(3)
        .base_delay(Duration::from_millis(10));

    let attempt = Arc::new(AtomicUsize::new(0));
    let attempt_clone = Arc::clone(&attempt);

    let result: Result<i32> = retry_with_backoff(&config, move || {
        let attempt = Arc::clone(&attempt_clone);
        async move {
            let current = attempt.fetch_add(1, Ordering::SeqCst);
            if current < 1 {
                Err(anyhow::anyhow!("Temporary failure"))
            } else {
                Ok(42)
            }
        }
    })
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[tokio::test]
async fn test_retry_with_backoff_failure() {
    let config = RetryConfig::default()
        .max_attempts(2)
        .base_delay(Duration::from_millis(10));

    let result: Result<i32> = retry_with_backoff(&config, || async {
        Err(anyhow::anyhow!("Permanent failure"))
    })
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_retry_with_backoff_no_retry_on_permanent_error() {
    let config = RetryConfig::default();

    let attempt_count = Arc::new(AtomicUsize::new(0));
    let attempt_clone = Arc::clone(&attempt_count);

    let result: Result<i32> = retry_with_backoff(&config, move || {
        let attempt = Arc::clone(&attempt_clone);
        async move {
            attempt.fetch_add(1, Ordering::SeqCst);
            Err(anyhow::anyhow!("Unauthorized"))
        }
    })
    .await;

    // Should fail immediately without retries
    assert!(result.is_err());
    assert_eq!(attempt_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_retry_with_backoff_max_attempts() {
    let config = RetryConfig::default()
        .max_attempts(2)
        .base_delay(Duration::from_millis(10));

    let attempt_count = Arc::new(AtomicUsize::new(0));
    let attempt_clone = Arc::clone(&attempt_count);

    let result: Result<i32> = retry_with_backoff(&config, move || {
        let attempt = Arc::clone(&attempt_clone);
        async move {
            attempt.fetch_add(1, Ordering::SeqCst);
            Err(anyhow::anyhow!("Connection timeout"))
        }
    })
    .await;

    assert!(result.is_err());
    assert_eq!(attempt_count.load(Ordering::SeqCst), 2); // Should retry once
}

#[test]
fn test_jitter_clamping() {
    let config = RetryConfig::new();
    assert_eq!(config.jitter, 0.1);

    // Test clamping - jitter builder method consumes self, so we test each case separately
    let config_too_high = RetryConfig::new().jitter(1.5);
    assert_eq!(config_too_high.jitter, 1.0);

    let config_negative = RetryConfig::new().jitter(-0.5);
    assert_eq!(config_negative.jitter, 0.0);
}
