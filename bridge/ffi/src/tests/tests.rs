use crate::{bindings::*, c_api::*};

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
			schema_map: std::ptr::null_mut(),
			schema_map_len: 0,
			dsnp_versions: std::ptr::null_mut(),
			dsnp_versions_len: 0,
		};

		let environment = Environment::Dev(c_config);

		unsafe {
			let result = initialize_graph_state(&environment as *const Environment);
			assert!(result.error.is_null());
			let graph_state = result.result;
			assert_ne!(graph_state, std::ptr::null_mut());

			// Expect singleton to be initialized
			let capacity_result = get_graph_capacity(graph_state as *mut _);
			assert!(capacity_result.error.is_null());
			let capacity_1 = *capacity_result.result;
			assert_eq!(capacity_1, 100);

			let result_with_capacity =
				initialize_graph_state_with_capacity(&environment as *const Environment, 50);
			assert!(result_with_capacity.error.is_null());
			let graph_state_with_capacity = result_with_capacity.result;
			assert_ne!(graph_state_with_capacity, std::ptr::null_mut());
			let capacity_result_with_capacity =
				get_graph_capacity(graph_state_with_capacity as *mut _);
			assert!(capacity_result_with_capacity.error.is_null());
			let capacity_2 = *capacity_result_with_capacity.result;
			assert_eq!(capacity_2, 50);

			let count_before = get_graph_states_count();
			assert!(count_before.error.is_null());
			let count_before = count_before.result.as_ref().unwrap();
			free_graph_state(graph_state as *mut _);
			let count_after = get_graph_states_count();
			assert!(count_after.error.is_null());
			let count_after = count_after.result.as_ref().unwrap();
			assert_eq!(count_before - 1, *count_after);

			let count_before_with_capacity = get_graph_states_count();
			assert!(count_before_with_capacity.error.is_null());
			let count_before_with_capacity = count_before_with_capacity.result.as_ref().unwrap();
			free_graph_state(graph_state_with_capacity as *mut _);
			let count_after_with_capacity = get_graph_states_count();
			assert!(count_after_with_capacity.error.is_null());
			let count_after_with_capacity = count_after_with_capacity.result.as_ref().unwrap();
			assert_eq!(count_before_with_capacity - 1, *count_after_with_capacity);
		}
	}

	// Add more tests as needed
}
