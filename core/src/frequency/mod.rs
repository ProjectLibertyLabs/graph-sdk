pub mod reader_writer;

use dsnp_graph_config::{Config, MAINNET_CONFIG, ROCOCO_CONFIG};

#[allow(dead_code)] // todo: remove
/// A utility to read/write data from and to Frequency chain specific implementation of DSNP
pub struct Frequency {
	config: Config,
}

impl Frequency {
	pub fn init(config: Config) -> Self {
		Frequency { config }
	}

	pub fn with_mainnet() -> Result<Self, ()> {
		Ok(Self::init(MAINNET_CONFIG.try_into().map_err(|_| ())?))
	}

	pub fn with_rococo() -> Result<Self, ()> {
		Ok(Self::init(ROCOCO_CONFIG.try_into().map_err(|_| ())?))
	}
}
