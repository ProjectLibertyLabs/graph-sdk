use crate::bindings::*;
use libc::{c_int, size_t};

// Define a C-compatible representation of GraphAPI
#[repr(C)]
pub struct GraphAPI {
	// Use raw pointers instead of references, since references are not C-compatible
	contains_user_graph: extern "C" fn(*const u64) -> bool,
	len: extern "C" fn() -> usize,
	remove_user_graph: extern "C" fn(*const u64),
	import_users_data: extern "C" fn(*const ImportBundle, size_t) -> c_int,
	export_updates: extern "C" fn(*mut *mut Update, *mut size_t) -> c_int,
	apply_actions: extern "C" fn(*const Action, size_t) -> c_int,
	get_connections_for_user_graph:
		extern "C" fn(*const u64, *const u16, bool, *mut *mut DsnpGraphEdge, *mut size_t) -> c_int,
	get_connections_without_keys: extern "C" fn(*mut *mut u64, *mut size_t) -> c_int,
}
