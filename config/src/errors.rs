//! Errors for graph-sdk crate
//!

use thiserror::Error;

pub type DsnpGraphResult<T> = std::result::Result<T, DsnpGraphError>;

#[derive(Debug, Error)]
pub enum DsnpGraphError {
	#[error("User graph for {0} is not imported")]
	UserGraphNotImported(String),
}
