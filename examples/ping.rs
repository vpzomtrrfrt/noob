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
        move |evt, client| {
            println!("event: {:?}", evt);
            if let noob::events::Event::MessageCreate(msg) = evt {
                println!("msg! {}", msg.content);
                if msg.content == "ping" {
                    handle.spawn(client.create_message("pong".to_owned())
                        .send(msg.channel_id)
                        .map_err(|e| {
                            eprintln!("{:?}", e);
                        }));
                }
            }
        });
    core.run(task).unwrap();
}
