use anyhow::{Error, Result};
use miniz_oxide::{
	deflate::{compress_to_vec, CompressionLevel},
	inflate::decompress_to_vec,
};

/// Common trait for different compression algorithms
pub trait CompressionBehavior {
	/// compress the input
	fn compress(obj: &[u8]) -> Result<Vec<u8>>;

	/// decompress the input
	fn decompress(data: &[u8]) -> Result<Vec<u8>>;
}

/// Deflate Compression algorithm
pub struct DeflateCompression;

impl CompressionBehavior for DeflateCompression {
	fn compress(obj: &[u8]) -> Result<Vec<u8>> {
		Ok(compress_to_vec(obj, CompressionLevel::BestCompression as u8))
	}

	fn decompress(data: &[u8]) -> Result<Vec<u8>> {
		let val = decompress_to_vec(data)
			.map_err(|e| Error::msg(format!("failed to decompress {:?}", e.status)))?;
		Ok(val)
	}
}
