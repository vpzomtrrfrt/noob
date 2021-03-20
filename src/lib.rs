//! Library for interacting with the Discord API and Gateway, especially for bots, using hyper/tokio.

#![warn(missing_docs)]

/// Objects for sending messages
pub mod builder;
mod client;
mod error;
/// Events and related objects
pub mod events;
mod types;

pub use builder::{EmbedBuilder, MessageBuilder};
pub use client::{Client, GatewayConnection, ListAnchor};
pub use error::Error;
pub use events::Event;
pub use types::*;

use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct DiscordBasePayload<I> {
    pub op: u8,
    pub d: I,
}
