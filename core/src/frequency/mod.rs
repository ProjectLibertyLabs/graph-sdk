pub mod reader_writer;

use dsnp_graph_config::{Config, MAINNET_CONFIG, ROCOCO_CONFIG};

#[allow(dead_code)] // todo: remove
/// A utility to read/write data from and to Frequency chain specific implementation of DSNP
pub struct Frequency<'a> {
	config: &'a Config,
}

impl<'a> Frequency<'a> {
	pub fn init(config: &'a Config) -> Self {
		Frequency { config }
	}

	pub fn with_mainnet() -> Result<Self, ()> {
		Ok(Self::init(&MAINNET_CONFIG))
	}

	pub fn with_rococo() -> Result<Self, ()> {
		Ok(Self::init(&ROCOCO_CONFIG))
	}
}
