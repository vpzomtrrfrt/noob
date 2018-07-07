extern crate futures;
extern crate noob;
extern crate tokio;

use futures::{Future, Stream};

fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN");

    tokio::run(
        noob::Client::connect(&token)
            .and_then(|(client, stream)| {
                stream.for_each(move |evt| {
                    println!("event: {:?}", evt);
                    if let noob::Event::MessageCreate(msg) = evt {
                        println!("msg! {}", msg.content);
                        if msg.content == "ping" {
                            tokio::spawn(
                                client
                                    .send_message(
                                        &noob::MessageBuilder::new("pong"),
                                        &msg.channel_id,
                                    )
                                    .map_err(|e| {
                                        eprintln!("{:?}", e);
                                    }),
                            );
                        }
                    }
                    Ok(())
                })
            })
            .map_err(|e| {
                eprintln!("{:?}", e);
            }),
    );
}
