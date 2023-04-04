use crate::dsnp::compression::{CompressionBehavior, DeflateCompression};

#[test]
fn deflate_compression_should_compress_and_decompress() {
	let data = vec![
		2u8, 1, 0, 23, 5, 82, 100, 56, 23, 120, 200, 250, 140, 83, 98, 0, 10, 234, 88, 23, 54, 23,
		23, 109, 198, 111, 70, 2, 89, 2u8, 1, 0, 23, 5, 82, 100, 56, 1, 120, 200, 250, 140, 83, 98,
		0, 10, 234, 88, 23, 54, 23, 23, 109, 198, 111, 70, 2, 89,
	];

	let compressed = DeflateCompression::compress(&data).unwrap();
	let decompressed = DeflateCompression::decompress(&compressed).unwrap();

	assert_eq!(decompressed, data);
}
