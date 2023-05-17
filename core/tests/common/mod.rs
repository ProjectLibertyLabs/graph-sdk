use dsnp_graph_config::{ConnectionType, Environment, SchemaId};

pub fn get_schema_from(env: Environment, connection_type: ConnectionType) -> SchemaId {
	env.get_config()
		.get_schema_id_from_connection_type(connection_type)
		.expect("get_schema_from connection_type should exist")
}
