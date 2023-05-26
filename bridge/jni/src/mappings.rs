use dsnp_graph_config::{
	Config as RustConfig, DsnpUserId, DsnpVersion as RustDsnpVersion,
	Environment as RustEnvironment, GraphKeyType as RustGraphKeyType, PageId,
	SchemaConfig as RustSchemaConfig, SchemaConfig, SchemaId,
};
use dsnp_graph_core::dsnp::{
	api_types::{
		Action as RustAction, Connection as RustConnection, ConnectionType as RustConnectionType,
		DsnpKeys as RustDsnpKeys, GraphKeyPair as RustGraphKeyPair,
		ImportBundle as RustImportBundle, KeyData as RustKeyData, PageData as RustPageData,
		PrivacyType as RustPrivacyType, Update as RustUpdate,
	},
	dsnp_types::{DsnpGraphEdge as RustDsnpGraphEdge, DsnpPublicKey as RustDsnpPublicKey},
};
use dsnp_graph_sdk_common::proto_types::{
	input as proto_input,
	output::{
		self as proto_output,
		updates::update::{AddKeyUpdate, DeletePageUpdate, PersistPageUpdate},
	},
};
use jni::{
	objects::JByteArray,
	sys::{jboolean, JNI_FALSE, JNI_TRUE},
	JNIEnv,
};
use protobuf::{EnumOrUnknown, Message, SpecialFields};
use std::collections::HashMap;

pub fn map_to_environment<'local>(
	env: &JNIEnv<'local>,
	environment: &JByteArray,
) -> Option<RustEnvironment> {
	let bytes = env.convert_byte_array(environment).ok()?;
	let env_proto = proto_output::Environment::parse_from_bytes(&bytes).ok()?;
	let result = match env_proto.environment_type.enum_value().ok()? {
		proto_output::EnvironmentType::MainNet => RustEnvironment::Mainnet,
		proto_output::EnvironmentType::Rococo => RustEnvironment::Rococo,
		proto_output::EnvironmentType::Dev => {
			let cfg = env_proto.config.into_option()?;
			let rust_cfg = map_config_to_rust(cfg)?;
			RustEnvironment::Dev(rust_cfg)
		},
	};
	Some(result)
}

pub fn map_to_actions<'local>(
	env: &JNIEnv<'local>,
	actions: &JByteArray,
) -> Option<Vec<RustAction>> {
	let bytes = env.convert_byte_array(actions).ok()?;
	let actions_proto = proto_input::Actions::parse_from_bytes(&bytes).ok()?;

	let mut result = vec![];
	for a in actions_proto.actions {
		result.push(map_action_to_rust(a)?);
	}
	Some(result)
}

pub fn map_to_imports<'local>(
	env: &JNIEnv<'local>,
	imports: &JByteArray,
) -> Option<Vec<RustImportBundle>> {
	let bytes = env.convert_byte_array(imports).ok()?;
	let imports_proto = proto_input::ImportBundles::parse_from_bytes(&bytes).ok()?;
	let mut result = vec![];
	for i in imports_proto.bundles {
		result.push(RustImportBundle {
			schema_id: SchemaId::try_from(i.schema_id).ok()?,
			dsnp_user_id: i.dsnp_user_id,
			dsnp_keys: map_dsnp_keys_to_rust(i.dsnp_keys.as_ref()?)?,
			key_pairs: map_graph_key_pairs_to_rust(&i.key_pairs)?,
			pages: map_page_datas_to_rust(&i.pages)?,
		});
	}
	Some(result)
}

pub fn serialize_public_keys<'local>(
	env: &JNIEnv<'local>,
	public_keys: &[RustDsnpPublicKey],
) -> Option<JByteArray<'local>> {
	let mut proto_keys = vec![];
	for k in public_keys {
		proto_keys.push(proto_output::dsnp_public_keys::DsnpPublicKey {
			key: k.key.clone(),
			key_id: k.key_id?,
			special_fields: SpecialFields::default(),
		});
	}
	let all_keys = proto_output::DsnpPublicKeys {
		public_key: proto_keys,
		special_fields: SpecialFields::default(),
	};

	let bytes = all_keys.write_to_bytes().ok()?;
	let arr = env.byte_array_from_slice(&bytes).ok()?;
	Some(arr)
}

