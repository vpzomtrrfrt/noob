use Error;
use events;
use events::Event;

use std;
use futures;
use hyper;
use hyper_tls;
use serde_json;
use websocket;
use tokio;

use futures::{Future, Sink, Stream};
use tokio::executor::Executor;

pub struct Client {
    http_client: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
    token: String,
}

impl Client {
    pub fn connect(token: &str) -> Box<Future<Item = (Client, Stream<Item=Event, Error=Error>), Error = Error> + Send> {
        let http =
            hyper::Client::builder().build(try_future_box!(hyper_tls::HttpsConnector::new(1)));

        let gateway_req = try_future_box!(
            hyper::Request::get("https://discordapp.com/api/v6/gateway/bot")
                .header(hyper::header::AUTHORIZATION, token)
                .body(Default::default())
                .map_err(|e| Error::Other(format!("{:?}", e)))
        );

        let token = token.to_owned();

        Box::new(
            http.request(gateway_req)
                .map_err(|e| e.into())
                .and_then(|resp| -> Box<Future<Item = hyper::Chunk, Error = Error> + Send> {
                    match resp.status() {
                        hyper::StatusCode::UNAUTHORIZED => {
                            Box::new(futures::future::err(Error::AuthenticationFailed))
                        }
                        hyper::StatusCode::OK => {
                            Box::new(resp.into_body().concat2().map_err(|e| e.into()))
                        }
                        status => Box::new(futures::future::err(Error::Other(format!(
                            "Gateway request returned unexpected status {}",
                            status
                        )))),
                    }
                })
                .and_then(|body| -> Box<Future<Item=_, Error=Error> + Send> {
                    #[derive(Deserialize)]
                    struct GetGatewayResult<'a> {
                        url: &'a str,
                    }
                    let result: GetGatewayResult =
                        try_future_box!(serde_json::from_slice(&body).map_err(|e| Error::Other(
                            format!("Unable to parse Gateway API response: {:?}", e)
                        )));

                    println!("{}", result.url);
                    Box::new(try_future_box!(websocket::ClientBuilder::new(&result.url)
                                             .map_err(|e| Error::Other(format!("Failed to parse Gateway URI: {:?}", e))))
                             .async_connect(None, &Default::default())
                             .map_err(|e| e.into()))
                })
                .and_then(|(socket, _)| {
                    socket.into_future()
                        .map_err(|(e, _)| e.into())
                })
                .and_then(move |(msg1, socket)| -> Box<Future<Item=_, Error=Error> + Send> {
                    #[derive(Deserialize)]
                    struct Hello {
                        pub heartbeat_interval: u64
                    }

                    #[derive(Serialize)]
                    struct Identify<'a> {
                        token: &'a str,
                        properties: IdentifyProperties<'a>,
                        compress: bool
                    }

                    #[derive(Serialize)]
                    struct IdentifyProperties<'a> {
                        #[serde(rename = "$os")]
                        os: &'a str,
                        #[serde(rename = "$browser")]
                        browser: &'a str,
                        #[serde(rename = "$device")]
                        device: &'a str
                    }

                    if let Some(websocket::message::OwnedMessage::Text(text)) = msg1 {
                        let payload: ::DiscordBasePayload<Hello> = try_future_box!(
                            serde_json::from_str(&text)
                            .map_err(|e| Error::Other(format!("Failed to parse hello message: {:?}", e))));
                        let identify = try_future_box!(serde_json::to_string(&::DiscordBasePayload {
                            op: 2,
                            d: Identify {
                                token: &token,
                                properties: IdentifyProperties {
                                    os: "linux", // TODO make this work
                                    browser: "noob",
                                    device: "noob"
                                },
                                compress: false
                            }
                        }).map_err(|e| Error::Other(format!("Failed to serialize identify message: {:?}", e))));
                        Box::new(socket.send(websocket::message::OwnedMessage::Text(identify)).map_err(|e| e.into())
                                 .map(|socket| (socket, token, payload.d)))
                    }
                    else {
                        Box::new(futures::future::err(Error::Other(format!("Unexpected first message: {:?}", msg1))))
                    }
                })
                .and_then(|(socket, token, hello)| {
                    let (sink, stream) = socket.split();
                    let heartbeat_stream = tokio::timer::Interval::new(std::time::Instant::now(), std::time::Duration::from_millis(hello.heartbeat_interval));
                    tokio::executor::DefaultExecutor::current()
                        .spawn(Box::new(sink.send_all(heartbeat_stream
                                               .map_err(|e| -> websocket::WebSocketError {
                                                   panic!("Timer error: {:?}", e);
                                               })
                                               .map(|_| {
                                                   websocket::message::OwnedMessage::Text(json!({
                                                       "op": 1,
                                                       "d": null
                                                   }).to_string())
                                               })).map(|_|()).map_err(|e| {
                            eprintln!("Websocket error in heartbeat stream: {:?}", e);
                        })))
                    .map_err(|e| Error::Other(format!("Failed to spawn heartbeat stream: {:?}", e)))?;
                    Ok((Client {
                        http_client: http,
                        token,
                    }, stream.map_err(|e| e.into()).filter_map(handle_packet)))
                }),
        )
    }

    pub fn send_message(&self, msg: &::MessageBuilder, channel: &str) -> Box<Future<Item=(), Error=Error> + Send> {
        let body = try_future_box!(msg.to_request_body(channel));
        let auth_value = format!("Bot {}", self.token);
        let auth_value_ref: &str = &auth_value;
        let req = try_future_box!(hyper::Request::post(format!("https://discordapp.com/api/v6/channels/{}/messages", channel))
            .header(hyper::header::AUTHORIZATION, auth_value_ref)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .header(hyper::header::CONTENT_LENGTH, body.len())
            .body(body.into())
            .map_err(|e| Error::Other(format!("Failed to create request: {:?}", e))));

        Box::new(self.http_client.request(req)
                 .map_err(|e| e.into())
                 .and_then(|resp| -> Box<Future<Item=(), Error=Error> + Send> {
                     match resp.status() {
                         hyper::StatusCode::OK => Box::new(futures::future::ok(())),
                         _ => {
                             Box::new(resp.into_body().concat2()
                                      .map_err(|e| e.into())
                                      .and_then(|body| Err(Error::Other(format!("Message sending failed {}", String::from_utf8_lossy(&body.to_vec()))))))
                         }
                     }
                 }))
    }
}

