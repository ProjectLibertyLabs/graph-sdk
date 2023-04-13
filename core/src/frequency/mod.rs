pub mod config;
pub mod reader_writer;

use crate::dsnp::api_types::Config;

#[allow(dead_code)] // todo: remove
/// A utility to read/write data from and to Frequency chain specific implementation of DSNP
pub struct Frequency {
	config: Box<dyn Config>,
}

/// Macro to create a Frequency instance with the appropriate config
///
/// Usage:
/// ```ignore
/// frequency!(ConfigMain); // instantiate Frequency for Mainnet
/// frequency!(ConfigRococo); // instantiate Frequency for Rococo
/// frequency!(ConfigDev); // instantiate Frequency, using environment (see below)
/// frequency!(ConfigDev { [(ConnectionType::Follow(PrivacyType::Public), /* schema id */), ... ]}); // instantiate Frequency with code-defined schema id values
/// ```
#[macro_export]
macro_rules! frequency {
	(ConfigMain) => {
		Frequency { config: Box::new(ConfigMain {}) }
	};
	(ConfigRococo) => {
		Frequency { config: Box::new(ConfigRococo {}) }
	};
	(ConfigDev) => {
		Frequency { config: Box::new(ConfigDev::new(None)) }
	};
	(ConfigDev { $map_values:expr }) => {{
		let map = HashMap::from($map_values);
		Frequency { config: Box::new(ConfigDev::new(Some(map))) }
	}};
}
