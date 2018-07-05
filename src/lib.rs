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

pub mod builder;
mod client;
mod error;
pub mod events;

pub use builder::{EmbedBuilder, MessageBuilder};
pub use client::Client;
pub use error::Error;
pub use events::Event;

#[derive(Deserialize, Serialize)]
struct DiscordBasePayload<I> {
    pub op: u8,
    pub d: I,
}
