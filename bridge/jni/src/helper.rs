use crate::{
	api::SdkJniResult,
	errors::{throw_exception, SdkJniError},
};
use jni::{
	objects::JByteArray,
	sys::{jboolean, jint, jlong, JNI_FALSE},
	JNIEnv,
};
use std::sync::RwLock;

#[inline(always)]
pub fn handle_result<R, E>(env: &mut JNIEnv, result: Result<SdkJniResult<R>, E>) -> R
where
	R: JniReturnValue,
	E: std::any::Any + Send + 'static,
{
	match result {
		Ok(Ok(r)) => r,
		Ok(Err(e)) => {
			throw_exception(env, e);
			R::default_value()
		},
		Err(r) => {
			throw_exception(env, SdkJniError::UnexpectedPanic(Box::new(r)));
			R::default_value()
		},
	}
}

#[inline(always)]
pub fn validate_handle(states: &RwLock<Vec<jlong>>, handle: jlong) -> SdkJniResult<()> {
	if handle == 0 {
		return Err(SdkJniError::InvalidHandle("is null"))
	}
	let graph_states = states.read().map_err(|_| SdkJniError::LockError)?;
	if !graph_states.contains(&handle) {
		return Err(SdkJniError::InvalidHandle("does not exist"))
	}
	Ok(())
}

/// Provides a return value when an exception is thrown.
pub trait JniReturnValue {
	fn default_value() -> Self;
}

impl JniReturnValue for jlong {
	fn default_value() -> Self {
		0
	}
}

impl JniReturnValue for jint {
	fn default_value() -> Self {
		0
	}
}

impl JniReturnValue for jboolean {
	fn default_value() -> Self {
		JNI_FALSE
	}
}

impl JniReturnValue for () {
	fn default_value() -> Self {}
}

impl JniReturnValue for JByteArray<'_> {
	fn default_value() -> Self {
		JByteArray::default()
	}
}
