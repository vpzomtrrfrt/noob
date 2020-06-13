use crate::Error;
pub use self::stream::GatewayConnection;
use serde_derive::Deserialize;

mod stream;

async fn res_to_error(
    res: hyper::Response<hyper::Body>,
) -> Result<hyper::Response<hyper::Body>, crate::Error> {
    if res.status().is_success() {
        Ok(res)
    } else {
        let bytes = hyper::body::to_bytes(res.into_body()).await?;
        Err(Error::Other(format!(
            "Error in remote response: {}",
            String::from_utf8_lossy(&bytes)
        )))
    }
}

/// Object used to interact with the Discord API
pub struct Client {
    http_client: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
    token: String,
}

impl Client {
    /// Connect to the Discord gateway with a bot token
    pub async fn connect(
        token: String,
    ) -> Result<(Client, stream::GatewayConnection), Error> {
        let http = hyper::Client::builder().build(hyper_tls::HttpsConnector::new());

        let gateway_req = hyper::Request::get("https://discordapp.com/api/v6/gateway/bot")
            .header(hyper::header::AUTHORIZATION, &token)
            .body(Default::default())
            .map_err(|e| Error::Other(format!("{:?}", e)))?;

        let resp = http.request(gateway_req).await?;
        let body = match resp.status() {
            hyper::StatusCode::UNAUTHORIZED => {
                return Err(Error::AuthenticationFailed);
            }
            hyper::StatusCode::OK => {
                hyper::body::to_bytes(resp.into_body()).await?
            }
            status => {
                return Err(Error::Other(format!(
                    "Gateway request returned unexpected status {}",
                    status
                )));
            }
        };

        #[derive(Deserialize)]
        struct GetGatewayResult<'a> {
            url: &'a str,
        }
        let result: GetGatewayResult = serde_json::from_slice(&body).map_err(|e| {
            Error::Other(format!("Unable to parse Gateway API response: {:?}", e))
        })?;

        println!("{}", result.url);
        url::Url::parse(&result.url)
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
    }

    /// Send a message on a channel
    pub async fn send_message(
        &self,
        message: &crate::MessageBuilder<'_>,
        channel: &str,
    ) -> Result<(), Error> {
        let body = message.to_request_body(channel)?;
        let auth_value = format!("Bot {}", self.token);
        let auth_value_ref: &str = &auth_value;
        let req = hyper::Request::post(format!(
                "https://discordapp.com/api/v6/channels/{}/messages",
                channel
        ))
            .header(hyper::header::AUTHORIZATION, auth_value_ref)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .header(hyper::header::CONTENT_LENGTH, body.len())
            .body(body.into())
            .map_err(|e| Error::Other(format!("Failed to create request: {:?}", e)))?;

        res_to_error(self.http_client.request(req).await?).await?;

        Ok(())
    }
}
