// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use async_trait::async_trait;
use bytes::Bytes;

use crate::Error;

/// Trait for types that can be converted from `http::Response`
#[async_trait]
pub trait FromHttpResponse {
    /// Convert from `http::Response` to our `Response`
    ///
    /// # Errors
    ///
    /// The implementation of this trait will map the response to an error if it should be interpreted as such.
    /// Typical HTTP status code errors are read by the default implementation of [`crate::HttpRequest::read_response`]
    /// already, so in most cases additional checks are not necessary here.
    ///
    /// Of course if the contents of the response cannot be parsed, this will usually be handled as an
    /// error as well.
    fn from_http_response(http_response: http::Response<Bytes>) -> Result<Self, Error>
    where
        Self: Sized;

    /// Convert from `reqwest::Response` to our `Response`
    ///
    /// # Errors
    ///
    /// The implementation of this trait will map the response to an error if it should be interpreted as such.
    /// Typical HTTP status code errors are read by the default implementation of [`crate::HttpRequest::read_response`]
    /// already, so in most cases additional checks are not necessary here.
    ///
    /// Of course if the contents of the response cannot be parsed, this will usually be handled as an
    /// error as well.
    #[cfg(feature = "reqwest")]
    async fn from_reqwest_response(response: reqwest::Response) -> Result<Self, Error>
    where
        Self: Sized;
}

#[cfg(feature = "serde")]
#[async_trait]
impl<D> FromHttpResponse for D
where
    D: serde::de::DeserializeOwned,
{
    fn from_http_response(http_response: http::Response<Bytes>) -> Result<Self, Error> {
        use snafu::ResultExt as _;
        serde_json::from_slice(http_response.body()).context(crate::error::JsonSnafu)
    }

    #[cfg(feature = "reqwest")]
    async fn from_reqwest_response(response: reqwest::Response) -> Result<Self, Error>
    where
        Self: Sized,
    {
        use snafu::ResultExt as _;

        use crate::error::ReqwestSnafu;

        serde_json::from_slice(&response.bytes().await.context(ReqwestSnafu {
            message: "Failed to receive success response",
        })?)
        .context(crate::error::JsonSnafu)
    }
}

#[cfg(all(test, feature = "reqwest", feature = "serde"))]
mod serde_reqwest_tests {
    use reqwest::Client;
    use serde::Deserialize;

    use super::*;

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestStruct {
        id: i32,
        name: String,
    }

    #[tokio::test]
    async fn test_from_reqwest_response_success() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id": 1, "name": "Test"}"#)
            .create_async()
            .await;

        let client = Client::new();
        let response = client
            .get(format!("{}/test", server.url()))
            .send()
            .await
            .unwrap();

        let result = TestStruct::from_reqwest_response(response).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            TestStruct {
                id: 1,
                name: "Test".to_string()
            }
        );
    }

    #[tokio::test]
    async fn test_from_reqwest_response_invalid_json() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id": 1, "name": "Test"#)
            .create_async()
            .await;

        let client = Client::new();
        let response = client
            .get(format!("{}/test", server.url()))
            .send()
            .await
            .unwrap();

        let result = TestStruct::from_reqwest_response(response).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Json { .. }));
    }
}
