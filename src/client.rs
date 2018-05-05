use tokio_core;
use hyper;
use futures;
use hyper_tls;
use serde_json;
use websocket;
use std;
use tokio_timer;

use events;
pub use embed::*;

use events::Event;
use error::Error;

use std::sync::Arc;
use futures::future::Future;
use std::str::FromStr;
use futures::Stream;

pub struct Client {
    token: String,
    http: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>
}

pub fn run_bot<F: 'static + Fn(Event, &Arc<Client>) -> ()>(
    handle: &tokio_core::reactor::Handle,
    token: &str,
    listener: F
    ) -> Box<Future<Item=(),Error=Error>> {
    let token = token.to_owned();
    let handle = handle.clone();
    let http = hyper::Client::configure()
        .connector(box_fut_try!(
                hyper_tls::HttpsConnector::new(1, &handle).map_err(|e|e.into())
                ))
        .build(&handle);
    let client = Arc::new(Client {
        token: token.clone(),
        http
    });
    let mut request = hyper::Request::new(
        hyper::Method::Get,
        box_fut_try!(
            hyper::Uri::from_str("https://discordapp.com/api/v6/gateway/bot")
            .map_err(|e| Error::HTTP(e.into()))
            ));
    request.headers_mut().set(hyper::header::Authorization(token.to_owned()));
    Box::new(client.http.request(request)
             .map_err(|e| e.into())
             .and_then(|response| {
                 let status = response.status();
                 if status == hyper::StatusCode::Unauthorized {
                     return Err(Error::AuthenticationFailed);
                 }
                 if status == hyper::StatusCode::Ok {
                     return Ok(response.body());
                 }
                 Err(Error::UnexpectedResponse(format!("Gateway request responded with unexpected status {}", status)))
             })
             .and_then(|body| body.concat2().map_err(|e| e.into()))
             .and_then(move |chunk| -> Box<futures::future::Future<Item=(),Error=Error>> {
                 #[derive(Deserialize)]
                 struct GetGatewayResult {
                     url: String
                 }
                 let result: GetGatewayResult = box_fut_try!(serde_json::from_slice(&chunk).map_err(|e| Error::UnexpectedResponse(format!("Unable to parse Gateway API response: {:?}", e))));
                 Box::new(box_fut_try!(websocket::ClientBuilder::new(&result.url)
                                       .map_err(|e|Error::UnexpectedResponse(
                                               format!("Failed to parse Gateway URI: {:?}", e))))
                          .async_connect(None, &handle)
                          .map_err(|e|e.into())
                          .and_then(|(socket, _)| {
                              use futures::Sink;
                              let (sink, recv) = futures::sync::mpsc::unbounded::<websocket::OwnedMessage>();
                              let (ws_sink, ws_stream) = socket.split();
                              let seq_store = std::sync::Arc::new(
                                  std::sync::RwLock::new(None));
                              let input = ws_stream.map_err(|e|e.into())
                                  .for_each(move |message| {
                                      match message {
                                          websocket::message::OwnedMessage::Text(t) => {
                                              match handle_packet(&t, &token, &sink, &handle, &seq_store, &client, &listener) {
                                                  Ok(_) => {},
                                                  Err(e) => eprintln!("Error handling packet: {:?}", e)
                                              }
                                          },
                                          e => {
                                              eprintln!("Unexpected websocket message: {:?}", e);
                                          }
                                      }
                                      Ok(())
                                  });
                              let output = ws_sink.sink_map_err(|e|Error::InternalError(format!("mpsc failure: {:?}", e)))
                                  .send_all(recv.map_err(|e|Error::InternalError(format!("mpsc failure 2: {:?}", e))));
                              input.join(output).map(|_|())
                          }))
             })
    )
}

#[derive(Debug, Deserialize, Serialize)]
struct Packet {
    op: u8,
    d: serde_json::Value,
    s: Option<u64>,
    t: Option<String>
}

