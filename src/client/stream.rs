use futures;
use serde_json;
use std;
use tokio;
use url;
use tokio_tungstenite;
use tokio_tungstenite::tungstenite;

use events;
use {Error, Event};

use futures::{Future, Sink, Stream};
use std::sync::{Arc, Mutex};
use tokio::executor::Executor;

/// Stream of gateway events
pub struct GatewayConnection {
    token: String,
    url: url::Url,
    state: ConnectionState,
}

enum ConnectionState {
    Pending(Box<Future<Item = ConnectionState, Error = Error> + Send>),
    Connected(Box<Stream<Item = Event, Error = Error> + Send>),
}

struct ReconnectInfo {
    session_id: String,
    last_event: u64,
}

impl GatewayConnection {
    #[doc(hidden)]
    pub fn connect_new(url: url::Url, token: String) -> Self {
        Self {
            state: ConnectionState::Pending(GatewayConnection::connect(&token, &url, None)),
            url,
            token,
        }
    }

    fn connect(
        token: &str,
        url: &url::Url,
        resume_info: Option<ReconnectInfo>,
    ) -> Box<Future<Item = ConnectionState, Error = Error> + Send> {
        let token = token.to_owned();
        Box::new(
            tokio_tungstenite::connect_async(url.clone())
                .map_err(|e| e.into())
                .and_then(|(socket, _)| socket.into_future().map_err(|(e, _)| e.into()))
                .and_then(
                    move |(msg1, socket)| -> Box<Future<Item = _, Error = _> + Send> {
                        #[derive(Deserialize)]
                        struct Hello {
                            pub heartbeat_interval: u64,
                        }

                        #[derive(Serialize)]
                        struct Identify<'a> {
                            token: &'a str,
                            properties: IdentifyProperties<'a>,
                            compress: bool,
                        }

                        #[derive(Serialize)]
                        struct Resume<'a> {
                            token: &'a str,
                            session_id: &'a str,
                            seq: u64,
                        }

                        #[derive(Serialize)]
                        struct IdentifyProperties<'a> {
                            #[serde(rename = "$os")]
                            os: &'a str,
                            #[serde(rename = "$browser")]
                            browser: &'a str,
                            #[serde(rename = "$device")]
                            device: &'a str,
                        }

                        if let Some(tungstenite::Message::Text(text)) = msg1 {
                            let payload: ::DiscordBasePayload<Hello> =
                                try_future_box!(serde_json::from_str(&text).map_err(|e| {
                                    Error::Other(format!("Failed to parse hello message: {:?}", e))
                                }));
                            let first_packet = match resume_info {
                                Some(info) => serde_json::to_string(&::DiscordBasePayload {
                                    op: 6,
                                    d: Resume {
                                        token: &token,
                                        session_id: &info.session_id,
                                        seq: info.last_event,
                                    },
                                }),
                                None => {
                                    serde_json::to_string(&::DiscordBasePayload {
                                        op: 2,
                                        d: Identify {
                                            token: &token,
                                            properties: IdentifyProperties {
                                                os: "linux", // TODO make this work
                                                browser: "noob",
                                                device: "noob",
                                            },
                                            compress: false,
                                        },
                                    })
                                }
                            };
                            let first_packet =
                                try_future_box!(first_packet.map_err(|e| Error::Other(format!(
                                    "Failed to serialize identify message: {:?}",
                                    e
                                ))));
                            Box::new(
                                socket
                                    .send(tungstenite::Message::Text(first_packet))
                                    .map_err(|e| e.into())
                                    .map(|socket| (socket, payload.d)),
                            )
                        } else {
                            Box::new(futures::future::err(Error::Other(format!(
                                "Unexpected first message: {:?}",
                                msg1
                            ))))
                        }
                    },
                )
                .and_then(|(socket, hello)| {
                    let session_info: Arc<Mutex<Option<ReconnectInfo>>> =
                        Arc::new(Mutex::new(None));
                    let (sink, stream) = socket.split();
                    let heartbeat_stream = tokio::timer::Interval::new(
                        std::time::Instant::now(),
                        std::time::Duration::from_millis(hello.heartbeat_interval),
                    );
                    let session_info_hb = session_info.clone();
                    tokio::executor::DefaultExecutor::current()
                        .spawn(Box::new(
                            sink.send_all(
                                heartbeat_stream
                                    .map_err(|e| -> tungstenite::Error {
                                        panic!("Timer error: {:?}", e);
                                    })
                                    .map(move |_| {
                                        tungstenite::Message::Text(
                                            json!({
                                                       "op": 1,
                                                       "d": match *session_info_hb.lock().unwrap() {
                                                           Some(ref info) => {
                                                               Some(info.last_event)
                                                           },
                                                           None => None
                                                       }
                                                   }).to_string(),
                                        )
                                    }),
                            ).map(|_| ())
                                .map_err(|e| {
                                    eprintln!("Websocket error in heartbeat stream: {:?}", e);
                                }),
                        ))
                        .map_err(|e| {
                            Error::Other(format!("Failed to spawn heartbeat stream: {:?}", e))
                        })?;
                    Ok(ConnectionState::Connected(Box::new(
                        stream
                            .map_err(|e| e.into())
                            .filter_map(move |packet| handle_packet(&session_info, packet)),
                    )))
                }),
        )
    }
}