pub fn serialize_graph_edges<'local>(
	env: &JNIEnv<'local>,
	graph_edge: &[RustDsnpGraphEdge],
) -> Option<JByteArray<'local>> {
	let mut proto_edge = vec![];
	for e in graph_edge {
		proto_edge.push(proto_output::dsnp_graph_edges::DsnpGraphEdge {
			user_id: e.user_id,
			since: e.since,
			special_fields: SpecialFields::default(),
		});
	}
	let all_edges =
		proto_output::DsnpGraphEdges { edge: proto_edge, special_fields: SpecialFields::default() };

	let bytes = all_edges.write_to_bytes().ok()?;
	let arr = env.byte_array_from_slice(&bytes).ok()?;
	Some(arr)
}

pub fn serialize_graph_updates<'local>(
	env: &JNIEnv<'local>,
	updates: &[RustUpdate],
) -> Option<JByteArray<'local>> {
	let mut protos = vec![];
	for e in updates {
		protos.push(map_update_to_proto(e)?);
	}
	let all_updates =
		proto_output::Updates { update: protos, special_fields: SpecialFields::default() };

	let bytes = all_updates.write_to_bytes().ok()?;
	let arr = env.byte_array_from_slice(&bytes).ok()?;
	Some(arr)
}

pub fn serialize_config<'local>(
	env: &JNIEnv<'local>,
	config: &RustConfig,
) -> Option<JByteArray<'local>> {
	let proto = proto_output::Config {
		max_page_id: config.max_page_id,
		max_key_page_size_bytes: config.max_key_page_size_bytes,
		sdk_max_users_graph_size: config.sdk_max_users_graph_size,
		sdk_max_stale_friendship_days: config.sdk_max_stale_friendship_days,
		max_graph_page_size_bytes: config.max_graph_page_size_bytes,
		dsnp_versions: map_dsnp_versions_to_proto(&config.dsnp_versions)?,
		schema_map: map_schema_map_to_proto(&config.schema_map)?,
		special_fields: SpecialFields::default(),
	};

	let bytes = proto.write_to_bytes().ok()?;
	let arr = env.byte_array_from_slice(&bytes).ok()?;
	Some(arr)
}

pub fn serialize_dsnp_users<'local>(
	env: &JNIEnv<'local>,
	dsnp_users: &[DsnpUserId],
) -> Option<JByteArray<'local>> {
	let mut proto = vec![];
	for e in dsnp_users {
		proto.push(*e);
	}
	let users = proto_output::DsnpUsers { user: proto, special_fields: SpecialFields::default() };

	let bytes = users.write_to_bytes().ok()?;
	let arr = env.byte_array_from_slice(&bytes).ok()?;
	Some(arr)
}

pub fn convert_jboolean(b: jboolean) -> Option<bool> {
	match b {
		JNI_FALSE => Some(false),
		JNI_TRUE => Some(true),
		_ => None,
	}
}

fn map_action_to_rust(action: proto_input::actions::Action) -> Option<RustAction> {
	let unwrapped = action.inner?;
	Some(match unwrapped {
		proto_input::actions::action::Inner::AddKeyAction(add_key) => RustAction::AddGraphKey {
			owner_dsnp_user_id: add_key.owner_dsnp_user_id,
			new_public_key: add_key.new_public_key,
		},
		proto_input::actions::action::Inner::ConnectAction(connect) => RustAction::Connect {
			owner_dsnp_user_id: connect.owner_dsnp_user_id,
			connection: map_connection_to_rust(connect.connection.as_ref()?)?,
			dsnp_keys: match connect.dsnp_keys.as_ref() {
				Some(k) => Some(map_dsnp_keys_to_rust(k)?),
				None => None,
			},
		},
		proto_input::actions::action::Inner::DisconnectAction(disconnect) =>
			RustAction::Disconnect {
				owner_dsnp_user_id: disconnect.owner_dsnp_user_id,
				connection: map_connection_to_rust(disconnect.connection.as_ref()?)?,
			},
		_ => return None,
	})
}

