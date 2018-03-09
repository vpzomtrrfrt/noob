#[macro_use] extern crate quick_error;
#[macro_use] extern crate serde_derive;
extern crate hyper;
extern crate hyper_tls;
extern crate futures;
extern crate tokio_core;
extern crate serde_json;
extern crate websocket;

macro_rules! fut_try(
    ($e:expr) => (match $e
                  {
                      Ok(e) => e,
                      Err(err) => return futures::future::err(err)
                  })
    );

macro_rules! box_fut_try(
    ($e:expr) => (match $e
                  {
                      Ok(e) => e,
                      Err(err) => return Box::new(futures::future::err(err))
                  })
    );

pub mod client;
pub mod events;
pub mod error;
