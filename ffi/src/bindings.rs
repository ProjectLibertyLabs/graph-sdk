use dsnp_graph_config::{Config, ConnectionType, Environment};
use dsnp_graph_core::{api::api::GraphState as InnerGraphState, dsnp::api_types::PrivacyType};

#[repr(C)]
pub enum CEnvironment {
	Mainnet,
	Rococo,
	Dev(Box<Config>),
}

#[repr(C)]
pub enum CPrivacyType {
	Public,
	Private,
}

impl From<CPrivacyType> for PrivacyType {
	fn from(privacy: CPrivacyType) -> Self {
		match privacy {
			CPrivacyType::Public => PrivacyType::Public,
			CPrivacyType::Private => PrivacyType::Private,
		}
	}
}

impl From<PrivacyType> for CPrivacyType {
	fn from(privacy: PrivacyType) -> Self {
		match privacy {
			PrivacyType::Public => CPrivacyType::Public,
			PrivacyType::Private => CPrivacyType::Private,
		}
	}
}

impl From<CEnvironment> for Environment {
	fn from(env: CEnvironment) -> Self {
		match env {
			CEnvironment::Mainnet => Environment::Mainnet,
			CEnvironment::Rococo => Environment::Rococo,
			CEnvironment::Dev(cfg) => Environment::Dev(*cfg),
		}
	}
}

impl From<Environment> for CEnvironment {
	fn from(env: Environment) -> Self {
		match env {
			Environment::Mainnet => CEnvironment::Mainnet,
			Environment::Rococo => CEnvironment::Rococo,
			Environment::Dev(cfg) => CEnvironment::Dev(Box::new(cfg)),
		}
	}
}

#[repr(C)]
pub enum CConnectionType {
	Follow(PrivacyType),
	Friendship(PrivacyType),
}

impl From<CConnectionType> for ConnectionType {
	fn from(connection: CConnectionType) -> Self {
		match connection {
			CConnectionType::Follow(privacy_type) => ConnectionType::Follow(privacy_type),
			CConnectionType::Friendship(privacy_type) => ConnectionType::Friendship(privacy_type),
		}
	}
}

impl From<ConnectionType> for CConnectionType {
	fn from(connection: ConnectionType) -> Self {
		match connection {
			ConnectionType::Follow(privacy_type) => CConnectionType::Follow(privacy_type),
			ConnectionType::Friendship(privacy_type) => CConnectionType::Friendship(privacy_type),
		}
	}
}

#[repr(C)]
pub struct GraphState {
	// the inner graph state
	pub inner: Box<InnerGraphState>,
}
