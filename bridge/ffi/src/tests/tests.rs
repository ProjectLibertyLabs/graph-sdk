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
			let graph_state = initialize_graph_state(&environment as *const Environment);
			assert!(!graph_state.is_null());

			// Expect singleton to be initialized
			let capacity_1 = get_graph_capacity(graph_state);
			assert_eq!(capacity_1, 100);

			//let graph_state_with_capacity =
			//	initialize_graph_state_with_capacity(&environment as *const Environment, 100);
			//assert!(!graph_state_with_capacity.is_null());
			//let capacity_2 = get_graph_capacity(graph_state_with_capacity);
			//assert_eq!(capacity_2, 100);
			free_graph_state(graph_state);
		}
	}

	// Add more tests as needed
}