fn map_connection_to_rust(conection: &proto_input::Connection) -> Option<RustConnection> {
	Some(RustConnection {
		dsnp_user_id: conection.dsnp_user_id,
		schema_id: SchemaId::try_from(conection.schema_id).ok()?,
	})
}

fn map_dsnp_keys_to_rust(dsnp_keys: &proto_input::DsnpKeys) -> Option<RustDsnpKeys> {
	Some(RustDsnpKeys {
		dsnp_user_id: dsnp_keys.dsnp_user_id,
		keys_hash: dsnp_keys.keys_hash,
		keys: map_key_data_to_rust(&dsnp_keys.keys)?,
	})
}

fn map_key_data_to_rust(key_datas: &Vec<proto_input::KeyData>) -> Option<Vec<RustKeyData>> {
	let mut keys = vec![];
	for k in key_datas {
		keys.push(RustKeyData { content: k.content.clone(), index: u16::try_from(k.index).ok()? });
	}
	Some(keys)
}

fn map_config_to_rust(config: proto_output::Config) -> Option<RustConfig> {
	let mut dsnp_versions = vec![];
	for version in config.dsnp_versions.into_iter() {
		dsnp_versions.push(map_dsnp_version_to_rust(version.enum_value().ok()?)?);
	}

	let mut schema_map = HashMap::<SchemaId, RustSchemaConfig>::new();
	for (key, val) in config.schema_map.into_iter() {
		schema_map.insert(key.try_into().ok()?, map_schema_config_to_rust(val)?);
	}

	Some(RustConfig {
		max_graph_page_size_bytes: config.max_graph_page_size_bytes,
		sdk_max_stale_friendship_days: config.sdk_max_stale_friendship_days,
		max_page_id: config.max_page_id,
		sdk_max_users_graph_size: config.sdk_max_users_graph_size,
		max_key_page_size_bytes: config.max_key_page_size_bytes,
		dsnp_versions,
		schema_map,
	})
}

fn map_dsnp_version_to_rust(version: proto_output::DsnpVersion) -> Option<RustDsnpVersion> {
	let result = match version {
		proto_output::DsnpVersion::Version1_0 => RustDsnpVersion::Version1_0,
	};
	Some(result)
}

fn map_schema_config_to_rust(
	schema_config: proto_output::SchemaConfig,
) -> Option<RustSchemaConfig> {
	let version = schema_config.dsnp_version.enum_value().ok()?;
	let connection_type = schema_config.connection_type.enum_value().ok()?;

	let result = RustSchemaConfig {
		dsnp_version: map_dsnp_version_to_rust(version)?,
		connection_type: map_connection_type_to_rust(connection_type)?,
	};

	Some(result)
}

fn map_connection_type_to_rust(
	connection_type: proto_output::ConnectionType,
) -> Option<RustConnectionType> {
	Some(match connection_type {
		proto_output::ConnectionType::FollowPrivate =>
			RustConnectionType::Follow(RustPrivacyType::Private),
		proto_output::ConnectionType::FollowPublic =>
			RustConnectionType::Follow(RustPrivacyType::Public),
		proto_output::ConnectionType::FriendshipPrivate =>
			RustConnectionType::Friendship(RustPrivacyType::Private),
		proto_output::ConnectionType::FriendshipPublic =>
			RustConnectionType::Friendship(RustPrivacyType::Public),
	})
}

fn map_update_to_proto(update: &RustUpdate) -> Option<proto_output::updates::Update> {
	let mut proto = proto_output::updates::Update::new();
	let inner = match update {
		RustUpdate::PersistPage { schema_id, page_id, prev_hash, owner_dsnp_user_id, payload } =>
			proto_output::updates::update::Inner::Persist(PersistPageUpdate {
				owner_dsnp_user_id: *owner_dsnp_user_id,
				prev_hash: *prev_hash,
				page_id: u32::try_from(*page_id).ok()?,
				schema_id: u32::try_from(*schema_id).ok()?,
				payload: payload.clone(),
				special_fields: SpecialFields::default(),
			}),
		RustUpdate::DeletePage { schema_id, page_id, prev_hash, owner_dsnp_user_id } =>
			proto_output::updates::update::Inner::Delete(DeletePageUpdate {
				owner_dsnp_user_id: *owner_dsnp_user_id,
				prev_hash: *prev_hash,
				page_id: u32::try_from(*page_id).ok()?,
				schema_id: u32::try_from(*schema_id).ok()?,
				special_fields: SpecialFields::default(),
			}),
		RustUpdate::AddKey { prev_hash, owner_dsnp_user_id, payload } =>
			proto_output::updates::update::Inner::AddKey(AddKeyUpdate {
				owner_dsnp_user_id: *owner_dsnp_user_id,
				prev_hash: *prev_hash,
				payload: payload.clone(),
				special_fields: SpecialFields::default(),
			}),
	};
	proto.inner = Some(inner);
	Some(proto)
}

