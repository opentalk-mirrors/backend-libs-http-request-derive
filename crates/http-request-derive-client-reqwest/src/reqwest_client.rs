// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use http_request_derive::HttpRequest;
use http_request_derive_client::Client;
use snafu::ResultExt as _;
use url::Url;

use crate::{
    reqwest_client_error::{
        BuildHttpResponseBodySnafu, ConvertToHttpRequestSnafu, ConvertToReqwestRequestSnafu,
        ReadResponseSnafu, RequestExecutionSnafu, RetrieveResponseBodySnafu,
    },
    ReqwestClientError,
};

/// A client for executing requests as defined by [`http_request_derive::HttpRequest`] implementations.
#[derive(Debug)]
pub struct ReqwestClient {
    client: reqwest::Client,
    base_url: Url,
}

impl ReqwestClient {
    /// Create a new [`ReqwestClient`] from a base [`Url`].
    pub fn new(base_url: Url) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    /// Returns the base URL which is used for subsequent requests.
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Sets the base URL to the given URL.
    pub fn set_base_url(&mut self, base_url: Url) {
        self.base_url = base_url
    }
}

#[async_trait::async_trait]
impl Client for ReqwestClient {
    type ClientError = ReqwestClientError;

    async fn execute<R: HttpRequest + Send>(
        &self,
        request: R,
    ) -> Result<R::Response, Self::ClientError> {
        let request = request
            .to_http_request(&self.base_url)
            .context(ConvertToHttpRequestSnafu)?;
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
        R::read_response(http_response).context(ReadResponseSnafu)
    }
}

#[cfg(test)]
mod tests {
    use http::StatusCode;
    use http_request_derive::HttpRequest;
    use http_request_derive_client::Client as _;
    use httptest::{
        all_of,
        matchers::{json_decoded, request},
        responders::{json_encoded, status_code},
        Expectation,
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
                    data: _
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
