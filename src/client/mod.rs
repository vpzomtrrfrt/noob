use tokio_core;
use hyper;
use hyper_tls;
use native_tls;
use futures;
use serde_json;
use websocket;
use std;

use futures::prelude::*;
use std::str::FromStr;

mod events;

pub enum Error {
    HTTPError(hyper::Error),
    TLSError(native_tls::Error),
    WebsocketError(websocket::WebSocketError),
    AuthenticationFailed,
    UnexpectedResponse(String),
    NotReady,
    UhWhat(String)
}

impl std::fmt::Debug for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        fmt.write_str(std::error::Error::description(self))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        fmt.write_str(std::error::Error::description(self))
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::HTTPError(ref err) => std::error::Error::description(err),
            Error::TLSError(ref err) => std::error::Error::description(err),
            Error::WebsocketError(ref err) => std::error::Error::description(err),
            Error::AuthenticationFailed => "Authentication Failed",
            Error::UnexpectedResponse(ref msg) => &msg,
            Error::UhWhat(ref msg) => &msg,
            Error::NotReady => "The client is in the wrong state to do that."
        }
    }
}

type WebSocket = websocket::client::async::Client<Box<websocket::stream::async::Stream + Send>>;

enum ConnectionState {
    Disconnected,
    Connecting,
    Connected(WebSocket),
    Ready(WebSocket),
    Failed(Error),
}

#[derive(Debug, Clone)]
struct BotAuthorizationScheme {
    token: String,
}

impl FromStr for BotAuthorizationScheme {
    type Err = Error;
    fn from_str(token: &str) -> Result<Self, Self::Err> {
        Ok(BotAuthorizationScheme {
            token: token.to_owned(),
        })
    }
}

impl hyper::header::Scheme for BotAuthorizationScheme {
    fn scheme() -> Option<&'static str> {
        Some("Bot")
    }
    fn fmt_scheme(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.token)
    }
}

pub struct Client {
    handle: tokio_core::reactor::Handle,
    gateway_url: String,
    connection: ConnectionState,
    handler: PacketHandler,
    token: String
}

