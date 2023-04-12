use std::time::{SystemTime, UNIX_EPOCH};

/// Calculate a timestamp in seconds, rounded to the nearest 1000
pub fn time_in_ksecs() -> u64 {
	SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() / 1_000
}
