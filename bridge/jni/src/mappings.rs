use crate::{
	api::SdkJniResult,
	errors::{SdkJniError, SdkJniError::InvalidRequest},
};
use dsnp_graph_config::{
	Config as RustConfig, DsnpUserId, DsnpVersion as RustDsnpVersion,
	Environment as RustEnvironment, GraphKeyType as RustGraphKeyType, PageId,
	SchemaConfig as RustSchemaConfig, SchemaConfig, SchemaId,
};
use dsnp_graph_core::{
	api::api_types::{
		Action as RustAction, ActionOptions as RustActionOptions, Connection as RustConnection,
		ConnectionType as RustConnectionType, DsnpKeys as RustDsnpKeys,
		GraphKeyPair as RustGraphKeyPair, ImportBundle as RustImportBundle, KeyData as RustKeyData,
		PageData as RustPageData, PrivacyType as RustPrivacyType, Update as RustUpdate,
	},
	dsnp::dsnp_types::{DsnpGraphEdge as RustDsnpGraphEdge, DsnpPublicKey as RustDsnpPublicKey},
};
use dsnp_graph_sdk_common::proto_types::{
	input::{self as proto_input},
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

pub fn map_to_environment(
	env: &JNIEnv<'_>,
	environment: &JByteArray,
) -> SdkJniResult<RustEnvironment> {
	let bytes = env.convert_byte_array(environment).map_err(|e| SdkJniError::from(e))?;
	let env_proto =
		proto_output::Environment::parse_from_bytes(&bytes).map_err(|e| SdkJniError::from(e))?;
	let result = match env_proto
		.environment_type
		.enum_value()
		.map_err(|_| SdkJniError::InvalidRequest("environment_type not set!"))?
	{
		proto_output::EnvironmentType::MainNet => RustEnvironment::Mainnet,
		proto_output::EnvironmentType::Rococo => RustEnvironment::Rococo,
		proto_output::EnvironmentType::TestnetPaseo => RustEnvironment::TestnetPaseo,
		proto_output::EnvironmentType::Dev => {
			let cfg = env_proto
				.config
				.into_option()
				.ok_or(SdkJniError::InvalidRequest("config not set!"))?;
			let rust_cfg = map_config_to_rust(cfg)?;
			RustEnvironment::Dev(rust_cfg)
		},
	};
	Ok(result)
}

pub fn map_to_actions(
	env: &JNIEnv<'_>,
	actions: &JByteArray,
) -> SdkJniResult<(Vec<RustAction>, Option<RustActionOptions>)> {
	let bytes = env.convert_byte_array(actions).map_err(|e| SdkJniError::from(e))?;
	let actions_proto =
		proto_input::Actions::parse_from_bytes(&bytes).map_err(|e| SdkJniError::from(e))?;

	let mut result = vec![];
	for a in actions_proto.actions {
		result.push(map_action_to_rust(a)?);
	}
	let options = match actions_proto.options.into_option() {
		Some(options) => Some(RustActionOptions {
			ignore_existing_connections: options.ignore_existing_connections,
			ignore_missing_connections: options.ignore_missing_connections,
			disable_auto_commit: options.disable_auto_commit,
		}),
		None => None,
	};
	Ok((result, options))
}

pub fn map_to_imports(
	env: &JNIEnv<'_>,
	imports: &JByteArray,
) -> SdkJniResult<Vec<RustImportBundle>> {
	let bytes = env.convert_byte_array(imports).map_err(|e| SdkJniError::from(e))?;
	let imports_proto =
		proto_input::ImportBundles::parse_from_bytes(&bytes).map_err(|e| SdkJniError::from(e))?;
	let mut result = vec![];
	for i in imports_proto.bundles {
		result.push(RustImportBundle {
			schema_id: SchemaId::try_from(i.schema_id)
				.map_err(|_| SdkJniError::UnexpectedResponse("invalid SchemaId"))?,
			dsnp_user_id: i.dsnp_user_id,
			dsnp_keys: map_dsnp_keys_to_rust(&i.dsnp_keys.into_option())?,
			key_pairs: map_graph_key_pairs_to_rust(&i.key_pairs)?,
			pages: map_page_datas_to_rust(&i.pages)?,
		});
	}
	Ok(result)
}

pub fn map_to_dsnp_keys(
	env: &JNIEnv<'_>,
	dsnp_keys: &JByteArray,
) -> SdkJniResult<Option<RustDsnpKeys>> {
	let bytes = env.convert_byte_array(dsnp_keys).map_err(|e| SdkJniError::from(e))?;
	let dsnp_keys_proto =
		proto_input::DsnpKeys::parse_from_bytes(&bytes).map_err(|e| SdkJniError::from(e))?;
	map_dsnp_keys_to_rust(&Some(dsnp_keys_proto))
}

pub fn serialize_graph_keypair<'local>(
	env: &JNIEnv<'local>,
	key_pair: &dsnp_graph_core::api::api_types::GraphKeyPair,
) -> SdkJniResult<JByteArray<'local>> {
	let proto = proto_input::import_bundles::import_bundle::GraphKeyPair {
		public_key: key_pair.public_key.clone(),
		secret_key: key_pair.secret_key.clone(),
		key_type: match key_pair.key_type {
			RustGraphKeyType::X25519 => proto_input::GraphKeyType::X25519.into(),
		},
		special_fields: SpecialFields::default(),
	};

	let bytes = proto.write_to_bytes().map_err(|e| SdkJniError::from(e))?;
	let arr = env.byte_array_from_slice(&bytes).map_err(|e| SdkJniError::from(e))?;
	Ok(arr)
}

