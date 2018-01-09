use tokio_core;
use hyper;
use hyper_tls;
use native_tls;
use futures;
use std;

use futures::prelude::*;
use std::str::FromStr;

pub enum Error {
    HTTPError(hyper::Error),
    TLSError(native_tls::Error),
    AuthenticationFailed,
    UnexpectedResponse(String)
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
            Error::AuthenticationFailed => "Authentication Failed",
            Error::UnexpectedResponse(ref msg) => &msg
        }
    }
}

pub struct Client {
    handle: tokio_core::reactor::Handle,
    http: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>
}

impl Client {
    pub fn login_bot(handle: &tokio_core::reactor::Handle, token: &str) -> Box<Future<Item=Client, Error=Error>> {
        let handle = handle.clone();
        let http = hyper::Client::configure()
            .connector(fut_try!(hyper_tls::HttpsConnector::new(1, &handle).map_err(|e|Error::TLSError(e))))
            .build(&handle);
        let mut request = hyper::Request::new(
            hyper::Method::Get,
            fut_try!(hyper::Uri::from_str("https://discordapp.com/api/gateway/bot").map_err(|e|Error::HTTPError(e.into())))
            );
        Box::new(http.request(request)
                 .map_err(|e|Error::HTTPError(e))
           .and_then(|response| {
                println!("{:?}", response);
                if response.status() == hyper::StatusCode::Unauthorized {
                    return futures::future::err(Error::AuthenticationFailed);
                }
                futures::future::ok(Client {
                    handle,
                    http
                })
            }))
    }
}
