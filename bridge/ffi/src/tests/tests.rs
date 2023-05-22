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
			let result = initialize_graph_state(&environment as *const Environment);
			assert_eq!(result.error.as_ptr(), ptr::null_mut());
			let graph_state = result.result;
			assert_ne!(graph_state.as_ptr(), ptr::null_mut());

			// Expect singleton to be initialized
			let capacity_result = get_graph_capacity(graph_state.as_ptr());
			assert_eq!(capacity_result.error.as_ptr(), ptr::null_mut());
			let capacity_1 = *capacity_result.result.as_ref();
			assert_eq!(capacity_1, 100);

			let result_with_capacity =
				initialize_graph_state_with_capacity(&environment as *const Environment, 50);
			assert_eq!(result_with_capacity.error.as_ptr(), ptr::null_mut());
			let graph_state_with_capacity = result_with_capacity.result;
			assert_ne!(graph_state_with_capacity.as_ptr(), ptr::null_mut());
			let capacity_result_with_capacity =
				get_graph_capacity(graph_state_with_capacity.as_ptr());
			assert_eq!(capacity_result_with_capacity.error.as_ptr(), ptr::null_mut());
			let capacity_2 = *capacity_result_with_capacity.result.as_ref();
			assert_eq!(capacity_2, 50);

			let count_before = get_graph_states_count();
			assert_eq!(count_before.error.as_ptr(), ptr::null_mut());
			let count_before = *count_before.result.as_ref();
			free_graph_state(graph_state.as_ptr());
			let count_after = get_graph_states_count();
			assert_eq!(count_after.error.as_ptr(), ptr::null_mut());
			let count_after = *count_after.result.as_ref();
			assert_eq!(count_before - 1, count_after);

			let count_before_with_capacity = get_graph_states_count();
			assert_eq!(count_before_with_capacity.error.as_ptr(), ptr::null_mut());
			let count_before_with_capacity = *count_before_with_capacity.result.as_ref();
			free_graph_state(graph_state_with_capacity.as_ptr());
			let count_after_with_capacity = get_graph_states_count();
			assert_eq!(count_after_with_capacity.error.as_ptr(), ptr::null_mut());
			let count_after_with_capacity = *count_after_with_capacity.result.as_ref();
			assert_eq!(count_before_with_capacity - 1, count_after_with_capacity);
		}
	}

	// Add more tests as needed
}