pub fn serialize_public_keys<'local>(
	env: &JNIEnv<'local>,
	public_keys: &[RustDsnpPublicKey],
) -> SdkJniResult<JByteArray<'local>> {
	let mut proto_keys = vec![];
	for k in public_keys {
		proto_keys.push(proto_output::dsnp_public_keys::DsnpPublicKey {
			key: k.key.clone(),
			key_id: k.key_id.ok_or(SdkJniError::UnexpectedResponse("key_id is not set"))?,
			special_fields: SpecialFields::default(),
		});
	}
	let all_keys = proto_output::DsnpPublicKeys {
		public_key: proto_keys,
		special_fields: SpecialFields::default(),
	};

	let bytes = all_keys.write_to_bytes().map_err(|e| SdkJniError::from(e))?;
	let arr = env.byte_array_from_slice(&bytes).map_err(|e| SdkJniError::from(e))?;
	Ok(arr)
}

pub fn serialize_graph_edges<'local>(
	env: &JNIEnv<'local>,
	graph_edge: &[RustDsnpGraphEdge],
) -> SdkJniResult<JByteArray<'local>> {
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

	let bytes = all_edges.write_to_bytes().map_err(|e| SdkJniError::from(e))?;
	let arr = env.byte_array_from_slice(&bytes).map_err(|e| SdkJniError::from(e))?;
	Ok(arr)
}

pub fn serialize_graph_updates<'local>(
	env: &JNIEnv<'local>,
	updates: &[RustUpdate],
) -> SdkJniResult<JByteArray<'local>> {
	let mut protos = vec![];
	for e in updates {
		protos.push(map_update_to_proto(e)?);
	}
	let all_updates =
		proto_output::Updates { update: protos, special_fields: SpecialFields::default() };

	let bytes = all_updates.write_to_bytes().map_err(|e| SdkJniError::from(e))?;
	let arr = env.byte_array_from_slice(&bytes).map_err(|e| SdkJniError::from(e))?;
	Ok(arr)
}

pub fn serialize_config<'local>(
	env: &JNIEnv<'local>,
	config: &RustConfig,
) -> SdkJniResult<JByteArray<'local>> {
	let proto = proto_output::Config {
		max_page_id: config.max_page_id,
		max_key_page_size_bytes: config.max_key_page_size_bytes,
		sdk_max_stale_friendship_days: config.sdk_max_stale_friendship_days,
		max_graph_page_size_bytes: config.max_graph_page_size_bytes,
		dsnp_versions: map_dsnp_versions_to_proto(&config.dsnp_versions)?,
		schema_map: map_schema_map_to_proto(&config.schema_map)?,
		graph_public_key_schema_id: u32::try_from(config.graph_public_key_schema_id)
			.map_err(|_| SdkJniError::UnexpectedResponse("invalid SchemaId"))?,
		special_fields: SpecialFields::default(),
	};

	let bytes = proto.write_to_bytes().map_err(|e| SdkJniError::from(e))?;
	let arr = env.byte_array_from_slice(&bytes).map_err(|e| SdkJniError::from(e))?;
	Ok(arr)
}

pub fn serialize_dsnp_users<'local>(
	env: &JNIEnv<'local>,
	dsnp_users: &[DsnpUserId],
) -> SdkJniResult<JByteArray<'local>> {
	let mut proto = vec![];
	for e in dsnp_users {
		proto.push(*e);
	}
	let users = proto_output::DsnpUsers { user: proto, special_fields: SpecialFields::default() };

	let bytes = users.write_to_bytes().map_err(|e| SdkJniError::from(e))?;
	let arr = env.byte_array_from_slice(&bytes).map_err(|e| SdkJniError::from(e))?;
	Ok(arr)
}

pub fn convert_jboolean(b: jboolean) -> SdkJniResult<bool> {
	match b {
		JNI_FALSE => Ok(false),
		JNI_TRUE => Ok(true),
		_ => SdkJniResult::Err(SdkJniError::BadJniParameter(" invalid boolean")),
	}
}

