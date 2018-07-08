//! Library for interacting with the Discord API and Gateway, especially for bots, using hyper/tokio.

#![warn(missing_docs)]

extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio;
extern crate websocket;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate try_future;
#[macro_use]
extern crate quick_error;

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

#[derive(Deserialize, Serialize)]
struct DiscordBasePayload<I> {
    pub op: u8,
    pub d: I,
}
