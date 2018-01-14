extern crate tokio_discord;
extern crate tokio_core;
extern crate futures;

use futures::future::Future;

fn main() {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
    let task = tokio_discord::Client::login_bot(
        &handle.clone(),
        &std::env::var("DISCORD_TOKEN")
        .expect("Missing DISCORD_TOKEN"),
        Box::new(move |evt, interface| {
            println!("Event received: {:?}", evt);
            match evt {
                tokio_discord::events::Event::MessageCreate(ref msg) => {
                    println!("ok, now what?");
                    handle.spawn(interface.create_message(msg.channel_id.clone(), "Hello from tokio!")
                        .send()
                        .map_err(|e| {
                            println!("what {}", e);
                            panic!("wut");
                        }))
                }
                _ => {}
            }
        })
        )
        .and_then(|client|client.run());
    core.run(task).unwrap();
}