fn map_action_to_rust(action: proto_input::actions::Action) -> SdkJniResult<RustAction> {
	let inner = action.inner.ok_or(SdkJniError::InvalidRequest("action not set!"))?;
	Ok(match inner {
		proto_input::actions::action::Inner::AddKeyAction(add_key) => RustAction::AddGraphKey {
			owner_dsnp_user_id: add_key.owner_dsnp_user_id,
			new_public_key: add_key.new_public_key,
		},
		proto_input::actions::action::Inner::ConnectAction(connect) => RustAction::Connect {
			owner_dsnp_user_id: connect.owner_dsnp_user_id,
			connection: map_connection_to_rust(
				&connect
					.connection
					.into_option()
					.ok_or(SdkJniError::InvalidRequest("connection not set!"))?,
			)?,
			dsnp_keys: map_dsnp_keys_to_rust(&connect.dsnp_keys.as_ref().cloned())?,
		},
		proto_input::actions::action::Inner::DisconnectAction(disconnect) =>
			RustAction::Disconnect {
				owner_dsnp_user_id: disconnect.owner_dsnp_user_id,
				connection: map_connection_to_rust(
					&disconnect
						.connection
						.into_option()
						.ok_or(SdkJniError::InvalidRequest("connection not set!"))?,
				)?,
			},
		_ => return SdkJniResult::Err(InvalidRequest("invalid action type!")),
	})
}

fn map_connection_to_rust(conection: &proto_input::Connection) -> SdkJniResult<RustConnection> {
	Ok(RustConnection {
		dsnp_user_id: conection.dsnp_user_id,
		schema_id: SchemaId::try_from(conection.schema_id)
			.map_err(|_| SdkJniError::InvalidRequest("invalid SchemaId"))?,
	})
}

fn map_dsnp_keys_to_rust(
	dsnp_keys: &Option<proto_input::DsnpKeys>,
) -> SdkJniResult<Option<RustDsnpKeys>> {
	match dsnp_keys {
		Some(keys) => Ok(Some(RustDsnpKeys {
			dsnp_user_id: keys.dsnp_user_id,
			keys_hash: keys.keys_hash,
			keys: map_key_data_to_rust(&keys.keys)?,
		})),
		None => Ok(None),
	}
}

fn map_key_data_to_rust(key_datas: &Vec<proto_input::KeyData>) -> SdkJniResult<Vec<RustKeyData>> {
	let mut keys = vec![];
	for k in key_datas {
		keys.push(RustKeyData {
			content: k.content.clone(),
			index: u16::try_from(k.index)
				.map_err(|_| SdkJniError::InvalidRequest("invalid key index"))?,
		});
	}
	Ok(keys)
}

fn map_config_to_rust(config: proto_output::Config) -> SdkJniResult<RustConfig> {
	let mut dsnp_versions = vec![];
	for version in config.dsnp_versions.into_iter() {
		dsnp_versions.push(map_dsnp_version_to_rust(
			version
				.enum_value()
				.map_err(|_| SdkJniError::InvalidRequest("version not set!"))?,
		)?);
	}

	let mut schema_map = HashMap::<SchemaId, RustSchemaConfig>::new();
	for (key, val) in config.schema_map.into_iter() {
		schema_map.insert(
			SchemaId::try_from(key).map_err(|_| SdkJniError::InvalidRequest("invalid SchemaId"))?,
			map_schema_config_to_rust(val)?,
		);
	}

	Ok(RustConfig {
		max_graph_page_size_bytes: config.max_graph_page_size_bytes,
		sdk_max_stale_friendship_days: config.sdk_max_stale_friendship_days,
		max_page_id: config.max_page_id,
		max_key_page_size_bytes: config.max_key_page_size_bytes,
		dsnp_versions,
		schema_map,
		graph_public_key_schema_id: SchemaId::try_from(config.graph_public_key_schema_id)
			.map_err(|_| SdkJniError::InvalidRequest("invalid SchemaId"))?,
	})
}

fn map_dsnp_version_to_rust(version: proto_output::DsnpVersion) -> SdkJniResult<RustDsnpVersion> {
	let result = match version {
		proto_output::DsnpVersion::Version1_0 => RustDsnpVersion::Version1_0,
	};
	Ok(result)
}

fn map_schema_config_to_rust(
	schema_config: proto_output::SchemaConfig,
) -> SdkJniResult<RustSchemaConfig> {
	let version = schema_config
		.dsnp_version
		.enum_value()
		.map_err(|_| SdkJniError::InvalidRequest("dsnp_version not set!"))?;
	let connection_type = schema_config
		.connection_type
		.enum_value()
		.map_err(|_| SdkJniError::InvalidRequest("connection_type not set!"))?;

	let result = RustSchemaConfig {
		dsnp_version: map_dsnp_version_to_rust(version)?,
		connection_type: map_connection_type_to_rust(connection_type)?,
	};

	Ok(result)
}