impl Stream for GatewayConnection {
    type Item = Event;
    type Error = Error;

    fn poll(&mut self) -> futures::Poll<Option<Self::Item>, Error> {
        enum ConnPollRes {
            NewState(ConnectionState),
            Result(futures::Poll<Option<Event>, Error>),
        }
        let res =
            match self.state {
                ConnectionState::Pending(ref mut fut) => {
                    let res = fut.poll();
                    match res {
                        Ok(futures::Async::Ready(new_state)) => ConnPollRes::NewState(new_state),
                        Ok(futures::Async::NotReady) => {
                            ConnPollRes::Result(Ok(futures::Async::NotReady))
                        }
                        Err(err) => ConnPollRes::Result(Err(err)),
                    }
                }
                ConnectionState::Connected(ref mut stream) => {
                    let res = stream.poll();
                    match res {
                        Ok(futures::Async::Ready(None)) => {
                            // stream ended, reconnect
                            println!("reconnecting");
                            ConnPollRes::NewState(ConnectionState::Pending(
                                GatewayConnection::connect(&self.token, &self.url, None),
                            ))
                        }
                        other => ConnPollRes::Result(other),
                    }
                }
            };
        match res {
            ConnPollRes::NewState(state) => {
                self.state = state;
                self.poll()
            }
            ConnPollRes::Result(res) => res,
        }
    }
}

fn handle_packet(
    session_info: &Arc<Mutex<Option<ReconnectInfo>>>,
    msg: tungstenite::Message,
) -> Option<Event> {
    if let tungstenite::Message::Text(text) = msg {
        #[derive(Deserialize)]
        struct RecvPayload<'a> {
            pub op: u8,
            pub d: serde_json::Value,
            pub s: Option<u64>,
            pub t: Option<&'a str>,
        }
        match serde_json::from_str::<RecvPayload>(&text) {
            Err(err) => {
                eprintln!("Failed to parse packet: {:?}", err);
                None
            }
            Ok(packet) => {
                match packet.op {
                    0 => {
                        if let Some(seq) = packet.s {
                            if let Some(ref mut info) = *session_info.lock().unwrap() {
                                info.last_event = seq;
                            }
                        }
                        match packet.t {
                            Some(ref t) => handle_event(&t, packet.d),
                            None => {
                                eprintln!("Missing event type");
                                None
                            }
                        }
                    }
                    11 => {
                        // heartbeat ACK
                        // potentially useful, but ignored for now
                        None
                    }
                    op => {
                        eprintln!("Unrecognized packet op: {}", op);
                        None
                    }
                }
            }
        }
    } else {
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
                }
                Ok(x) => x,
            };
            Some(Event::Ready(events::ReadyData { user }))
        }
        "MESSAGE_CREATE" => match serde_json::from_value(d) {
            Err(err) => {
                eprintln!("Failed to parse message: {:?}", err);
                None
            }
            Ok(data) => Some(Event::MessageCreate(data)),
        },
        _ => {
            eprintln!("Unrecognized event type: {}", t);
            None
        }
    }
}
