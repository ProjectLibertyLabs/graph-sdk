use dsnp_graph_config::errors::DsnpGraphError;
use jni::{objects::JThrowable, JNIEnv};
use std::fmt;

#[derive(Debug)]
pub enum SdkJniError {
	DsnpGraph(DsnpGraphError),
	Jni(jni::errors::Error),
	BadJniParameter(&'static str),
	InvalidProto(protobuf::Error),
	InvalidRequest(&'static str),
	UnexpectedResponse(&'static str),
	LockError,
	InvalidHandle(&'static str),
	UnexpectedPanic(std::boxed::Box<dyn std::any::Any + std::marker::Send>),
}

pub fn throw_exception(env: &mut JNIEnv, error: SdkJniError) {
	let (class, signature) = error.get_java_class_and_constructor_signature();
	let throwable = env
		.new_string(error.to_string())
		.and_then(|message| env.new_object(class, signature, &[(&message).into()]));
	match throwable {
		Ok(o) => {
			let result = env.throw(JThrowable::from(o));
			if let Err(failure) = result {
				log::error!("failed to throw exception for {}: {}", error, failure);
			}
		},
		Err(e) => {
			log::error!("failed to create exception for {}", SdkJniError::from(e));
		},
	}
}

impl From<DsnpGraphError> for SdkJniError {
	fn from(e: DsnpGraphError) -> SdkJniError {
		SdkJniError::DsnpGraph(e)
	}
}

impl From<jni::errors::Error> for SdkJniError {
	fn from(e: jni::errors::Error) -> SdkJniError {
		SdkJniError::Jni(e)
	}
}

impl From<protobuf::Error> for SdkJniError {
	fn from(e: protobuf::Error) -> SdkJniError {
		SdkJniError::InvalidProto(e)
	}
}

impl fmt::Display for SdkJniError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			SdkJniError::DsnpGraph(s) => write!(f, "ErrorCode({}) {}", s.error_code(), s),
			SdkJniError::Jni(s) => write!(f, "JNI error {}", s),
			SdkJniError::InvalidProto(s) => write!(f, "invalid proto {}", s),
			SdkJniError::InvalidRequest(s) => write!(f, "invalid request {}", s),
			SdkJniError::UnexpectedResponse(s) => write!(f, "unexpected response {}", s),
			SdkJniError::LockError => write!(f, "unable to acquire lock"),
			SdkJniError::InvalidHandle(s) => write!(f, "invalid handle {}", s),
			SdkJniError::BadJniParameter(m) => write!(f, "bad parameter type {}", m),
			SdkJniError::UnexpectedPanic(e) => {
				write!(f, "unexpected panic: {}", describe_panic(e))
			},
		}
	}
}

// Related to https://github.com/rust-lang/rfcs/issues/1389
pub fn describe_panic(any: &Box<dyn std::any::Any + Send>) -> String {
	if let Some(msg) = any.downcast_ref::<&str>() {
		msg.to_string()
	} else if let Some(msg) = any.downcast_ref::<String>() {
		msg.to_string()
	} else {
		"(break on rust_panic to debug)".to_string()
	}
}

const JAVA_ERROR_PATH: &'static str = "io/amplica/graphsdk/exceptions/";

impl SdkJniError {
	fn get_java_class_and_constructor_signature(&self) -> (String, String) {
		let string_message_signature = "(Ljava/lang/String;)V".to_string();
		match self {
			SdkJniError::DsnpGraph(_) => {
				(format!("{}{}", JAVA_ERROR_PATH, "GraphSdkException"), string_message_signature)
			},
			SdkJniError::Jni(_) => {
				(format!("{}{}", JAVA_ERROR_PATH, "JniException"), string_message_signature)
			},
			SdkJniError::InvalidProto(_) => (
				format!("{}{}", JAVA_ERROR_PATH, "InvalidProtoException"),
				string_message_signature,
			),
			SdkJniError::InvalidRequest(_) | SdkJniError::BadJniParameter(_) => (
				format!("{}{}", JAVA_ERROR_PATH, "InvalidRequestException"),
				string_message_signature,
			),
			SdkJniError::UnexpectedResponse(_) => (
				format!("{}{}", JAVA_ERROR_PATH, "UnexpectedResponseException"),
				string_message_signature,
			),
			SdkJniError::LockError => (
				format!("{}{}", JAVA_ERROR_PATH, "AcquiringLockException"),
				string_message_signature,
			),
			SdkJniError::InvalidHandle(_) => (
				format!("{}{}", JAVA_ERROR_PATH, "InvalidHandleException"),
				string_message_signature,
			),
			SdkJniError::UnexpectedPanic(_) => {
				(format!("{}{}", JAVA_ERROR_PATH, "UnknownException"), string_message_signature)
			},
		}
	}
}
