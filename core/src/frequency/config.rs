use super::*;
use crate::dsnp::api_types::{Config, SchemaId};

impl Config for Frequency {
	// todo: conditional config compilation for environment (local, rococo, mainnet, tests) using read schema values for mainnet & rococo
	const PUBLIC_FOLLOW_SCHEMAID: SchemaId = 3;
	const PRIVATE_FOLLOW_SCHEMAID: SchemaId = 4;
	const PUBLIC_FRIEND_SCHEMAID: SchemaId = 5;
	const PRIVATE_FRIEND_SCHEMAID: SchemaId = 6;
}