fn handle_packet<F: Fn(Event, &Arc<Client>) -> ()>(
    text: &str,
    token: &str,
    sink: &futures::sync::mpsc::UnboundedSender<websocket::OwnedMessage>,
    handle: &tokio_core::reactor::Handle,
    seq_store: &std::sync::Arc<std::sync::RwLock<Option<u64>>>,
    client: &Arc<Client>,
    listener: &F) -> Result<(), Error> {
    let packet: Packet = serde_json::from_str(text).map_err(
        |e|Error::UnexpectedResponse(format!("Unable to parse JSON: {:?}", e))
        )?;
    if let Some(s) = packet.s {
        let mut ptr = seq_store.write().unwrap();
        *ptr = Some(s);
    }
    // println!("packet: {:?}", packet);
    match packet.op {
        0 => {
            if let Some(t) = packet.t {
                handle_event(&t, &packet.d, client, listener);
            }
            Ok(())
        },
        10 => {
            send_packet(sink, &Packet {
                op: 2,
                s: None,
                t: None,
                d: json!({
                    "token": token,
                    "properties": {
                        "$os": "linux", // TODO make this work
                        "$browser": "noob",
                        "$device": "noob"
                    },
                    "compress": false
                })
            })?;
            let seq_store = seq_store.clone();
            let sink = sink.clone();
            handle.spawn(tokio_timer::Timer::default().interval(std::time::Duration::from_millis(packet.d["heartbeat_interval"].as_u64().ok_or_else(||Error::UnexpectedResponse("Unable to parse heartbeat interval".to_owned()))?))
                .for_each(move |_| {
                    let s = {
                        let ptr = seq_store.read().unwrap();
                        *ptr
                    };
                    send_packet(&sink, &Packet {
                        op: 1,
                        s: None,
                        t: None,
                        d: json!(s)
                    }).unwrap();
                    Ok(())
                }).map_err(|e|panic!(e)));
            Ok(())
        },
        11 => {
            // heartbeat ack
            // TODO actually do something with this
            Ok(())
        },
        op => {
            eprintln!("Unexpected opcode: {}", op);
            Ok(())
        }
    }
}

fn handle_event<F: Fn(Event, &Arc<Client>) -> ()>(t: &str, d: &serde_json::Value, client: &Arc<Client>, listener: &F) {
    match t {
        "READY" => {
            listener(Event::Ready(events::ReadyData {
                guilds: &d["guilds"].as_array().unwrap().iter().map(|v| v["id"].as_str().unwrap().to_owned()).collect::<Vec<_>>(),
                session_id: d["session_id"].as_str().unwrap(),
                user: serde_json::from_str(&serde_json::to_string(&d["user"]).unwrap()).unwrap()
            }), client);
        },
        "MESSAGE_CREATE" => {
            listener(Event::MessageCreate(serde_json::from_str(&serde_json::to_string(&d).unwrap()).unwrap()), client);
        },
        _ => {
            eprintln!("Unrecognized event type: {}", t);
        }
    }
}

fn send_packet(
    sink: &futures::sync::mpsc::UnboundedSender<websocket::OwnedMessage>,
    packet: &Packet) -> Result<(), Error>
{
    sink.unbounded_send(websocket::OwnedMessage::Text(serde_json::to_string(packet).map_err(|e|Error::UnexpectedResponse(format!("Unable to serialize packet: {:?}", e)))?))
        .map_err(|e|Error::InternalError(format!("mpsc failure3: {:?}", e)))
}

impl Client {
    pub fn create_message(&self, content: String) -> MessageBuilder {
        MessageBuilder::new(self, content)
    }
    fn get_auth_header(&self) -> hyper::header::Authorization<String> {
        hyper::header::Authorization(format!("Bot {}", self.token))
    }
}

#[must_use]
pub struct MessageBuilder<'a> {
    client: &'a Client,
    content: String,
    embed: Option<Embed>
}

#[derive(Serialize)]
struct MessageCreationObject<'a> {
    content: &'a str,
    channel_id: &'a str,
    embed: &'a Option<Embed>
}

impl<'a> MessageBuilder<'a> {
    fn new(client: &'a Client, content: String) -> Self {
        MessageBuilder {
            client,
            content,
            embed: None
        }
    }
    pub fn set_embed(&mut self, embed: Embed) -> &mut Self {
        self.embed = Some(embed);
        self
    }
    pub fn send(&self, channel_id: &str) -> Box<Future<Item=(),Error=Error>> {
        let mut request = hyper::Request::new(
            hyper::Method::Post,
            box_fut_try!(
                hyper::Uri::from_str(&format!("https://discordapp.com/api/v6/channels/{}/messages", channel_id))
                .map_err(|e| Error::HTTP(e.into()))));
        request.headers_mut().set(self.client.get_auth_header());
        request.headers_mut().set(hyper::header::ContentType::json());
        let body = box_fut_try!(serde_json::to_string(&MessageCreationObject {
            content: &self.content,
            channel_id,
            embed: &self.embed
        }).map_err(|e|e.into()));
        request.headers_mut().set(hyper::header::ContentLength(body.len() as u64));
        request.set_body(body);
        Box::new(self.client.http.request(request)
                 .map_err(|e| e.into())
                 .and_then(|response| -> Box<Future<Item=(),Error=Error>> {
                     let status = response.status();
                     if status == hyper::StatusCode::Ok {
                         Box::new(futures::future::ok(()))
                     }
                     else {
                         Box::new(response.body().concat2()
                                  .map_err(|e|e.into())
                             .and_then(|body| 
                                       Err(Error::UnexpectedResponse(
                                               format!("Message sending failed: {}",
                                                       String::from_utf8(body.to_vec())
                                                       .map_err(|e| Error::UnexpectedResponse(format!(
                                                               "Failed to decode error message: {:?}",
                                                               e
                                                               )))?)))))
                     }
                 }))
    }
}
