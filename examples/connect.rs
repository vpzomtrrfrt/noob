extern crate noob;
extern crate tokio_core;
extern crate futures;

use futures::Future;

fn main() {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
    let task = noob::client::run_bot(
        &handle.clone(),
        &std::env::var("DISCORD_TOKEN")
        .expect("Missing DISCORD_TOKEN"),
        |evt| {
            println!("event: {:?}", evt);
        }
        )
        .and_then(|_| {
            println!("done");
            Ok(())
        });
    core.run(task).unwrap();
}
