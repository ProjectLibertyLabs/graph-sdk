//! Graph SDK
//!
//! Canonical implementation of social graph creation and evolution based on `DSNP` specification
//! and `Frequency` blockchain
//!
//! # Recommended Usage
//! To ensure local SDK state is in sync with the graph state on blockchain, it is recommended to
//! only initialize and use the library in case of needing to read the graph state or applying any
//! changes to the graph. This is the opposite of having a long living in-memory instance. On demand
//! initiation of SDK with the latest data, minimizes the probability of dealing with stale local state.
//!
pub mod api;
#[cfg(all(test, feature = "calculate-page-capacity"))]
mod benches;
pub mod dsnp;
pub mod frequency;
mod graph;
#[cfg(test)]
mod tests;
pub mod util;
