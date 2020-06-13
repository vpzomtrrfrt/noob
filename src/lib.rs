//! Library for interacting with the Discord API and Gateway, especially for bots, using hyper/tokio.

#![warn(missing_docs)]

/// Objects for sending messages
pub mod builder;
mod client;
mod error;
/// Events and related objects
pub mod events;

pub use builder::{EmbedBuilder, MessageBuilder};
pub use client::{Client, GatewayConnection};
pub use error::Error;
pub use events::Event;

use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct DiscordBasePayload<I> {
    pub op: u8,
    pub d: I,
}
