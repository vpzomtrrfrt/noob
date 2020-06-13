use tokio_tungstenite::tungstenite;

#[derive(Debug)]
/// Error type for noob functions
pub enum Error {
    /// Failed to authenticate with the API
    AuthenticationFailed,
    /// Some other error
    Other(String),
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Error::Other(format!("HTTP Failure: {:?}", e))
    }
}

impl From<tungstenite::Error> for Error {
    fn from(e: tungstenite::Error) -> Self {
        Error::Other(format!("WebSocket Failure: {:?}", e))
    }
}
