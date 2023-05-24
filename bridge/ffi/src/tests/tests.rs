use crate::{bindings::*, c_api::*};
use std::ptr;

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_graph_state_new() {
		let c_config = Config {
			sdk_max_users_graph_size: 100,
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

			let capacity_result = get_graph_capacity(graph_state);
			assert!(capacity_result.error.is_none());
			assert!(capacity_result.result.is_some());
			let capacity = capacity_result.result.unwrap().as_ptr();
			assert_ne!(capacity, ptr::null_mut());
			assert_eq!(*capacity, 100);

			let result_with_capacity =
				initialize_graph_state_with_capacity(&environment as *const Environment, 50);
			assert!(result_with_capacity.error.is_none());
			let graph_state_with_capacity = result_with_capacity.result;
			assert!(graph_state_with_capacity.is_some());
			let graph_state_with_capacity = graph_state_with_capacity.unwrap().as_ptr();
			assert_ne!(graph_state_with_capacity, ptr::null_mut());
			let capacity_result_with_capacity = get_graph_capacity(graph_state_with_capacity);
			assert!(capacity_result_with_capacity.error.is_none());
			assert!(capacity_result_with_capacity.result.is_some());
			let capacity_with_capacity = capacity_result_with_capacity.result.unwrap().as_ptr();
			assert_ne!(capacity_with_capacity, ptr::null_mut());
			assert_eq!(*capacity_with_capacity, 50);

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

			let count_before_with_capacity = get_graph_states_count();
			assert!(count_before_with_capacity.error.is_none());
			assert!(count_before_with_capacity.result.is_some());
			let count_before_with_capacity = count_before_with_capacity.result.unwrap().as_ptr();
			assert_ne!(count_before_with_capacity, ptr::null_mut());
			let count_before_with_capacity = *count_before_with_capacity;
			assert!(count_before_with_capacity > 0);
			free_graph_state(graph_state_with_capacity);
			let count_after_with_capacity = get_graph_states_count();
			assert!(count_after_with_capacity.error.is_none());
			assert!(count_after_with_capacity.result.is_some());
			let count_after_with_capacity = count_after_with_capacity.result.unwrap().as_ptr();
			assert_ne!(count_after_with_capacity, ptr::null_mut());
			let count_after_with_capacity = *count_after_with_capacity;
			assert_eq!(count_before_with_capacity - 1, count_after_with_capacity);
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
