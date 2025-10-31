pub trait IsRetryable {
    fn is_retryable(&self) -> bool;
}