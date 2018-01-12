extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate native_tls;
extern crate tokio_core;
extern crate websocket;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;

macro_rules! fut_try(
    ($e:expr) => (match $e
                  {
                      Ok(e) => e,
                      Err(err) => return Box::new(futures::future::err(err))
                  })
    );

mod client;
pub use client::Client;
