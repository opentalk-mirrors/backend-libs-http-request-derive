// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::time::SystemTime;

use bytes::Bytes;
use http_request_derive::HttpRequest;
use http_request_derive_client::Client;
use http_request_derive_logging::HttpLogger;
use snafu::ResultExt as _;
use url::Url;

use crate::{
    ReqwestClientError,
    reqwest_client_error::{
        BuildHttpResponseBodySnafu, ConvertToHttpRequestSnafu, ConvertToReqwestRequestSnafu,
        ReadResponseSnafu, RequestExecutionSnafu, RetrieveResponseBodySnafu,
    },
};

/// A client for executing requests as defined by [`http_request_derive::HttpRequest`] implementations.
#[derive(Debug, Clone)]
pub struct ReqwestClient {
    client: reqwest::Client,
    base_url: Url,
    logger: Option<HttpLogger>,
}

impl ReqwestClient {
    /// Create a new [`ReqwestClient`] from a base [`Url`].
    pub fn new(base_url: Url) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            logger: None,
        }
    }

    /// Get the logger for dumping information about the HTTP communication if it is set.
    pub fn logger(&self) -> Option<HttpLogger> {
        self.logger.clone()
    }

    /// Set a logger for dumping information about the HTTP communication.
    pub fn set_logger(&mut self, logger: HttpLogger) {
        self.logger = Some(logger);
    }

    /// Return the client with a new logger for dumping the HTTP communication.
    pub fn with_logger(mut self, logger: HttpLogger) -> Self {
        self.logger = Some(logger);
        self
    }

    /// Returns the base URL which is used for subsequent requests.
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Sets the base URL to the given URL.
    pub fn set_base_url(&mut self, base_url: Url) {
        self.base_url = base_url
    }

    /// Return the client with a new base URL.
    pub fn with_base_url(mut self, base_url: Url) -> Self {
        self.base_url = base_url;
        self
    }

    async fn execute_http_request(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> Result<http::Response<Bytes>, ReqwestClientError> {
        let request = reqwest::Request::try_from(request).context(ConvertToReqwestRequestSnafu)?;
        let response = self
            .client
            .execute(request)
            .await
            .context(RequestExecutionSnafu)?;
        let mut http_response = http::Response::builder()
            .status(response.status())
            .version(response.version());
        if let Some(headers) = http_response.headers_mut() {
            *headers = response.headers().clone();
        }
        let body = response.bytes().await.context(RetrieveResponseBodySnafu)?;
        let http_response = http_response
            .body(body)
            .context(BuildHttpResponseBodySnafu)?;
        Ok(http_response)
    }

    async fn execute_http_request_with_optional_logging(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> Result<http::Response<Bytes>, ReqwestClientError> {
        if let Some(logger) = self.logger.as_ref() {
            let start_time = SystemTime::now();
            let response = self.execute_http_request(request.clone()).await;
            logger
                .log_request(start_time, &request, response.as_ref().ok())
                .await;
            return response;
        }

        self.execute_http_request(request).await
    }
}

#[async_trait::async_trait(?Send)]
impl Client for ReqwestClient {
    type ClientError = ReqwestClientError;

    async fn execute<R: HttpRequest + Send>(
        &self,
        request: R,
    ) -> Result<R::Response, Self::ClientError> {
        let request = request
            .to_http_request(&self.base_url)
            .context(ConvertToHttpRequestSnafu)?;
        let http_response = self
            .execute_http_request_with_optional_logging(request)
            .await?;
        R::read_response(http_response).context(ReadResponseSnafu)
    }
}

#[cfg(test)]
mod tests {
    use http::StatusCode;
    use http_request_derive::HttpRequest;
    use http_request_derive_client::Client as _;
    use httptest::{
        Expectation, all_of,
        matchers::{json_decoded, request},
        responders::{json_encoded, status_code},
    };
    use pretty_assertions::{assert_eq, assert_matches};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use url::Url;

    use crate::ReqwestClient;

    #[tokio::test]
    async fn simple_successful_get_request() {
        #[derive(HttpRequest)]
        #[http_request(method = "GET",response = ResponseBody, path = "/query/a/response")]
        struct Request;

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct ResponseBody {
            name: String,
        }

        _ = pretty_env_logger::try_init();
        let server = httptest::Server::run();

        server.expect(
            Expectation::matching(request::method_path("GET", "/api/query/a/response"))
                .respond_with(json_encoded(json!(ResponseBody {
                    name: "hello".to_string()
                }))),
        );

        let url = server
            .url("/api")
            .to_string()
            .parse()
            .expect("must be a valid url");
        let client = ReqwestClient::new(url);

        let response = client
            .execute(Request)
            .await
            .expect("valid response expected");

        assert_eq!(
            response,
            ResponseBody {
                name: "hello".to_string()
            }
        );
    }

    #[tokio::test]
    async fn simple_get_request_failing_with_invalid_json_body() {
        #[derive(HttpRequest)]
        #[http_request(method = "GET",response = ResponseBody, path = "/query/a/response")]
        struct Request;

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct ResponseBody {
            name: String,
        }

        _ = pretty_env_logger::try_init();
        let server = httptest::Server::run();

        server.expect(
            Expectation::matching(request::method_path("GET", "/api/query/a/response"))
                .respond_with(http::Response::new(r#"{ "invalid": JSON }"#)),
        );

        let url = server
            .url("/api")
            .to_string()
            .parse()
            .expect("must be a valid url");
        let client = ReqwestClient::new(url);

        let response = client.execute(Request).await;
        assert_matches!(
            response,
            Err(crate::ReqwestClientError::ReadResponse {
                source: http_request_derive::Error::Json { source: _ }
            })
        );
    }

    #[tokio::test]
    async fn simple_get_request_failing_with_internal_server_error() {
        #[derive(HttpRequest)]
        #[http_request(method = "GET",response = ResponseBody, path = "/query/a/response")]
        struct Request;

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct ResponseBody {
            name: String,
        }

        _ = pretty_env_logger::try_init();
        let server = httptest::Server::run();

        server.expect(
            Expectation::matching(request::method_path("GET", "/api/query/a/response"))
                .respond_with(status_code(StatusCode::INTERNAL_SERVER_ERROR.as_u16())),
        );

        let url = server
            .url("/api")
            .to_string()
            .parse()
            .expect("must be a valid url");
        let client = ReqwestClient::new(url);

        let response = client.execute(Request).await;
        assert_matches!(
            response,
            Err(crate::ReqwestClientError::ReadResponse {
                source: http_request_derive::Error::NonSuccessStatus {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    body: _
                }
            })
        );
    }

    #[tokio::test]
    async fn simple_successful_post_request() {
        #[derive(HttpRequest)]
        #[http_request(method = "POST",response = ResponseBody, path = "/post/a/resource")]
        struct Request {
            #[http_request(body)]
            body: RequestBody,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct RequestBody {
            resource: String,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct ResponseBody {
            name: String,
        }

        _ = pretty_env_logger::try_init();
        let server = httptest::Server::run();

        server.expect(
            Expectation::matching(all_of![
                request::method_path("POST", "/api/post/a/resource"),
                request::body(json_decoded(|b: &RequestBody| {
                    b == &RequestBody {
                        resource: "user".to_string(),
                    }
                })),
            ])
            .respond_with(json_encoded(json!(ResponseBody {
                name: "hello".to_string()
            }))),
        );

        let url = server
            .url("/api")
            .to_string()
            .parse()
            .expect("must be a valid url");
        let client = ReqwestClient::new(url);

        let response = client
            .execute(Request {
                body: RequestBody {
                    resource: "user".to_string(),
                },
            })
            .await
            .expect("valid response expected");

        assert_eq!(
            response,
            ResponseBody {
                name: "hello".to_string()
            }
        );
    }

    #[tokio::test]
    async fn get_base_url() {
        let url = Url::parse("http://localhost:9090/v1/api").expect("must be a valid url");
        let mut client = ReqwestClient::new(url.clone());

        assert_eq!(client.base_url(), &url);

        let new_url = Url::parse("http://localhost:9090/v2/api").expect("must be a valid url");
        client.set_base_url(new_url.clone());

        assert_eq!(client.base_url(), &new_url);
    }
}
