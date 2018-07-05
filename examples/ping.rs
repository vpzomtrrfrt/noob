extern crate futures;
extern crate noob;
extern crate tokio_core;

use futures::{Future, Stream};

fn main() {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    let token = std::env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN");

    let task = noob::Client::connect(&token).and_then(|(client, stream)| {
        stream.for_each(move |evt| {
            println!("event: {:?}", evt);
            if let noob::Event::MessageCreate(msg) = evt {
                println!("msg! {}", msg.content);
                if msg.content == "ping" {
                    handle.spawn(
                        client
                            .send_message(&noob::MessageBuilder::new("pong"), &msg.channel_id)
                            .map_err(|e| {
                                eprintln!("{:?}", e);
                            }),
                    );
                }
            }
            Ok(())
        })
    });

    core.run(task).unwrap();
}
