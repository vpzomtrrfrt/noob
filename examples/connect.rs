extern crate noob;
extern crate tokio_core;
extern crate futures;

use futures::{Future, Stream};

fn main() {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
    let token = std::env::var("DISCORD_TOKEN")
        .expect("Missing DISCORD_TOKEN");

    let task = noob::Client::connect(&token)
        .and_then(|(client, stream)| {
            stream.for_each(|evt| {
                println!("{:?}", evt);
                Ok(())
            })
        });
    core.run(task).unwrap();
}
