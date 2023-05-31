use dryoc::keypair::StackKeyPair;
use dsnp_graph_config::{ConnectionType, Environment, GraphKeyType, SchemaId};
use dsnp_graph_core::dsnp::{
	api_types::{GraphKeyPair, ResolvedKeyPair},
	dsnp_configs::KeyPairType,
};

pub fn get_schema_from(env: Environment, connection_type: ConnectionType) -> SchemaId {
	env.get_config()
		.get_schema_id_from_connection_type(connection_type)
		.expect("get_schema_from connection_type should exist")
}

pub fn create_new_keys(key_id: u64) -> (StackKeyPair, ResolvedKeyPair, GraphKeyPair) {
	let key_pair_raw = StackKeyPair::gen();
	let resolved_key =
		ResolvedKeyPair { key_pair: KeyPairType::Version1_0(key_pair_raw.clone()), key_id };
	let keypair = GraphKeyPair {
		secret_key: key_pair_raw.secret_key.to_vec(),
		public_key: key_pair_raw.public_key.to_vec(),
		key_type: GraphKeyType::X25519,
	};
	(key_pair_raw, resolved_key, keypair)
}
