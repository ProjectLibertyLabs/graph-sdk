//! Errors for graph-sdk crate
//!
use super::*;
use thiserror::Error;

pub type DsnpGraphResult<T> = std::result::Result<T, DsnpGraphError>;

#[repr(u8)]
#[derive(Debug, Error)]
pub enum DsnpGraphError {
	#[error(transparent)]
	AvroError(#[from] apache_avro::Error),

	#[error("Add of duplicate connection in another page detected")]
	DuplicateConnectionDetected,

	#[error("Calling apply_prids in non private friendship graph!")]
	CallToPridsInPublicGraph,

	#[error("Call to private friends in non private graph")]
	CallToPrivateFriendsInPublicGraph,

	#[error("Connection from {0} to {1} already exists!")]
	ConnectionAlreadyExists(DsnpUserId, DsnpUserId),

	#[error("Connection from {0} to {1} does not exist!")]
	ConnectionDoesNotExist(DsnpUserId, DsnpUserId),

	#[error("Connection not found")]
	ConnectionNotFound,

	#[error("Failed to decompress: {0}")]
	DecompressError(String),

	#[error("Failed to decrypt: {0}")]
	DecryptionError(String),

	#[error("Duplicate update events detected")]
	DuplicateUpdateEvents,

	#[error("Event exists")]
	EventExists,

	#[error("Failed to encrypt: {0}")]
	EncryptionError(String),

	#[error("Failed to retrieve graph page")]
	FailedToRetrieveGraphPage,

	#[error("Failed to acquire read lock on state manager")]
	FailedtoReadLockStateManager,

	#[error("Failed to acquire write lock on state manager")]
	FailedtoWriteLockStateManager,

	#[error("FFI error: {0}")]
	FFIError(String),

	#[error("Graph is full")]
	GraphIsFull,

	#[error("GraphState instance is full")]
	GraphStateIsFull,

	#[error("Invalid user id: {0}")]
	InvalidDsnpUserId(DsnpUserId),

	#[error("Invalid schema id: {0}")]
	InvalidSchemaId(SchemaId),

	#[error("Invalid Page ID: {0}")]
	InvalidPageId(PageId),

	#[error("Invalid private schema id")]
	InvalidPrivateSchemaId,

	#[error("Invalid public key")]
	InvalidPublicKey,

	#[error("Invalid secret key")]
	InvalidSecretKey,

	#[error("{0}")]
	InvalidInput(String),

	#[error("Imported key not found for user {0} and id {1}")]
	ImportedKeyNotFound(DsnpUserId, String),

	#[error("Incorrect connection type: {0}")]
	IncorrectConnectionType(String),

	#[error("Incompatible privacy type for blob export")]
	IncompatiblePrivacyTypeForBlobExport,

	#[error("Key derivation error: {0}")]
	KeyDerivationError(String),

	#[error("No pris imported for user: {0}")]
	NoPrisImportedForUser(DsnpUserId),

	#[error("No public key found for user: {0}")]
	NoPublicKeyFoundForUser(DsnpUserId),

	#[error("No resolved active key found")]
	NoResolvedActiveKeyFound,

	#[error("New page for existing page id")]
	NewPageForExistingPageId,

	#[error("Page is aggressively full")]
	PageAggressivelyFull,

	#[error("Page is trivially full")]
	PageTriviallyFull,

	#[error("Given public key already exists: {0}")]
	PublicKeyAlreadyExists(String),

	#[error("Public key not compatible with secret key")]
	PublicKeyNotCompatibleWithSecretKey,

	#[error(
		"page_id: {0}, prids len should be equal to connections len (connections: {1}, prids: {2})"
	)]
	PridsLenShouldBeEqualToConnectionsLen(PageId, usize, usize),

	#[error("Unsupported schema: {0}")]
	UnsupportedSchema(SchemaId),

	#[error(transparent)]
	Unknown(#[from] anyhow::Error),

	#[error("User graph for {0} is not imported")]
	UserGraphNotImported(DsnpUserId),

	#[error("Unable to decrypt private graph with any of the imported keys")]
	UnableToDecryptGraphChunkWithAnyKey,
}

