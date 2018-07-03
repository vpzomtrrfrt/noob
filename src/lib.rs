extern crate hyper;
extern crate hyper_tls;
extern crate futures;
extern crate websocket;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate try_future;
#[macro_use] extern crate quick_error;

mod client;
mod error;
pub mod events;

pub use client::Client;
pub use error::Error;
pub use events::Event;

#[derive(Deserialize, Serialize)]
struct DiscordBasePayload<I> {
    pub op: u8,
    pub d: I
}
