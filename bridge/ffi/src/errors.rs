use dsnp_graph_config::errors::DsnpGraphError;

// Opaque handle for DsnpGraphError
pub struct DsnpGraphErrorHandle {
	// to make errors opaque this field must be private
	// it enables to create a pointer to the error
	error: *mut DsnpGraphError,
}

impl DsnpGraphErrorHandle {
	pub fn from_error(error: DsnpGraphError) -> *mut Self {
		// allocate memory for the error
		let mut error_handle = Box::new(Self { error: std::ptr::null_mut() });

		// allocate memory for the error
		error_handle.error = Box::into_raw(Box::new(error));

		// return a pointer to the error
		Box::into_raw(error_handle)
	}

	pub fn error_code(&self) -> i32 {
		unsafe { (*self.error).error_code() }
	}

	pub fn error_message(&self) -> String {
		unsafe { (*self.error).to_string() }
	}
}
