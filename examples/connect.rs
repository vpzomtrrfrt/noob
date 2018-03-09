extern crate noob;
extern crate tokio_core;
extern crate futures;

use futures::Future;

fn main() {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
    let task = noob::client::Client::login_bot(
        &handle.clone(),
        &std::env::var("DISCORD_TOKEN")
        .expect("Missing DISCORD_TOKEN")
        )
        .and_then(|client| {
            println!("connected");
            Ok(())
        });
    core.run(task).unwrap();
}
