use crate::{bindings::*, c_api::*};
use std::ptr;

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_graph_state_new() {
		let c_config = Config {
			sdk_max_stale_friendship_days: 90,
			max_graph_page_size_bytes: 1024,
			max_page_id: 10,
			max_key_page_size_bytes: 1024,
			schema_map: ptr::null_mut(),
			schema_map_len: 0,
			dsnp_versions: ptr::null_mut(),
			dsnp_versions_len: 0,
		};

		let environment = Environment::Dev(c_config);

		unsafe {
			// just to make sure we can call it multiple times
			let _result1 = initialize_graph_state(&environment as *const Environment);
			let _result2 = initialize_graph_state(&environment as *const Environment);

			let result = initialize_graph_state(&environment as *const Environment);
			assert!(result.error.is_none());
			let graph_state = result.result;
			assert!(graph_state.is_some());
			let graph_state = graph_state.unwrap().as_ptr();
			assert_ne!(graph_state, ptr::null_mut());

			let count_before = get_graph_states_count();
			assert!(count_before.error.is_none());
			assert!(count_before.result.is_some());
			let count_before = count_before.result.unwrap().as_ptr();
			assert_ne!(count_before, ptr::null_mut());
			let count_before = *count_before;
			free_graph_state(graph_state);
			let count_after = get_graph_states_count();
			assert!(count_after.error.is_none());
			assert!(count_after.result.is_some());
			let count_after = count_after.result.unwrap().as_ptr();
			assert_ne!(count_after, ptr::null_mut());
			let count_after = *count_after;
			assert_eq!(count_before - 1, count_after);

			free_graph_states();
			let count_after_free = get_graph_states_count();
			assert!(count_after_free.error.is_none());
			assert!(count_after_free.result.is_some());
			let count_after_free = count_after_free.result.unwrap().as_ptr();
			assert_ne!(count_after_free, ptr::null_mut());
			let count_after_free = *count_after_free;
			assert_eq!(0, count_after_free);
		}
	}

	// Add more tests as needed
}
