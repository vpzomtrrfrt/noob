use tokio_core;
use hyper;
use futures;
use hyper_tls;
use serde_json;
use websocket;

use events::Event;
use error::Error;
use futures::future;
use futures::future::Future;

use std::str::FromStr;
use futures::Stream;

pub struct Client {
}

impl Client {
    pub fn login_bot(handle: &tokio_core::reactor::Handle, token: &str) -> Box<Future<Item=Client,Error=Error>> {
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
            .and_then(move |chunk| -> Box<futures::future::Future<Item=Client,Error=Error>> {
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
                        Ok(Client {})
                    }))
            })
            )
    }
}
