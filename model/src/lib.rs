//! # RaidProtect model
//!
//! This crate contains shared models between other workspace crates and
//! database connection wrappers.

pub mod collection;
mod state;

pub use state::ClusterState;
