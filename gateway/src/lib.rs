//! # RaidProtect Gateway
//!
//! The gateway is the main component of the RaidProtect Discord bot.
//! It is responsible for the following :
//!
//! - Events from Discord are received and forwarded to appropriate services.
//! - An in-memory cache store information like guilds list and is exposed via
//!   RPC.
//! - The HTTP proxy provides global limiting for requests to the REST API.
//!
//! The exposed server is located in the [`server`] module.

mod cluster;

pub mod event;

pub use cluster::{ClusterError, ShardCluster};
