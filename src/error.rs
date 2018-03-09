extern crate native_tls;

use hyper;
use websocket;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        HTTP(e: hyper::Error) {
            from()
        }
        TLS(e: native_tls::Error) {
            from()
        }
        Websocket(e: websocket::WebSocketError) {
            from()
        }
        AuthenticationFailed {}
        UnexpectedResponse(msg: String) {}
        InternalError(msg: String) {}
    }
}
