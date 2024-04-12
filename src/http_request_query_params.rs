// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use snafu::ResultExt;

use crate::Error;

/// A trait implemented by entities tha can build a HTTP request query string
pub trait HttpRequestQueryParams {
    /// Build the HTTP request query string
    fn http_request_query_string(&self) -> Result<Option<String>, Error>;
}

#[cfg(not(feature = "serde"))]
impl HttpRequestQueryParams for () {
    fn http_request_query_string(&self) -> Result<Option<String>, Error> {
        Ok(None)
    }
}

#[cfg(not(feature = "serde"))]
impl HttpRequestQueryParams for String {
    fn http_request_query_string(&self) -> Result<Option<String>, Error> {
        Ok(Some(self.to_string()))
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> HttpRequestQueryParams for T {
    fn http_request_query_string(&self) -> Result<Option<String>, Error> {
        let value: serde_json::Value =
            serde_json::to_value(self).context(crate::error::JsonSnafu)?;

        match value {
            serde_json::Value::Null => Ok(None),
            serde_json::Value::String(s) => Ok(Some(s)),
            serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::Array(_) => Err(crate::error::QueryStringSnafu {
                message: "query item must serialize to `null`, `string`, or `object`".to_string(),
            }
            .build()),
            serde_json::Value::Object(_) => Ok(Some(
                serde_url_params::to_string(self).context(crate::error::SerdeUrlParamsSnafu)?,
            )),
        }
    }
}
