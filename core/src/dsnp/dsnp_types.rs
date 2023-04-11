use serde::{
	de::{SeqAccess, Visitor},
	Deserialize, Deserializer, Serialize, Serializer,
};
use std::{cmp, cmp::Ordering, error::Error, fmt};

/// DSNP User Id
pub type DsnpUserId = u64;

/// Prid len in bytes
const PRID_LEN_IN_BYTES: usize = 8;
/// Inner Graph type used in both private and public graphs
pub type DsnpInnerGraph = Vec<DsnpGraphEdge>;
/// `Pseudonymous Relationship Identifier` which allows private connection verification
/// Wrapping in its own Struct allows easier serialization and deserialization
#[derive(Debug, Clone, PartialEq)]
pub struct DsnpPrid {
	inner: Vec<u8>,
}

impl From<Vec<u8>> for DsnpPrid {
	fn from(vec: Vec<u8>) -> Self {
		DsnpPrid { inner: vec }
	}
}

/// Public key defined in DSNP used for encryption/decryption in private graph
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct DsnpPublicKey {
	/// Multi-codec public key
	#[serde(rename = "publicKey")]
	#[serde(with = "serde_bytes")]
	pub key: Vec<u8>,

	/// User-Assigned Key Identifier
	#[serde(rename = "keyId")]
	pub key_id: u64,
}

/// Public Graph Chunk defined in DSNP to store compressed public graph
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DsnpUserPublicGraphChunk {
	#[serde(rename = "compressedPublicGraph")]
	#[serde(with = "serde_bytes")]
	pub compressed_public_graph: Vec<u8>,
}

/// Graph Edge defined in DSNP to store each connection
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct DsnpGraphEdge {
	/// DSNP User Id of object of relationship
	#[serde(rename = "userId")]
	pub user_id: DsnpUserId,

	/// Unix epoch in seconds when this relationship was originally established rounded to the nearest 1000
	pub since: u64,
}

/// Private Graph Chunk defined in DSNP to store compressed and encrypted private graph
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DsnpUserPrivateGraphChunk {
	/// User-Assigned Key Identifier
	#[serde(rename = "keyId")]
	pub key_id: u64,

	/// list of `Pseudonymous Relationship Identifier`s associated with this private graph chunk
	#[serde(rename = "pridList")]
	pub prids: Vec<DsnpPrid>,

	/// Days since the Unix Epoch when PRIds were last refreshed for this chunk
	#[serde(rename = "lastUpdated")]
	pub last_updated: u64,

	/// lib_sodium sealed box
	#[serde(rename = "encryptedCompressedPrivateGraph")]
	#[serde(with = "serde_bytes")]
	pub encrypted_compressed_private_graph: Vec<u8>,
}

impl DsnpPrid {
	/// Construct a new `DsnpPrid`
	pub fn new(data: &[u8]) -> Self {
		let d = data.to_vec();
		assert_eq!(d.len(), PRID_LEN_IN_BYTES, "Prid size should be {} bytes", PRID_LEN_IN_BYTES);
		Self { inner: d }
	}
}

/// Serialization of avro fixed type
impl Serialize for DsnpPrid {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_bytes(&self.inner)
	}
}

/// Deserialization of avro fixed type
impl<'de> Deserialize<'de> for DsnpPrid {
	fn deserialize<D>(deserializer: D) -> Result<DsnpPrid, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_byte_buf(PridVisitor)
	}
}

impl PartialOrd for DsnpPublicKey {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for DsnpPublicKey {
	fn cmp(&self, other: &Self) -> Ordering {
		self.key_id.cmp(&other.key_id)
	}
}

/// Used in deserialization of fixed type
struct PridVisitor;

impl<'de> Visitor<'de> for PridVisitor {
	type Value = DsnpPrid;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("byte array")
	}

	fn visit_str<E>(self, v: &str) -> Result<DsnpPrid, E>
	where
		E: Error,
	{
		Ok(DsnpPrid::new(v.as_bytes()))
	}

	fn visit_string<E>(self, v: String) -> Result<DsnpPrid, E>
	where
		E: Error,
	{
		Ok(DsnpPrid::new(v.as_bytes()))
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<DsnpPrid, E>
	where
		E: Error,
	{
		Ok(DsnpPrid::new(v))
	}

	fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<DsnpPrid, E>
	where
		E: Error,
	{
		Ok(DsnpPrid::new(&v))
	}

	fn visit_seq<V>(self, mut visitor: V) -> Result<DsnpPrid, V::Error>
	where
		V: SeqAccess<'de>,
	{
		let len = cmp::min(visitor.size_hint().unwrap_or(0), PRID_LEN_IN_BYTES);
		let mut bytes = Vec::with_capacity(len);

		while let Some(b) = visitor.next_element()? {
			bytes.push(b);
		}

		Ok(DsnpPrid::new(&bytes))
	}
}
