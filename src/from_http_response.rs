// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use bytes::Bytes;
use snafu::ResultExt;

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
        serde_json::from_slice(http_response.body()).context(crate::error::JsonSnafu)
    }
}
