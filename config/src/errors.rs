//! Errors for graph-sdk crate
//!

use thiserror::Error;

pub type DsnpGraphResult<T> = std::result::Result<T, DsnpGraphError>;

#[derive(Debug, Error)]
pub enum DsnpGraphError {
	// Errors related to Avro
	#[error("Avro error: {0}")]
	AvroError(String),

	// Failure to import a user graph
	#[error("User graph for {0} is not imported")]
	UserGraphNotImported(String),

	// no public key found for user
	#[error("No public key found for user: {0}")]
	NoPublicKeyFoundForUser(String),

	// given public key already exists
	#[error("Given public key already exists: {0}")]
	PublicKeyAlreadyExists(String),

	// Failed to encrypt
	#[error("Failed to encrypt: {0}")]
	EncryptError(String),

	// Failed to decrypt
	#[error("Failed to decrypt: {0}")]
	DecryptError(String),

	// Failed to SerializeKey
	#[error("Failed to SerializeKey: {0}")]
	SerializeKeyError(String),

	// Failed to deserialize key
	#[error("Failed to deserialize key: {0}")]
	DeserializeKeyError(String),

	// "imported key not found for user {} and id {}"
	#[error("Imported key not found for user {0} and id {1}")]
	ImportedKeyNotFound(String, String),

	// no pris imported for user
	#[error("No pris imported for user: {0}")]
	NoPrisImportedForUser(String),

	//Failed to decompress
	#[error("Failed to decompress: {0}")]
	DecompressError(String),

	// Incorrect connection type
	#[error("Incorrect connection type: {0}")]
	IncorrectConnectionType(String),

	// Invalid Page ID
	#[error("Invalid Page ID: {0}")]
	InvalidPageId(String),

	// Page error
	#[error("Page error: {0}")]
	PageError(String),

	// "Connection from {} to {} does not exists to be disconnected!"
	#[error("Connection from {0} to {1} does not exists!")]
	ConnectionDoesNotExist(String, String),

	// "Connection from {} to {} already exists!"
	#[error("Connection from {0} to {1} already exists!")]
	ConnectionAlreadyExists(String, String),

	// Invalid schema id
	#[error("Invalid schema id: {0}")]
	InvalidSchemaId(String),

	// Dsnp version of {} schema is not supported!
	#[error(" unsupported schema: {0}")]
	UnsupportedSchema(String),

	// Key derivation error
	#[error("Key derivation error: {0}")]
	KeyDerivationError(String),

	// page_id: {}, prids len should be equal to connections len (connections: {}, prids: {})
	#[error(
		"page_id: {0}, prids len should be equal to connections len (connections: {1}, prids: {2})"
	)]
	PridsLenShouldBeEqualToConnectionsLen(String, usize, usize),

	// generic error
	#[error("Unknown error caught: {0}")]
	UnknownError(String),

	// Invalid private schema id
	#[error("Invalid private schema id")]
	InvalidPrivateSchemaId,

	// Graph is full error
	#[error("Graph is full")]
	GraphIsFull,

	// GraphState instance is full
	#[error("GraphState instance is full")]
	GraphStateIsFull,

	// No resolved active key found error
	#[error("No resolved active key found")]
	NoResolvedActiveKeyFound,

	// New page for existing page id error
	#[error("New page for existing page id")]
	NewPageForExistingPageId,

	// Failed to retrieve new graph page error
	#[error("Failed to retrieve graph page")]
	FailedToRetrieveGraphPage,

	// Duplicate connection detected error
	#[error("Add of duplicate connection in another page detected")]
	DuplicateConnectionDetected,

	// Connection not found error
	#[error("Connection not found")]
	ConnectionNotFound,

	// Duplicate update events error
	#[error("Duplicate update events detected")]
	DuplicateUpdateEvents,

	// Call to private friends in public graph error
	#[error("Call to private friends in non private graph")]
	CallToPrivateFriendsInPublicGraph,

	// Call to PRIDs in public graph error
	#[error("Calling apply_prids in non private friendship graph!")]
	CallToPridsInPublicGraph,

	// Event exists
	#[error("Event exists")]
	EventExists,

	// Invalid key
	#[error("Invalid public key")]
	InvalidPublicKey,

	// Invalid secret key
	#[error("Invalid secret key")]
	InvalidSecretKey,

	// Public key not compatible with secret key
	#[error("Public key not compatible with secret key")]
	PublicKeyNotCompatibleWithSecretKey,

	// Unable to decrypt private graph with any of the imported keys
	#[error("Unable to decrypt private graph with any of the imported keys")]
	UnableToDecryptGraphChunkWithAnyKey,

	// Incompatible privacy type for blob export
	#[error("Incompatible privacy type for blob export")]
	IncompatiblePrivacyTypeForBlobExport,

	// Failed to acquire write lock on state manager
	#[error("Failed to acquire write lock on state manager")]
	FailedtWriteLockStateManager,

	// Failed to acquire read lock on state manager
	#[error("Failed to acquire read lock on state manager")]
	FailedtReadLockStateManager,
}

impl From<apache_avro::Error> for DsnpGraphError {
	fn from(error: apache_avro::Error) -> Self {
		// Convert the underlying error to DsnpGraphError variant
		// based on your error handling logic
		// For example:
		DsnpGraphError::AvroError(error.to_string())
	}
}
