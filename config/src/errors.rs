//! Errors for graph-sdk crate
//!

use thiserror::Error;

pub type Result<T> = std::result::Result<T, DsnpGraphError>;

/// Errors for graph-sdk crate
#[derive(Debug, Error)]
pub enum DsnpGraphError {}
