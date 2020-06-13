use futures::{TryStreamExt};

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN");

    let (client, stream) = noob::Client::connect(token).await.expect("Failed to connect to Discord");
    let client = std::sync::Arc::new(client);
    let res = stream.try_for_each(move |evt| {
        println!("event: {:?}", evt);
        if let noob::Event::MessageCreate(msg) = evt {
            println!("msg! {}", msg.content);
            if msg.content == "ping" {
                let client = client.clone();
                tokio::spawn(async move {
                    let res = client
                        .send_message(
                            &noob::MessageBuilder::new("pong"),
                            &msg.channel_id,
                        ).await;
                    if let Err(e) = res {
                        eprintln!("{:?}", e);
                    }
                });
            }
        }
        futures::future::ready(Ok(()))
    }).await;

    if let Err(err) = res {
        eprintln!("{:?}", err);
    }
}
