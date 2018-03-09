use tokio_core;
use hyper;
use futures;
use hyper_tls;
use serde_json;
use websocket;
use std;

use events::Event;
use error::Error;
use futures::future;
use futures::future::Future;

use std::str::FromStr;
use futures::Stream;

type WebSocket = websocket::client::async::Client<Box<websocket::stream::async::Stream + Send>>;

pub fn run_bot<F: Fn(Event) -> ()>(
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
    let mut request = hyper::Request::new(
        hyper::Method::Get,
        box_fut_try!(
            hyper::Uri::from_str("https://discordapp.com/api/v6/gateway/bot")
            .map_err(|e| Error::HTTP(e.into()))
            ));
    request.headers_mut().set(hyper::header::Authorization(token.to_owned()));
    Box::new(http.request(request)
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
                              let (sink, recv) = futures::sync::mpsc::channel::<websocket::OwnedMessage>(64);
                              let (ws_sink, ws_stream) = socket.split();
                              let input = ws_stream.map_err(|e|e.into())
                                  .for_each(move |message| {
                                      println!("message: {:?}", message);
                                      match message {
                                          websocket::message::OwnedMessage::Text(t) => {
                                              match handle_packet(&t, &token, &mut sink.clone(), &handle) {
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

fn handle_packet(
    text: &str,
    token: &str,
    sink: &mut futures::sync::mpsc::Sender<websocket::OwnedMessage>,
    handle: &tokio_core::reactor::Handle) -> Result<(), Error> {
    let packet: Packet = serde_json::from_str(text).map_err(
        |e|Error::UnexpectedResponse(format!("Unable to parse JSON: {:?}", e))
        )?;
    println!("packet: {:?}", packet);
    match packet.op {
        10 => {
            handle.spawn(send_packet(sink, &Packet {
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
            }).map_err(|e|panic!(e)));
            Ok(())
        },
        op => {
            eprintln!("Unexpected opcode: {}", op);
            Ok(())
        }
    }
}

fn send_packet<S: 'static + futures::Sink<SinkItem=websocket::OwnedMessage>>(
    sink: &mut S,
    packet: &Packet) -> Box<Future<Item=(),Error=Error>>
where <S as futures::Sink>::SinkError: std::fmt::Debug
{
    use futures::Sink;
    use futures::IntoFuture;
    Box::new(sink.start_send(websocket::OwnedMessage::Text(box_fut_try!(serde_json::to_string(packet).map_err(|e|Error::UnexpectedResponse(format!("Unable to serialize packet: {:?}", e))))))
             .map(|_|()).map_err(|e|Error::UnexpectedResponse(format!("Error sending packet: {:?}", e))).into_future())
}
