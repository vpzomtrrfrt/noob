extern crate tokio_discord;
extern crate tokio_core;
extern crate futures;

use futures::future::Future;

fn main() {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let task = tokio_discord::Client::login_bot(
        &core.handle(),
        &std::env::var("DISCORD_TOKEN")
        .expect("Missing DISCORD_TOKEN"),
        Box::new(|evt| {
            println!("Event received: {:?}", evt);
        })
        )
        .and_then(|client|client.run());
    core.run(task).unwrap();
}
