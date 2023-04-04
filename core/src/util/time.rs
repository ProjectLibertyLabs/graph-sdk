use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const SECONDS_PER_DAY: u64 = 86_400;

pub trait AsDays {
	fn as_days(&self) -> u32;
}

impl AsDays for Duration {
	fn as_days(&self) -> u32 {
		(self.as_secs() / SECONDS_PER_DAY) as u32
	}
}

/// Calculate a timestamp in seconds, rounded to the nearest 1000
pub fn time_in_ksecs() -> u64 {
	SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() / 1_000
}

#[allow(dead_code)]
/// Calculate number of days since the Unix Epoch
pub fn time_in_days() -> u32 {
	SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_days()
}