fn map_connection_type_to_rust(
	connection_type: proto_output::ConnectionType,
) -> SdkJniResult<RustConnectionType> {
	Ok(match connection_type {
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

fn map_update_to_proto(update: &RustUpdate) -> SdkJniResult<proto_output::updates::Update> {
	let mut proto = proto_output::updates::Update::new();
	let inner = match update {
		RustUpdate::PersistPage { schema_id, page_id, prev_hash, owner_dsnp_user_id, payload } =>
			proto_output::updates::update::Inner::Persist(PersistPageUpdate {
				owner_dsnp_user_id: *owner_dsnp_user_id,
				prev_hash: *prev_hash,
				page_id: u32::try_from(*page_id)
					.map_err(|_| SdkJniError::InvalidRequest("invalid PageId"))?,
				schema_id: u32::try_from(*schema_id)
					.map_err(|_| SdkJniError::InvalidRequest("invalid SchemaId"))?,
				payload: payload.clone(),
				special_fields: SpecialFields::default(),
			}),
		RustUpdate::DeletePage { schema_id, page_id, prev_hash, owner_dsnp_user_id } =>
			proto_output::updates::update::Inner::Delete(DeletePageUpdate {
				owner_dsnp_user_id: *owner_dsnp_user_id,
				prev_hash: *prev_hash,
				page_id: u32::try_from(*page_id)
					.map_err(|_| SdkJniError::InvalidRequest("invalid PageId"))?,
				schema_id: u32::try_from(*schema_id)
					.map_err(|_| SdkJniError::InvalidRequest("invalid SchemaId"))?,
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
	Ok(proto)
}

fn map_graph_key_pairs_to_rust(
	key_pairs: &[proto_input::import_bundles::import_bundle::GraphKeyPair],
) -> SdkJniResult<Vec<RustGraphKeyPair>> {
	let mut result = vec![];
	for p in key_pairs {
		result.push(RustGraphKeyPair {
			public_key: p.public_key.clone(),
			secret_key: p.secret_key.clone(),
			key_type: map_graph_key_type_to_rust(
				p.key_type
					.enum_value()
					.map_err(|_| SdkJniError::InvalidRequest("key_type not set!"))?,
			)?,
		})
	}
	Ok(result)
}

fn map_graph_key_type_to_rust(
	key_type: proto_input::GraphKeyType,
) -> SdkJniResult<RustGraphKeyType> {
	Ok(match key_type {
		proto_input::GraphKeyType::X25519 => RustGraphKeyType::X25519,
	})
}

fn map_page_datas_to_rust(pages: &[proto_input::PageData]) -> SdkJniResult<Vec<RustPageData>> {
	let mut result = vec![];
	for p in pages {
		result.push(RustPageData {
			page_id: PageId::try_from(p.page_id)
				.map_err(|_| SdkJniError::InvalidRequest("invalid PageId"))?,
			content_hash: p.content_hash,
			content: p.content.clone(),
		})
	}
	Ok(result)
}

fn map_dsnp_versions_to_proto(
	versions: &Vec<RustDsnpVersion>,
) -> SdkJniResult<Vec<EnumOrUnknown<proto_output::DsnpVersion>>> {
	let mut result = vec![];
	for v in versions {
		result.push(map_dsnp_version_to_proto(v)?);
	}
	Ok(result)
}

fn map_dsnp_version_to_proto(
	version: &RustDsnpVersion,
) -> SdkJniResult<EnumOrUnknown<proto_output::DsnpVersion>> {
	Ok(match version {
		RustDsnpVersion::Version1_0 => EnumOrUnknown::new(proto_output::DsnpVersion::Version1_0),
	})
}

fn map_connection_type_to_proto(
	connection_type: &RustConnectionType,
) -> SdkJniResult<EnumOrUnknown<proto_output::ConnectionType>> {
	Ok(match connection_type {
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
) -> SdkJniResult<HashMap<u32, proto_output::SchemaConfig>> {
	let mut result = HashMap::new();
	for (k, v) in map {
		result.insert(
			u32::try_from(*k).map_err(|_| SdkJniError::UnexpectedResponse("invalid SchemaId"))?,
			map_schema_config_to_proto(v)?,
		);
	}
	Ok(result)
}

fn map_schema_config_to_proto(
	schema_config: &SchemaConfig,
) -> SdkJniResult<proto_output::SchemaConfig> {
	Ok(proto_output::SchemaConfig {
		dsnp_version: map_dsnp_version_to_proto(&schema_config.dsnp_version)?,
		connection_type: map_connection_type_to_proto(&schema_config.connection_type)?,
		special_fields: SpecialFields::default(),
	})
}
