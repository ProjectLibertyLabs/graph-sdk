use anyhow::Result;
pub use dsnp_graph_config::DsnpUserId;
use serde::{
	de::{SeqAccess, Visitor},
	Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
	cmp,
	cmp::Ordering,
	error::Error,
	fmt,
	hash::{Hash, Hasher},
};

/// Prid len in bytes
const PRID_LEN_IN_BYTES: usize = 8;
/// Inner Graph type used in both private and public graphs
pub type DsnpInnerGraph = Vec<DsnpGraphEdge>;

/// `Pseudonymous Relationship Identifier` which allows private connection verification
/// Wrapping in its own Struct allows easier serialization and deserialization
#[derive(Debug, Clone, Eq, PartialEq)]
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

	/// User-Assigned Key Identifier which in Frequency it is itemized index from Stateful storage
	/// This is not being serialized or deserialized directly and is only used to eliminate adding
	/// a new type for keys that encapsulates keys and their ids.
	#[serde(skip)]
	pub key_id: Option<u64>,
}

/// Public Graph Chunk defined in DSNP to store compressed public graph
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DsnpUserPublicGraphChunk {
	#[serde(rename = "compressedPublicGraph")]
	#[serde(with = "serde_bytes")]
	pub compressed_public_graph: Vec<u8>,
}

/// Graph Edge defined in DSNP to store each connection
#[repr(C)]
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct DsnpGraphEdge {
	/// DSNP User Id of object of relationship
	#[serde(rename = "userId")]
	pub user_id: DsnpUserId,

	/// Unix epoch in seconds when this relationship was originally established rounded to the nearest 1000
	pub since: u64,
}

impl PartialEq for DsnpGraphEdge {
	fn eq(&self, other: &Self) -> bool {
		self.user_id == other.user_id
	}
}

impl Eq for DsnpGraphEdge {}

impl Hash for DsnpGraphEdge {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.user_id.hash(state);
	}
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

	/// lib_sodium sealed box
	#[serde(rename = "encryptedCompressedPrivateGraph")]
	#[serde(with = "serde_bytes")]
	pub encrypted_compressed_private_graph: Vec<u8>,
}

/// Deserialized Private Graph Chunk
#[derive(Debug, Clone, PartialEq)]
pub struct PrivateGraphChunk {
	/// User-Assigned Key Identifier
	pub key_id: u64,

	/// User-Assigned Key Identifier
	pub prids: Vec<DsnpPrid>,

	/// connections
	pub inner_graph: DsnpInnerGraph,
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

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn dsnp_public_key_should_be_ordered_by_key_id_asc() {
		let a = DsnpPublicKey { key_id: Some(1), key: vec![] };
		let b = DsnpPublicKey { key_id: Some(19), key: vec![] };
		let c = DsnpPublicKey { key_id: Some(20), key: vec![] };
		let mut arr = vec![b.clone(), a.clone(), c.clone()];

		arr.sort();

		assert_eq!(arr, vec![a, b, c]);
	}

	#[test]
	#[should_panic]
	fn prid_creation_with_less_than_8_byte_values_should_fail() {
		DsnpPrid::new(&[1, 2, 3, 4, 5, 6, 7]);
	}

	#[test]
	#[should_panic]
	fn prid_creation_with_more_than_8_byte_values_should_fail() {
		DsnpPrid::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
	}
}
