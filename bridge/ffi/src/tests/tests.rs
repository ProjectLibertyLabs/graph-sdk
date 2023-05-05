use crate::{bindings::*, c_api::*};

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_graph_state_new() {
		let environment = Environment {
			environment_type: EnvironmentType::Dev,
			config: Config {
				sdk_max_users_graph_size: 100,
				sdk_max_stale_friendship_days: 90,
				max_graph_page_size_bytes: 1024,
				max_page_id: 10,
				max_key_page_size_bytes: 1024,
				schema_map: std::ptr::null_mut(),
				schema_map_len: 0,
				dsnp_versions: std::ptr::null_mut(),
				dsnp_versions_len: 0,
			},
		};

		let graph_state = unsafe { graph_state_new(&environment as *const Environment) };

		assert!(!graph_state.is_null());

		unsafe { graph_state_free(graph_state) };
	}

	// Add more tests as needed
}
