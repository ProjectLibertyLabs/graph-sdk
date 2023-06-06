//! Module that defines helpers to create or read timestamps
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const SECS_PER_DAY: u64 = 60 * 60 * 24;

/// Calculates current timestamp from EPOCH in seconds, rounded to the nearest 1000
pub fn time_in_ksecs() -> u64 {
	SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() / 1_000
}

/// Calculates duration in days between now and provided timestamp from EPOCH
pub fn duration_days_since(since_ksecs: u64) -> u64 {
	let from_sec = since_ksecs.saturating_mul(1_000);
	let to_sec = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
	duration_sec_from_to(from_sec, to_sec) / SECS_PER_DAY
}

fn duration_sec_from_to(from_sec: u64, to_sec: u64) -> u64 {
	let from = Duration::from_secs(from_sec);
	let to = Duration::from_secs(to_sec);
	to.saturating_sub(from).as_secs()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn duration_days_since_should_return_correct_number_of_days() {
		// arrange
		let days = 90;
		let past = SystemTime::now().checked_sub(Duration::from_secs(days * SECS_PER_DAY)).unwrap();
		let past_ksec = past.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() / 1_000;

		// act
		let duration_days = duration_days_since(past_ksec);

		// assert
		assert_eq!(duration_days, days);
	}
}