fn map_graph_key_pairs_to_rust(
	key_pairs: &[proto_input::import_bundles::import_bundle::GraphKeyPair],
) -> Option<Vec<RustGraphKeyPair>> {
	let mut result = vec![];
	for p in key_pairs {
		result.push(RustGraphKeyPair {
			public_key: p.public_key.clone(),
			secret_key: p.secret_key.clone(),
			key_type: map_graph_key_type_to_rust(p.key_type.enum_value().ok()?)?,
		})
	}
	Some(result)
}

fn map_graph_key_type_to_rust(key_type: proto_input::GraphKeyType) -> Option<RustGraphKeyType> {
	Some(match key_type {
		proto_input::GraphKeyType::X25519 => RustGraphKeyType::X25519,
	})
}

fn map_page_datas_to_rust(pages: &[proto_input::PageData]) -> Option<Vec<RustPageData>> {
	let mut result = vec![];
	for p in pages {
		result.push(RustPageData {
			page_id: PageId::try_from(p.page_id).ok()?,
			content_hash: p.content_hash,
			content: p.content.clone(),
		})
	}
	Some(result)
}

fn map_dsnp_versions_to_proto(
	versions: &Vec<RustDsnpVersion>,
) -> Option<Vec<EnumOrUnknown<proto_output::DsnpVersion>>> {
	let mut result = vec![];
	for v in versions {
		result.push(map_dsnp_version_to_proto(v)?);
	}
	Some(result)
}

fn map_dsnp_version_to_proto(
	version: &RustDsnpVersion,
) -> Option<EnumOrUnknown<proto_output::DsnpVersion>> {
	Some(match version {
		RustDsnpVersion::Version1_0 => EnumOrUnknown::new(proto_output::DsnpVersion::Version1_0),
	})
}

fn map_connection_type_to_proto(
	connection_type: &RustConnectionType,
) -> Option<EnumOrUnknown<proto_output::ConnectionType>> {
	Some(match connection_type {
		RustConnectionType::Friendship(RustPrivacyType::Private) =>
			EnumOrUnknown::new(proto_output::ConnectionType::FriendshipPrivate),
		RustConnectionType::Friendship(RustPrivacyType::Public) =>
			EnumOrUnknown::new(proto_output::ConnectionType::FriendshipPublic),
		RustConnectionType::Follow(RustPrivacyType::Private) =>
			EnumOrUnknown::new(proto_output::ConnectionType::FollowPrivate),
		RustConnectionType::Follow(RustPrivacyType::Public) =>
			EnumOrUnknown::new(proto_output::ConnectionType::FollowPublic),
	})
}

fn map_schema_map_to_proto(
	map: &HashMap<SchemaId, SchemaConfig>,
) -> Option<HashMap<u32, proto_output::SchemaConfig>> {
	let mut result = HashMap::new();
	for (k, v) in map {
		result.insert(u32::try_from(*k).ok()?, map_schema_config_to_proto(v)?);
	}
	Some(result)
}

fn map_schema_config_to_proto(schema_config: &SchemaConfig) -> Option<proto_output::SchemaConfig> {
	Some(proto_output::SchemaConfig {
		dsnp_version: map_dsnp_version_to_proto(&schema_config.dsnp_version)?,
		connection_type: map_connection_type_to_proto(&schema_config.connection_type)?,
		special_fields: SpecialFields::default(),
	})
}
