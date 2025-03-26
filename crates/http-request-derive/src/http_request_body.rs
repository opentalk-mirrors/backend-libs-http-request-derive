// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use http::HeaderMap;

use crate::Error;

/// Trait defined on every body type that can be used with [`crate::HttpRequest`].
pub trait HttpRequestBody {
    /// Convert the request contents to a [`Vec<u8>`].
    ///
    /// # Errors
    ///
    /// If the conversion to the [`Vec`] goes wrong, this will be indicated by
    /// returning an appropriate [`Error`].
    fn to_vec(&self) -> Result<Vec<u8>, Error>;

    /// Apply the headers that this body requires for the request to be valid.
    /// This will usually add a [`http::header::CONTENT_TYPE`] header.
    fn apply_headers(&self, _headers: &mut HeaderMap) {}
}

#[cfg(feature = "serde")]
impl<B: serde::Serialize> HttpRequestBody for B {
    fn to_vec(&self) -> Result<Vec<u8>, Error> {
        use snafu::ResultExt;
        serde_json::to_vec(self).context(crate::error::JsonSnafu)
    }

    fn apply_headers(&self, headers: &mut HeaderMap) {
        let _ = headers
            .entry(http::header::CONTENT_TYPE)
            .or_insert_with(|| http::HeaderValue::from_static("application/json"));
    }
}
