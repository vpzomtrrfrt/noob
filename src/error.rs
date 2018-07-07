use hyper;
use hyper_tls;
use websocket;

quick_error! {
    #[derive(Debug)]
    /// Error type for noob functions
    pub enum Error {
        /// Failed to authenticate with the API
        AuthenticationFailed {}
        /// Some other error
        Other(e: String) {}
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Error::Other(format!("HTTP Failure: {:?}", e))
    }
}

impl From<hyper_tls::Error> for Error {
    fn from(e: hyper_tls::Error) -> Self {
        Error::Other(format!("HTTPS Failure: {:?}", e))
    }
}

impl From<websocket::WebSocketError> for Error {
    fn from(e: websocket::WebSocketError) -> Self {
        Error::Other(format!("WebSocket Failure: {:?}", e))
    }
}
