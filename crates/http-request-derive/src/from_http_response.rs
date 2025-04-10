// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use bytes::Bytes;

use crate::Error;

/// Trait for types that can be converted from `http::Response`
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
}

#[cfg(feature = "serde")]
impl<D> FromHttpResponse for D
where
    D: serde::de::DeserializeOwned,
{
    fn from_http_response(http_response: http::Response<Bytes>) -> Result<Self, Error> {
        use snafu::ResultExt as _;
        serde_json::from_slice(http_response.body()).context(crate::error::JsonSnafu)
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use http::StatusCode;
    use serde::Deserialize;

    use super::*;

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestStruct {
        id: i32,
        name: String,
    }

    #[tokio::test]
    async fn test_from_response_success() {
        let response = http::Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(Bytes::from_static(r#"{"id":1,"name":"Test"}"#.as_bytes()))
            .expect("valid response required");

        let result = TestStruct::from_http_response(response);
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
    async fn test_from_response_invalid_json() {
        let response = http::Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(Bytes::from_static(r#"{"id":1,"name":"Test"#.as_bytes()))
            .expect("valid response required");

        let result = TestStruct::from_http_response(response);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Json { .. }));
    }
}
