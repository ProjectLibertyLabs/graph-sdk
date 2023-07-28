//! Errors for graph-sdk crate
//!
use super::*;
use thiserror::Error;

/// Result type for graph SDK calls
pub type DsnpGraphResult<T> = std::result::Result<T, DsnpGraphError>;

/// Graph SDK errors
#[repr(u8)]
#[derive(Debug, Error)]
pub enum DsnpGraphError {
	/// Avro error
	#[error(transparent)]
	AvroError(#[from] apache_avro::Error),

	/// Duplicate connection detected
	#[error("Add of duplicate connection in another page detected")]
	DuplicateConnectionDetected,

	/// Calling apply_prids in non private friendship graph!
	#[error("Calling apply_prids in non private friendship graph!")]
	CallToPridsInPublicGraph,

	/// Calling private friends in non private graph
	#[error("Call to private friends in non private graph")]
	CallToPrivateFriendsInPublicGraph,

	/// Connection already exists
	#[error("Connection from {0} to {1} already exists!")]
	ConnectionAlreadyExists(DsnpUserId, DsnpUserId),

	/// Connection does not exist
	#[error("Connection from {0} to {1} does not exist!")]
	ConnectionDoesNotExist(DsnpUserId, DsnpUserId),

	/// Connection not found
	#[error("Connection not found")]
	ConnectionNotFound,

	/// Failed to decompress
	#[error("Failed to decompress: {0}")]
	DecompressError(String),

	/// Failed to decrypt
	#[error("Failed to decrypt: {0}")]
	DecryptionError(String),

	/// Duplicate update events detected
	#[error("Duplicate update events detected")]
	DuplicateUpdateEvents,

	/// Event exists
	#[error("Event exists")]
	EventExists,

	/// Failed to encrypt
	#[error("Failed to encrypt: {0}")]
	EncryptionError(String),

	/// Failed to retrieve graph page
	#[error("Failed to retrieve graph page")]
	FailedToRetrieveGraphPage,

	/// Failed to acquire read lock on state manager
	#[error("Failed to acquire read lock on {0}")]
	FailedtoReadLock(String),

	/// Failed to acquire write lock on state manager
	#[error("Failed to acquire write lock on {0}")]
	FailedtoWriteLock(String),

	/// FFI error
	#[error("FFI error: {0}")]
	FFIError(String),

	///	Graph is full
	#[error("Graph is full")]
	GraphIsFull,

	/// Invalid DSNP user id
	#[error("Invalid user id: {0}")]
	InvalidDsnpUserId(DsnpUserId),

	/// Invalid schema id
	#[error("Invalid schema id: {0}")]
	InvalidSchemaId(SchemaId),

	/// Invalid page id
	#[error("Invalid Page ID: {0}")]
	InvalidPageId(PageId),

	/// Invalid private schema id
	#[error("Invalid private schema id")]
	InvalidPrivateSchemaId,

	/// Invalid public key
	#[error("Invalid public key")]
	InvalidPublicKey,

	/// Invalid secret key
	#[error("Invalid secret key")]
	InvalidSecretKey,

	/// Invalid input
	#[error("{0}")]
	InvalidInput(String),

	/// Imported key not found
	#[error("Imported key not found for user {0} and id {1}")]
	ImportedKeyNotFound(DsnpUserId, String),

	/// Incorrect connection type
	#[error("Incorrect connection type: {0}")]
	IncorrectConnectionType(String),

	/// Incompatible privacy type for blob export
	#[error("Incompatible privacy type for blob export")]
	IncompatiblePrivacyTypeForBlobExport,

	/// Key derivation error
	#[error("Key derivation error: {0}")]
	KeyDerivationError(String),

	/// No pris imported for user
	#[error("No pris imported for user: {0}")]
	NoPrisImportedForUser(DsnpUserId),

	/// No public key found for user
	#[error("No public key found for user: {0}")]
	NoPublicKeyFoundForUser(DsnpUserId),

	/// No resolved active key found
	#[error("No resolved active key found")]
	NoResolvedActiveKeyFound,

	/// New page for existing page id
	#[error("New page for existing page id")]
	NewPageForExistingPageId,

	/// Page is aggressively full
	#[error("Page is aggressively full")]
	PageAggressivelyFull,

	/// Page is trivially full
	#[error("Page is trivially full")]
	PageTriviallyFull,

	/// Public key already exists
	#[error("Given public key already exists: {0}")]
	PublicKeyAlreadyExists(String),

	/// Public key not compatible with secret key
	#[error("Public key not compatible with secret key")]
	PublicKeyNotCompatibleWithSecretKey,

	/// Prids len should be equal to connections len
	#[error(
		"page_id: {0}, prids len should be equal to connections len (connections: {1}, prids: {2})"
	)]
	PridsLenShouldBeEqualToConnectionsLen(PageId, usize, usize),

	/// Unsupported schema
	#[error("Unsupported schema: {0}")]
	UnsupportedSchema(SchemaId),

	/// Unknown error
	#[error(transparent)]
	Unknown(#[from] anyhow::Error),

	/// User graph for user is not imported
	#[error("User graph for {0} is not imported")]
	UserGraphNotImported(DsnpUserId),

	/// Unable to decrypt private graph with any of the imported keys
	#[error("Unable to decrypt private graph with any of the imported keys")]
	UnableToDecryptGraphChunkWithAnyKey,

	/// Unsupported connection type
	#[error("No schema ID found for connection type")]
	UnsupportedConnectionTypeForConfig(ConnectionType),
}

impl DsnpGraphError {
	/// Returns the error code for the error
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
			DsnpGraphError::FailedtoReadLock(_) => 14,
			DsnpGraphError::FailedtoWriteLock(_) => 15,
			DsnpGraphError::GraphIsFull => 16,
			DsnpGraphError::InvalidDsnpUserId(_) => 17,
			DsnpGraphError::InvalidSchemaId(_) => 20,
			DsnpGraphError::InvalidPageId(_) => 21,
			DsnpGraphError::InvalidPrivateSchemaId => 22,
			DsnpGraphError::InvalidPublicKey => 23,
			DsnpGraphError::InvalidSecretKey => 24,
			DsnpGraphError::InvalidInput(_) => 25,
			DsnpGraphError::ImportedKeyNotFound(..) => 26,
			DsnpGraphError::IncorrectConnectionType(_) => 27,
			DsnpGraphError::IncompatiblePrivacyTypeForBlobExport => 28,
			DsnpGraphError::KeyDerivationError(_) => 29,
			DsnpGraphError::NoPrisImportedForUser(_) => 30,
			DsnpGraphError::NoPublicKeyFoundForUser(_) => 31,
			DsnpGraphError::NoResolvedActiveKeyFound => 32,
			DsnpGraphError::NewPageForExistingPageId => 33,
			DsnpGraphError::PageAggressivelyFull => 34,
			DsnpGraphError::PageTriviallyFull => 35,
			DsnpGraphError::PublicKeyAlreadyExists(_) => 36,
			DsnpGraphError::PublicKeyNotCompatibleWithSecretKey => 37,
			DsnpGraphError::PridsLenShouldBeEqualToConnectionsLen(..) => 38,
			DsnpGraphError::UnsupportedSchema(_) => 39,
			DsnpGraphError::Unknown(..) => 40,
			DsnpGraphError::UserGraphNotImported(_) => 41,
			DsnpGraphError::UnableToDecryptGraphChunkWithAnyKey => 42,
			DsnpGraphError::FFIError(_) => 43,
			DsnpGraphError::UnsupportedConnectionTypeForConfig(..) => 44,
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
