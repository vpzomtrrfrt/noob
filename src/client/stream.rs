use crate::{Error, Event};

use futures::{SinkExt, Stream, StreamExt, TryStreamExt};
use serde_derive::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio_tungstenite::tungstenite;

struct ConnParams {
    token: String,
    url: url::Url,
}

/// Stream of gateway events
pub struct GatewayConnection {
    conn_params: Arc<ConnParams>,
    state: ConnectionState,
}

enum ConnectionState {
    Pending(Pin<Box<dyn Future<Output = Result<ConnectionState, Error>> + Send>>),
    Connected(Pin<Box<dyn Stream<Item = Result<Event, Error>> + Send>>),
}

struct ReconnectInfo {
    session_id: String,
    last_event: u64,
}

impl GatewayConnection {
    pub(crate) fn connect_new(url: url::Url, token: String) -> Self {
        let conn_params = Arc::new(ConnParams { token, url });
        Self {
            state: ConnectionState::Pending(Box::pin(GatewayConnection::connect(
                conn_params.clone(),
                None,
            ))),
            conn_params,
        }
    }

    async fn connect(
        conn_params: Arc<ConnParams>,
        resume_info: Option<ReconnectInfo>,
    ) -> Result<ConnectionState, Error> {
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

        let (socket, _) = tokio_tungstenite::connect_async(&conn_params.url).await?;
        let (msg1, mut socket) = socket.into_future().await;

        let msg1 = msg1.transpose()?;

        let (socket, hello) = if let Some(tungstenite::Message::Text(text)) = msg1 {
            let payload: crate::DiscordBasePayload<Hello> = serde_json::from_str(&text)
                .map_err(|e| Error::Other(format!("Failed to parse hello message: {:?}", e)))?;
            let first_packet = match resume_info {
                Some(info) => serde_json::to_string(&crate::DiscordBasePayload {
                    op: 6,
                    d: Resume {
                        token: &conn_params.token,
                        session_id: &info.session_id,
                        seq: info.last_event,
                    },
                }),
                None => {
                    serde_json::to_string(&crate::DiscordBasePayload {
                        op: 2,
                        d: Identify {
                            token: &conn_params.token,
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
            let first_packet = first_packet.map_err(|e| {
                Error::Other(format!("Failed to serialize identify message: {:?}", e))
            })?;

            socket
                .send(tungstenite::Message::Text(first_packet))
                .await?;

            Ok((socket, payload.d))
        } else {
            Err(Error::Other(format!(
                "Unexpected first message: {:?}",
                msg1
            )))
        }?;
        let session_info: Arc<Mutex<Option<ReconnectInfo>>> = Arc::new(Mutex::new(None));
        let (mut sink, stream) = socket.split();
        let mut heartbeat_interval =
            tokio::time::interval(std::time::Duration::from_millis(hello.heartbeat_interval));
        let session_info_hb = session_info.clone();
        tokio::spawn(async move {
            loop {
                heartbeat_interval.tick().await;
                if let Err(err) = sink
                    .send(tungstenite::Message::Text(
                        serde_json::json!({
                            "op": 1,
                            "d": match *session_info_hb.lock().unwrap() {
                                Some(ref info) => {
                                    Some(info.last_event)
                                },
                                None => None
                            }
                        })
                        .to_string(),
                    ))
                    .await
                {
                    eprintln!("Websocket error in heartbeat stream: {:?}", err);
                }
            }
        });
        Ok(ConnectionState::Connected(Box::pin(
            stream.map_err(|e| e.into()).try_filter_map(move |packet| {
                futures::future::ready(Ok(handle_packet(&session_info, packet)))
            }),
        )))
    }
}

impl Stream for GatewayConnection {
    type Item = Result<Event, Error>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        ctx: &mut futures::task::Context,
    ) -> futures::task::Poll<Option<Self::Item>> {
        enum ConnPollRes {
            NewState(ConnectionState),
            Result(futures::task::Poll<Option<Result<Event, Error>>>),
        }
        let res = match self.state {
            ConnectionState::Pending(ref mut fut) => {
                let res = fut.as_mut().poll(ctx);
                match res {
                    futures::task::Poll::Ready(Ok(new_state)) => ConnPollRes::NewState(new_state),
                    futures::task::Poll::Pending => {
                        ConnPollRes::Result(futures::task::Poll::Pending)
                    }
                    futures::task::Poll::Ready(Err(err)) => {
                        ConnPollRes::Result(futures::task::Poll::Ready(Some(Err(err))))
                    }
                }
            }
            ConnectionState::Connected(ref mut stream) => {
                let res = stream.as_mut().poll_next(ctx);
                match res {
                    futures::task::Poll::Ready(None) => {
                        // stream ended, reconnect
                        println!("reconnecting");
                        ConnPollRes::NewState(ConnectionState::Pending(Box::pin(
                            GatewayConnection::connect(self.conn_params.clone(), None),
                        )))
                    }
                    other => ConnPollRes::Result(other),
                }
            }
        };
        match res {
            ConnPollRes::NewState(state) => {
                self.state = state;
                self.poll_next(ctx)
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
            Some(Event::Ready(crate::events::ReadyData { user }))
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