impl Client {
    pub fn login_bot(
        handle: &tokio_core::reactor::Handle,
        token: &str,
        event_callback: Box<Fn(events::Event) -> ()>
    ) -> Box<Future<Item = Client, Error = Error>> {
        let token = token.to_owned();
        let handle = handle.clone();
        let http = hyper::Client::configure()
            .connector(fut_try!(
                hyper_tls::HttpsConnector::new(1, &handle).map_err(|e| Error::TLSError(e))
            ))
            .build(&handle);
        let mut request = hyper::Request::new(
            hyper::Method::Get,
            fut_try!(
                hyper::Uri::from_str("https://discordapp.com/api/v6/gateway/bot")
                    .map_err(|e| Error::HTTPError(e.into()))
            ),
        );
        request.headers_mut().set(hyper::header::Authorization(
            fut_try!(BotAuthorizationScheme::from_str(&token)),
        ));
        Box::new(
            http.request(request)
                .map_err(|e| Error::HTTPError(e))
                .and_then(|response| {
                    println!("{:?}", response);
                    let status = response.status();
                    if status == hyper::StatusCode::Unauthorized {
                        return Err(Error::AuthenticationFailed);
                    }
                    if status == hyper::StatusCode::Ok {
                        return Ok(response.body());
                    }
                    Err(Error::UnexpectedResponse(format!(
                        "Gateway request responded with unexpected status {}",
                        status
                    )))
                })
                .and_then(|body| body.concat2().map_err(|e| Error::HTTPError(e)))
                .and_then(
                    |chunk| -> Box<futures::future::Future<Item = Client, Error = Error>> {
                        let value: serde_json::Value = match serde_json::from_slice(&chunk) {
                            Ok(value) => value,
                            Err(err) => {
                                return Box::new(futures::future::err(Error::UnexpectedResponse(
                                    format!("Unable to parse gateway API response: {:?}", err),
                                )))
                            }
                        };
                        println!("{}", value);
                        let mut client = Client::new(handle, match value["url"].as_str() {
                            None => return Box::new(futures::future::err(Error::UnexpectedResponse(
                                        "Gateway URI was not a string".to_owned()
                                        ))),
                            Some(x) => x.to_owned()
                        }, token, event_callback);
                        Box::new(client.connect())
                    },
                ),
        )
    }
    fn new(handle: tokio_core::reactor::Handle, gateway_url: String, token: String, event_callback: Box<Fn(events::Event) -> ()>) -> Self {
        Client {
            handle,
            gateway_url,
            connection: ConnectionState::Disconnected,
            token,
            handler: PacketHandler::new(event_callback)
        }
    }
    fn connect(mut self) -> Box<futures::future::Future<Item = Client, Error = Error>> {
        if match self.connection {
            ConnectionState::Disconnected => true,
            ConnectionState::Connecting => false,
            ConnectionState::Connected(_) => false,
            ConnectionState::Ready(_) => return Box::new(futures::future::ok(self)),
            ConnectionState::Failed(_) => true,
        } {
            self.connection = ConnectionState::Connecting;
            let uri = format!("{}/?v=6&encoding=json", self.gateway_url);
            let builder = match websocket::ClientBuilder::new(
                &uri,
            ) {
                Ok(builder) => builder,
                Err(err) => {
                    return Box::new(futures::future::err(Error::UnexpectedResponse(
                        format!("Unable to parse gateway URI {}: {}", uri, err),
                    )))
                }
            };
            let connection: websocket::client::async::ClientNew<Box<websocket::async::Stream + Send>> = builder.async_connect(None, &self.handle);
            return Box::new(connection
                .and_then(move |(socket, _)| {
                    self.connection = ConnectionState::Connected(socket);
                    println!("hi");
                    futures::future::ok(self)
                })
                .map_err(|e|Error::WebsocketError(e)));
        }
        Box::new(futures::future::ok(self))
    }
    pub fn run(mut self) -> Box<futures::future::Future<Item=(),Error=Error>> {
        let mut handler = self.handler;
        let token = self.token;
        match self.connection {
            ConnectionState::Connected(socket) => {
                let (sink, stream) = socket.split();
                let mapped = sink.sink_map_err(|e|Error::WebsocketError(e));
                Box::new(mapped.send_all(
                        stream.map_err(|e|Error::WebsocketError(e))
                        .and_then(move |packet| {
                            handler.handle_message(packet, &token)
                        })
                        .filter_map(|x|x)).map(|_|()))
            },
            _ => Box::new(futures::future::err(Error::NotReady)),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct Packet {
    op: u8,
    d: serde_json::Value,
    s: Option<u64>,
    t: Option<String>
}

struct PacketHandler {
    heartbeat_interval: Option<u64>,
    event_callback: Box<Fn(events::Event) -> ()>
}

impl PacketHandler {
    fn new(event_callback: Box<Fn(events::Event) -> ()>) -> Self {
        return PacketHandler {
            heartbeat_interval: None,
            event_callback
        };
    }

    fn handle_message(&mut self, message: websocket::message::OwnedMessage, token: &str) -> Result<Option<websocket::OwnedMessage>, Error> {
        use websocket::message::OwnedMessage;
        println!("{:?}", message);
        match message {
            OwnedMessage::Text(ref text) => {
                let packet: Packet = serde_json::from_str(text)
                    .map_err(|e|Error::UnexpectedResponse(
                            format!("Unable to parse packet JSON: {}", e)))?;
                self.handle_packet(packet, token)
            },
            _ => Err(Error::UnexpectedResponse(format!("Unexpected message type: {:?}", message)))
        }
    }

    fn handle_packet(&mut self, packet: Packet, token: &str) -> Result<Option<websocket::OwnedMessage>, Error> {
        match packet.op {
            0 => {
                let t = packet.t.ok_or(Error::UnexpectedResponse("Missing \"t\" in event dispatch".to_owned()))?;
                let event = match &t as &str {
                    "READY" => Ok(events::Event::Ready),
                    _ => Err(Error::UnexpectedResponse(format!("Unexpected event type: {}", t)))
                }?;
                (self.event_callback)(event);
                Ok(None)
            },
            10 => {
                self.heartbeat_interval = Some(packet.d["heartbeat_interval"].as_u64().ok_or(Error::UnexpectedResponse("heartbeat interval isn't a number?".to_owned()))?);
                Ok(Some(websocket::OwnedMessage::Text(serde_json::to_string(&Packet {
                    op: 2,
                    d: json!({
                        "token": token,
                        "properties": {
                            "$os": "linux",
                            "$browser": "tokio_discord",
                            "$device": "tokio_discord"
                        },
                        "compress": false
                    }),
                    s: None,
                    t: None
                }).map_err(|e|Error::UhWhat(format!("unable to serialize JSON: {}", e)))?)))
            },
            _ => Err(Error::UnexpectedResponse(format!("Unexpected opcode: {}", packet.op)))
        }
    }
}
