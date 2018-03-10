extern crate native_tls;

use hyper;
use websocket;
use tokio_timer;

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
        Timer(e: tokio_timer::TimerError) {
            from()
        }
        AuthenticationFailed {}
        UnexpectedResponse(msg: String) {}
        InternalError(msg: String) {}
    }
}