fn handle_packet(msg: websocket::message::OwnedMessage) -> Option<Event> {
    if let websocket::message::OwnedMessage::Text(text) = msg {
        #[derive(Deserialize)]
        struct RecvPayload<'a> {
            pub op: u8,
            pub d: serde_json::Value,
            pub s: Option<u64>,
            pub t: Option<&'a str>
        }
        match serde_json::from_str::<RecvPayload>(&text) {
            Err(err) => {
                eprintln!("Failed to parse packet: {:?}", err);
                None
            },
            Ok(packet) => {
                match packet.op {
                    0 => {
                        match packet.t {
                            Some(ref t) => {
                                handle_event(&t, packet.d)
                            }
                            None => {
                                eprintln!("Missing event type");
                                None
                            }
                        }
                    },
                    11 => {
                        // heartbeat ACK
                        // potentially useful, but ignored for now
                        None
                    },
                    op => {
                        eprintln!("Unrecognized packet op: {}", op);
                        None
                    }
                }
            }
        }
    }
    else {
        eprintln!("Unexpected message type: {:?}", msg);
        None
    }
}

fn handle_event(t: &str, d: serde_json::Value) -> Option<Event> {
    match t {
        "READY" => {
            let user = match serde_json::from_value(d["user"].clone()) {
                Err(err) => {
                    eprintln!("Failed to parse ready user: {:?}", err);
                    return None;
                },
                Ok(x) => x
            };
            Some(Event::Ready(events::ReadyData {
                user
            }))
        },
        "MESSAGE_CREATE" => {
            match serde_json::from_value(d) {
                Err(err) => {
                    eprintln!("Failed to parse message: {:?}", err);
                    None
                },
                Ok(data) => Some(Event::MessageCreate(data))
            }
        },
        _ => {
            eprintln!("Unrecognized event type: {}", t);
            None
        }
    }
}
