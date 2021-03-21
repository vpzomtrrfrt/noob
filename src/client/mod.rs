pub use self::stream::GatewayConnection;
use crate::types::Snowflake;
use crate::Error;
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
    auth_value: String,
}

impl Client {
    /// Connect to the Discord gateway with a bot token
    pub async fn connect(token: &str) -> Result<(Client, stream::GatewayConnection), Error> {
        let auth_value = format!("Bot {}", token);

        let http = hyper::Client::builder().build(hyper_tls::HttpsConnector::new());

        let gateway_req = hyper::Request::get("https://discordapp.com/api/v6/gateway/bot")
            .header(hyper::header::AUTHORIZATION, &auth_value)
            .body(Default::default())
            .map_err(|e| Error::Other(format!("{:?}", e)))?;

        let resp = http.request(gateway_req).await?;
        let body = match resp.status() {
            hyper::StatusCode::UNAUTHORIZED => {
                return Err(Error::AuthenticationFailed);
            }
            hyper::StatusCode::OK => hyper::body::to_bytes(resp.into_body()).await?,
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
        let result: GetGatewayResult = serde_json::from_slice(&body)
            .map_err(|e| Error::Other(format!("Unable to parse Gateway API response: {:?}", e)))?;

        println!("{}", result.url);
        url::Url::parse(&result.url)
            .map_err(|e| Error::Other(format!("Unable to parse Gateway URL: {:?}", e)))
            .map(|url| {
                (
                    Client {
                        http_client: http,
                        auth_value: auth_value.clone(),
                    },
                    stream::GatewayConnection::connect_new(url, auth_value),
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
        let req = hyper::Request::post(format!(
            "https://discordapp.com/api/v6/channels/{}/messages",
            channel
        ))
        .header(hyper::header::AUTHORIZATION, &self.auth_value)
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .header(hyper::header::CONTENT_LENGTH, body.len())
        .body(body.into())
        .map_err(|e| Error::Other(format!("Failed to create request: {:?}", e)))?;

        res_to_error(self.http_client.request(req).await?).await?;

        Ok(())
    }

    /// Returns the messages for a channel
    pub async fn get_channel_messages(
        &self,
        channel: &str,
        anchor: ListAnchor<'_>,
        limit: impl Into<Option<u8>>,
    ) -> Result<Vec<crate::types::Message>, Error> {
        let mut url = format!(
            "https://discordapp.com/api/v6/channels/{}/messages?",
            percent_encoding::utf8_percent_encode(channel, percent_encoding::NON_ALPHANUMERIC),
        );

        {
            let len = url.len();
            let mut query_ser = form_urlencoded::Serializer::for_suffix(&mut url, len);
            anchor.write_to_query(&mut query_ser);

            if let Some(limit) = limit.into() {
                query_ser.append_pair("limit", &limit.to_string());
            }
        }

        let req = hyper::Request::get(url)
            .header(hyper::header::AUTHORIZATION, &self.auth_value)
            .body(Default::default())
            .map_err(|e| Error::Other(format!("Failed to create request: {:?}", e)))?;

        let res = res_to_error(self.http_client.request(req).await?).await?;
        let body = hyper::body::to_bytes(res.into_body()).await?;

        serde_json::from_slice(&body)
            .map_err(|e| Error::Other(format!("Unable to parse Gateway API response: {:?}", e)))
    }
}

/// How the message list should be paginated
pub enum ListAnchor<'a> {
    /// Retrieve messages around the specified ID
    Around(&'a Snowflake),
    /// Retrieve messages before the specified ID
    Before(&'a Snowflake),
    /// Retrieve messages after the specified ID
    After(&'a Snowflake),
}

impl<'a> ListAnchor<'a> {
    fn write_to_query<T: form_urlencoded::Target>(&self, ser: &mut form_urlencoded::Serializer<T>) {
        match self {
            ListAnchor::Around(id) => ser.append_pair("around", id),
            ListAnchor::Before(id) => ser.append_pair("before", id),
            ListAnchor::After(id) => ser.append_pair("after", id),
        };
    }
}
