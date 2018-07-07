use futures;
use hyper;
use hyper_tls;
use serde_json;
use websocket;

use Error;

use futures::{Future, Stream};

mod stream;

/// Object used to interact with the Discord API
pub struct Client {
    http_client: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
    token: String,
}

impl Client {
    /// Connect to the Discord gateway with a bot token
    pub fn connect(
        token: &str,
    ) -> Box<Future<Item = (Client, stream::GatewayConnection), Error = Error> + Send> {
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
                .and_then(
                    |resp| -> Box<Future<Item = hyper::Chunk, Error = Error> + Send> {
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
                    },
                )
                .and_then(|body| {
                    #[derive(Deserialize)]
                    struct GetGatewayResult<'a> {
                        url: &'a str,
                    }
                    let result: GetGatewayResult = serde_json::from_slice(&body).map_err(|e| {
                        Error::Other(format!("Unable to parse Gateway API response: {:?}", e))
                    })?;

                    println!("{}", result.url);
                    websocket::client::builder::Url::parse(&result.url)
                        .map_err(|e| Error::Other(format!("Unable to parse Gateway URL: {:?}", e)))
                        .map(|url| {
                            (
                                Client {
                                    http_client: http,
                                    token: token.clone(),
                                },
                                stream::GatewayConnection::connect_new(url, token),
                            )
                        })
                }),
        )
    }

    /// Send a message on a channel
    pub fn send_message(
        &self,
        message: &::MessageBuilder,
        channel: &str,
    ) -> Box<Future<Item = (), Error = Error> + Send> {
        let body = try_future_box!(message.to_request_body(channel));
        let auth_value = format!("Bot {}", self.token);
        let auth_value_ref: &str = &auth_value;
        let req = try_future_box!(
            hyper::Request::post(format!(
                "https://discordapp.com/api/v6/channels/{}/messages",
                channel
            )).header(hyper::header::AUTHORIZATION, auth_value_ref)
                .header(hyper::header::CONTENT_TYPE, "application/json")
                .header(hyper::header::CONTENT_LENGTH, body.len())
                .body(body.into())
                .map_err(|e| Error::Other(format!("Failed to create request: {:?}", e)))
        );

        Box::new(
            self.http_client
                .request(req)
                .map_err(|e| e.into())
                .and_then(|resp| -> Box<Future<Item = (), Error = Error> + Send> {
                    match resp.status() {
                        hyper::StatusCode::OK => Box::new(futures::future::ok(())),
                        _ => Box::new(resp.into_body().concat2().map_err(|e| e.into()).and_then(
                            |body| {
                                Err(Error::Other(format!(
                                    "Message sending failed {}",
                                    String::from_utf8_lossy(&body.to_vec())
                                )))
                            },
                        )),
                    }
                }),
        )
    }
}
