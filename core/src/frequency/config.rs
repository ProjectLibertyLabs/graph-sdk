use super::*;
use crate::dsnp::api_types::Config;

impl Config for Frequency {
	/// Graph page id
	type PageId = u16;
	/// Schema ID
	type SchemaId = u16;

	// todo: conditional config compilation for environment (local, rococo, mainnet, tests) using read schema values for mainnet & rococo
	const PUBLIC_FOLLOW_SCHEMAID: Self::SchemaId = 3;
	const PRIVATE_FOLLOW_SCHEMAID: Self::SchemaId = 4;
	const PUBLIC_FRIEND_SCHEMAID: Self::SchemaId = 5;
	const PRIVATE_FRIEND_SCHEMAID: Self::SchemaId = 6;
}