impl DsnpGraphError {
	pub fn error_code(&self) -> i32 {
		match self {
			DsnpGraphError::AvroError { .. } => 1,
			DsnpGraphError::DuplicateConnectionDetected => 2,
			DsnpGraphError::CallToPridsInPublicGraph => 3,
			DsnpGraphError::CallToPrivateFriendsInPublicGraph => 4,
			DsnpGraphError::ConnectionAlreadyExists(..) => 5,
			DsnpGraphError::ConnectionDoesNotExist(..) => 6,
			DsnpGraphError::ConnectionNotFound => 7,
			DsnpGraphError::DecompressError(_) => 8,
			DsnpGraphError::DecryptionError(_) => 9,
			DsnpGraphError::DuplicateUpdateEvents => 10,
			DsnpGraphError::EventExists => 11,
			DsnpGraphError::EncryptionError(_) => 12,
			DsnpGraphError::FailedToRetrieveGraphPage => 13,
			DsnpGraphError::FailedtoReadLockStateManager => 14,
			DsnpGraphError::FailedtoWriteLockStateManager => 15,
			DsnpGraphError::GraphIsFull => 16,
			DsnpGraphError::GraphStateIsFull => 17,
			DsnpGraphError::InvalidDsnpUserId(_) => 18,
			DsnpGraphError::InvalidSchemaId(_) => 19,
			DsnpGraphError::InvalidPageId(_) => 20,
			DsnpGraphError::InvalidPrivateSchemaId => 21,
			DsnpGraphError::InvalidPublicKey => 22,
			DsnpGraphError::InvalidSecretKey => 23,
			DsnpGraphError::InvalidInput(_) => 24,
			DsnpGraphError::ImportedKeyNotFound(..) => 25,
			DsnpGraphError::IncorrectConnectionType(_) => 26,
			DsnpGraphError::IncompatiblePrivacyTypeForBlobExport => 27,
			DsnpGraphError::KeyDerivationError(_) => 28,
			DsnpGraphError::NoPrisImportedForUser(_) => 29,
			DsnpGraphError::NoPublicKeyFoundForUser(_) => 30,
			DsnpGraphError::NoResolvedActiveKeyFound => 31,
			DsnpGraphError::NewPageForExistingPageId => 32,
			DsnpGraphError::PageAggressivelyFull => 33,
			DsnpGraphError::PageTriviallyFull => 34,
			DsnpGraphError::PublicKeyAlreadyExists(_) => 35,
			DsnpGraphError::PublicKeyNotCompatibleWithSecretKey => 36,
			DsnpGraphError::PridsLenShouldBeEqualToConnectionsLen(..) => 37,
			DsnpGraphError::UnsupportedSchema(_) => 38,
			DsnpGraphError::Unknown(..) => 39,
			DsnpGraphError::UserGraphNotImported(_) => 40,
			DsnpGraphError::UnableToDecryptGraphChunkWithAnyKey => 41,
			DsnpGraphError::FFIError(_) => 42,
		}
	}
}

/// Macro to replicate Option<T>::ok_or, but logging if the returned
/// Result is an Err variant.
// (note: could have been implemented as a trait, but then the resulting log
// event would not contain the correct file:line, since the std::file! macro records
// the location only from an enclosing macro call)
#[macro_export]
macro_rules! ok_or_log {
	($target:expr, $error:expr, $level:expr) => {{
		$target.ok_or_else(|| {
			log::log!($level, "{}", $error);
			$error
		})
	}};
	($target:expr, $error:expr, $level:expr, $context:expr) => {{
		$target.ok_or_else(|| {
			log::log!($level, "{}: {}", $context, $error);
			$error
		})
	}};
}

/// Macro to log a Result::Err.
// (note: could have been implemented as a trait, but then the resulting log
// event would not contain the correct file:line, since the std::file! macro records
// the location only from an enclosing macro call)
#[macro_export]
macro_rules! log_err {
	($target:expr) => {{
		let r = $target;
		if let Some(err) = r.as_ref().err() {
			log::error!("{}", err);
		}
		r
	}};
	($target:expr, $context:expr) => {{
		let r = $target;
		if let Some(err) = r.as_ref().err() {
			log::error!("{}:\n{}", $context, err);
		}
		r
	}};
}
