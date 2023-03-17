use serde::{
	de::{SeqAccess, Visitor},
	Deserialize, Deserializer, Serialize, Serializer,
};
use std::{cmp, error::Error, fmt};

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

/// Public key defined in DSNP used for encryption/decryption in private graph
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DsnpPublicKey {
	/// Multi-codec public key
	#[serde(rename = "publicKey")]
	#[serde(with = "serde_bytes")]
	pub key: Vec<u8>,

	/// User-Assigned Key Identifier
	#[serde(rename = "keyId")]
	pub key_id: u64,

	/// Unix epoch seconds
	#[serde(rename = "revokedAsOf")]
	pub revoked_as_of: u64,
}

/// Public Graph Chunk defined in DSNP to store compressed public graph
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DsnpUserPublicGraphChunk {
	#[serde(rename = "compressedPublicGraph")]
	#[serde(with = "serde_bytes")]
	pub compressed_public_graph: Vec<u8>,
}

/// Graph Edge defined in DSNP to store each connection
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DsnpGraphEdge {
	/// DSNP User Id of object of relationship
	#[serde(rename = "userId")]
	pub user_id: u64,

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
mod tests {
	use super::*;

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
