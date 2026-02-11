//! Retry Logic and Error Resilience
//!
//! This module provides retry mechanisms with exponential backoff and jitter
//! for handling transient failures in MCP communication.
//!
//! # Features
//!
//! - **Exponential Backoff**: Retry delay grows exponentially with each attempt
//! - **Jitter**: Random delay variation to prevent thundering herd
//! - **Configurable**: Max attempts, base delay, max delay, jitter factor
//! - **Smart Retry**: Only retry on transient/intermittent errors
//!
//! # Example
//!
//! ```ignore
//! use ironclaw_orchestrator::mcp::retry::{RetryConfig, retry_with_backoff};
//!
//! let config = RetryConfig::default()
//!     .max_attempts(3)
//!     .base_delay(Duration::from_millis(100))
//!     .max_delay(Duration::from_secs(5));
//!
//! let result = retry_with_backoff(&config, || async {
//!     // Operation that might fail transiently
//!     Ok(42)
//! }).await?;
//! ```

use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

/// Retry configuration
///
/// Controls the retry behavior for transient failures.
///
/// # Fields
///
/// * `max_attempts` - Maximum number of retry attempts (default: 3)
/// * `base_delay` - Initial delay before first retry (default: 100ms)
/// * `max_delay` - Maximum delay between retries (default: 5s)
/// * `jitter` - Random delay variation factor (default: 0.1 = 10%)
///
/// # Example
///
/// ```ignore
/// let config = RetryConfig::default()
///     .max_attempts(5)
///     .base_delay(Duration::from_millis(50))
///     .max_delay(Duration::from_secs(10));
/// ```
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (including initial attempt)
    pub max_attempts: usize,

    /// Base delay before first retry
    pub base_delay: Duration,

    /// Maximum delay between retries
    pub max_delay: Duration,

    /// Jitter factor (0.0 to 1.0) - adds random variation to delays
    /// This prevents thundering herd when multiple clients retry simultaneously
    pub jitter: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            jitter: 0.1,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of retry attempts
    ///
    /// # Arguments
    ///
    /// * `attempts` - Maximum retry attempts (including first attempt)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = RetryConfig::new().max_attempts(5);
    /// ```
    pub fn max_attempts(mut self, attempts: usize) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Set the base delay between retries
    ///
    /// # Arguments
    ///
    /// * `delay` - Base delay duration
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = RetryConfig::new().base_delay(Duration::from_millis(50));
    /// ```
    pub fn base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    /// Set the maximum delay between retries
    ///
    /// # Arguments
    ///
    /// * `delay` - Maximum delay duration
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = RetryConfig::new().max_delay(Duration::from_secs(10));
    /// ```
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set the jitter factor
    ///
    /// Jitter adds random variation to delays to prevent synchronization
    /// when multiple clients retry simultaneously.
    ///
    /// # Arguments
    ///
    /// * `jitter` - Jitter factor between 0.0 (no jitter) and 1.0 (100% jitter)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = RetryConfig::new().jitter(0.2); // 20% jitter
    /// ```
    pub fn jitter(mut self, jitter: f64) -> Self {
        self.jitter = jitter.clamp(0.0, 1.0);
        self
    }

    /// Calculate delay for a given retry attempt
    ///
    /// Uses exponential backoff: delay = base_delay * 2^(attempt-1)
    /// Then applies jitter and caps at max_delay.
    pub fn calculate_delay(&self, attempt: usize) -> Duration {
        let exponential_delay = self.base_delay * 2_u32.pow(attempt as u32);

        // Apply jitter: random variation +/- jitter/2
        let jitter_range = exponential_delay.mul_f64(self.jitter);
        let jitter_offset = (rand::random::<f64>() - 0.5) * 2.0 * jitter_range.as_secs_f64();
        let jittered_delay =
            exponential_delay.saturating_add(Duration::from_secs_f64(jitter_offset.abs()));

        // Cap at max delay
        jittered_delay.min(self.max_delay)
    }

    /// Check if an error should be retried
    ///
    /// Transient errors that should be retried:
    /// - Network errors (connection refused, timeout, etc.)
    /// - HTTP 5xx server errors (with exceptions)
    /// - Temporary failures
    ///
    /// Non-retryable errors:
    /// - HTTP 4xx client errors (except 408 Request Timeout, 429 Too Many Requests)
    /// - Authentication failures
    /// - Invalid data/format errors
    pub fn should_retry_error(&self, error: &anyhow::Error) -> bool {
        let error_msg = error.to_string().to_lowercase();

        // Don't retry authentication errors
        if error_msg.contains("unauthorized") || error_msg.contains("forbidden") {
            return false;
        }

        // Don't retry invalid request errors
        if error_msg.contains("invalid") && !error_msg.contains("timeout") {
            return false;
        }

        // Retry network errors
        if error_msg.contains("connection")
            || error_msg.contains("timeout")
            || error_msg.contains("timed out")
            || error_msg.contains("network")
            || error_msg.contains("dns")
            || error_msg.contains("temporary")
        {
            return true;
        }

        // Default: don't retry unknown errors
        false
    }
}

/// Retry an operation with exponential backoff
///
/// This function will attempt the operation up to `max_attempts` times,
/// with exponential backoff and jitter between attempts.
///
/// Only transient errors (as determined by `should_retry_error`) will
/// trigger a retry. Permanent errors will fail immediately.
///
/// # Arguments
///
/// * `config` - Retry configuration
/// * `operation` - Async operation to retry
///
/// # Returns
///
/// Returns the operation's result on success, or the last error on failure.
///
/// # Example
///
/// ```ignore
/// let result = retry_with_backoff(&config, || async {
///     // Some fallible operation
///     fetch_data().await
/// }).await?;
/// ```
pub async fn retry_with_backoff<F, T, Fut>(config: &RetryConfig, mut operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error = None;

    for attempt in 0..config.max_attempts {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    tracing::info!(
                        "Operation succeeded on attempt {} after {} retries",
                        attempt + 1,
                        attempt
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                // Check if this error should be retried
                if attempt < config.max_attempts - 1 && config.should_retry_error(&e) {
                    tracing::warn!(
                        "Attempt {} failed: {}, retrying after delay",
                        attempt + 1,
                        e
                    );

                    let delay = config.calculate_delay(attempt);
                    tracing::debug!("Waiting {:?} before retry", delay);
                    sleep(delay).await;

                    last_error = Some(e);
                } else {
                    // Don't retry this error
                    tracing::error!("Operation failed after {} attempts: {}", attempt + 1, e);
                    return Err(e);
                }
            }
        }
    }

    // Should not reach here, but handle it gracefully
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
}

/// Check if an HTTP status code should be retried
///
/// Retryable status codes:
/// - 408 Request Timeout
/// - 429 Too Many Requests
/// - 5xx Server Error (except 501 Not Implemented, 505 HTTP Version Not Supported)
///
/// Non-retryable status codes:
/// - 4xx Client Errors (except 408, 429)
/// - 501 Not Implemented
/// - 505 HTTP Version Not Supported
pub fn should_retry_status(status: u16) -> bool {
    match status {
        // 408 Request Timeout - retry
        408 => true,
        // 429 Too Many Requests - retry (rate limit)
        429 => true,
        // 5xx server errors - retry with exceptions
        500..=599 => {
            // Don't retry 501 Not Implemented or 505 HTTP Version Not Supported
            status != 501 && status != 505
        }
        // All other status codes - don't retry
        _ => false,
    }
}

#[cfg(test)]
mod tests;
