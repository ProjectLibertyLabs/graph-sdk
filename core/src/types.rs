use serde::{
	de::{SeqAccess, Visitor},
	Deserialize, Deserializer, Serialize, Serializer,
};
use std::{cmp, error::Error, fmt};

/// Prid len in bytes
const PRID_LEN_IN_BYTES: usize = 8;
/// Public Graph type
pub type PublicGraph = Vec<GraphEdge>;
/// Pseudonymous Relationship Identifier
#[derive(Debug, Clone, PartialEq)]
pub struct Prid(Vec<u8>);

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PublicKey {
	/// Multi-codec public key
	#[serde(rename = "publicKey")]
	#[serde(with = "serde_bytes")]
	pub key: Vec<u8>,

	/// User-Assigned Key Identifier
	#[serde(rename = "keyId")]
	pub key_id: i64,

	/// Unix epoch seconds
	#[serde(rename = "revokedAsOf")]
	pub revoked_as_of: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct UserPublicGraphChunk {
	#[serde(rename = "compressedPublicGraph")]
	#[serde(with = "serde_bytes")]
	pub compressed_public_graph: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct GraphEdge {
	/// DSNP User Id of object of relationship
	#[serde(rename = "userId")]
	pub user_id: i64,

	/// Unix epoch in seconds when this relationship was originally established rounded to the nearest 1000
	pub since: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct UserPrivateGraphChunk {
	/// User-Assigned Key Identifier
	#[serde(rename = "keyId")]
	pub key_id: i64,

	/// User-Assigned Key Identifier
	#[serde(rename = "pridList")]
	pub prids: Vec<Prid>,

	/// lib_sodium sealed box
	#[serde(rename = "encryptedCompressedPrivateGraph")]
	#[serde(with = "serde_bytes")]
	pub encrypted_compressed_private_graph: Vec<u8>,
}

impl Prid {
	pub fn new(data: &[u8]) -> Self {
		let d = data.to_vec();
		assert_eq!(d.len(), PRID_LEN_IN_BYTES, "Prid size should be {} bytes", PRID_LEN_IN_BYTES);
		Self(d)
	}
}

impl Serialize for Prid {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_bytes(&self.0)
	}
}

impl<'de> Deserialize<'de> for Prid {
	fn deserialize<D>(deserializer: D) -> Result<Prid, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_byte_buf(PridVisitor)
	}
}

struct PridVisitor;

impl<'de> Visitor<'de> for PridVisitor {
	type Value = Prid;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("byte array")
	}

	fn visit_str<E>(self, v: &str) -> Result<Prid, E>
	where
		E: Error,
	{
		Ok(Prid::new(v.as_bytes()))
	}

	fn visit_string<E>(self, v: String) -> Result<Prid, E>
	where
		E: Error,
	{
		Ok(Prid::new(v.as_bytes()))
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Prid, E>
	where
		E: Error,
	{
		Ok(Prid::new(v))
	}

	fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Prid, E>
	where
		E: Error,
	{
		Ok(Prid::new(&v))
	}

	fn visit_seq<V>(self, mut visitor: V) -> Result<Prid, V::Error>
	where
		V: SeqAccess<'de>,
	{
		let len = cmp::min(visitor.size_hint().unwrap_or(0), PRID_LEN_IN_BYTES);
		let mut bytes = Vec::with_capacity(len);

		while let Some(b) = visitor.next_element()? {
			bytes.push(b);
		}

		Ok(Prid::new(&bytes))
	}
}
