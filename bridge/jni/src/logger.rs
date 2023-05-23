use jni::{
	objects::{GlobalRef, JClass, JObject, JValue},
	sys::jint,
	JNIEnv, JavaVM,
};
use log;
use std::{
	panic::{catch_unwind, AssertUnwindSafe},
	process::abort,
};

#[derive(Clone, Copy, Debug)]
enum SLF4JLogLevel {
	Trace = 0,
	Debug = 10,
	Info = 20,
	Warn = 30,
	Error = 40,
}

impl From<log::Level> for SLF4JLogLevel {
	fn from(level: log::Level) -> Self {
		use log::Level::*;
		match level {
			Error => Self::Error,
			Warn => Self::Warn,
			Info => Self::Info,
			Debug => Self::Debug,
			Trace => Self::Trace,
		}
	}
}

impl From<SLF4JLogLevel> for log::Level {
	fn from(level: SLF4JLogLevel) -> Self {
		use SLF4JLogLevel::*;
		match level {
			Error => Self::Error,
			Warn => Self::Warn,
			Info => Self::Info,
			Debug => Self::Debug,
			Trace => Self::Trace,
		}
	}
}

impl TryFrom<jint> for SLF4JLogLevel {
	type Error = ();

	fn try_from(level: jint) -> Result<Self, ()> {
		match level {
			0 => Ok(Self::Trace),
			10 => Ok(Self::Debug),
			20 => Ok(Self::Info),
			30 => Ok(Self::Warn),
			40 => Ok(Self::Error),
			_ => Err(()),
		}
	}
}

impl From<SLF4JLogLevel> for &str {
	fn from(level: SLF4JLogLevel) -> Self {
		use SLF4JLogLevel::*;
		match level {
			Trace => "trace",
			Debug => "debug",
			Info => "info",
			Warn => "warn",
			Error => "error",
		}
	}
}

struct SLF4JLogger {
	vm: JavaVM,
	logger_class: GlobalRef,
}

impl SLF4JLogger {
	fn new(env: JNIEnv, logger_class: JClass) -> jni::errors::Result<Self> {
		Ok(Self { vm: env.get_java_vm()?, logger_class: env.new_global_ref(logger_class)? })
	}

	fn log_impl(&self, record: &log::Record) -> jni::errors::Result<()> {
		let mut env = self.vm.attach_current_thread()?;
		let level: &str = SLF4JLogLevel::from(record.level()).into();
		let message = format!(
			"{}:{}: {}",
			record.file().unwrap_or("<unknown>"),
			record.line().unwrap_or(0),
			record.args(),
		);

		const SIGNATURE: &str = "(ILjava/lang/String;)V";
		let jstr = env.new_string(message)?;
		let jobj = JObject::from(jstr);
		let jvalue = JValue::Object(&jobj);
		let args = [jvalue];

		let result = env.call_static_method(&self.logger_class, level, SIGNATURE, args.as_slice());

		let throwable = env.exception_occurred()?;
		if **throwable == *JObject::null() {
			result?;
		} else {
			env.exception_clear()?;
		}
		Ok(())
	}
}

/// Implement the Log trait for SLF4JLogger.
impl log::Log for SLF4JLogger {
	fn enabled(&self, _metadata: &log::Metadata) -> bool {
		true
	}

	/// If a logging attempt produces an error, ignore,
	/// since obviously we can't log it :-)
	fn log(&self, record: &log::Record) {
		if self.log_impl(record).is_err() {}
	}

	fn flush(&self) {}
}

/// A low-level version of `run_ffi_safe` that just aborts on errors.
///
/// This is important for logging failures because we might want to log during the normal
/// `run_ffi_safe`. This should *not* be used normally because we don't want to crash the app!
fn abort_on_panic(f: impl FnOnce()) {
	catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|_e| {
		eprintln!("fatal error");
		// eprintln!("fatal error: {}", describe_panic(&e));
		abort();
	});
}

fn set_max_level_from_slf4j_level(max_level: jint) {
	let level: SLF4JLogLevel = match max_level.try_into() {
		Ok(level) => level,
		_ => panic!("invalid log level"),
	};

	log::set_max_level(log::Level::from(level).to_level_filter())
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_signal_libsignal_internal_Native_Logger_1Initialize(
	env: JNIEnv,
	_class: JClass,
	max_level: jint,
	logger_class: JClass,
) {
	abort_on_panic(|| {
		let logger = SLF4JLogger::new(env, logger_class).expect("could not initialize logging");

		match log::set_logger(Box::leak(Box::new(logger))) {
			Ok(_) => {
				set_max_level_from_slf4j_level(max_level);
				log::info!(
					"Initializing {} version:{}",
					env!("CARGO_PKG_NAME"),
					env!("CARGO_PKG_VERSION")
				);
				let backtrace_mode = {
					cfg_if::cfg_if! {
						if #[cfg(target_os = "android")] {
							log_panics::BacktraceMode::Unresolved
						} else {
							log_panics::BacktraceMode::Resolved
						}
					}
				};
				log_panics::Config::new().backtrace_mode(backtrace_mode).install_panic_hook();
			},
			Err(_) => {
				log::warn!(
					"Duplicate logger initialization ignored for {}",
					env!("CARGO_PKG_NAME")
				);
			},
		}
	});
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_signal_libsignal_internal_Native_Logger_1SetMaxLevel(
	_env: JNIEnv,
	_class: JClass,
	max_level: jint,
) {
	abort_on_panic(|| set_max_level_from_slf4j_level(max_level));
}
