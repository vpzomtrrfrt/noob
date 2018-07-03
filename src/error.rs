use hyper;
use hyper_tls;
use websocket;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        AuthenticationFailed {}
        Hyper(e: hyper::Error) { from() }
        HyperTLS(e: hyper_tls::Error) { from() }
        Websocket(e: websocket::WebSocketError) { from() }
        Other(e: String) {}
    }
}
