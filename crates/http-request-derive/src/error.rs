// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use bytes::Bytes;
use http::{uri::InvalidUri, StatusCode};
use snafu::{Location, Snafu};
use url::Url;

/// Errors that originate from this crate
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    /// Encountered a non-success http status code that was not handled otherwise
    #[snafu(display("server returned a non-success http status code {status}"))]
    NonSuccessStatus {
        /// The returned status code.
        status: StatusCode,

        /// The body returned from the request
        body: Bytes,
    },

    /// An error occurred when building a HTTP request
    #[snafu(display("could not build http request: {source}"))]
    BuildRequest {
        /// The source http error
        source: http::Error,
    },

    /// Encountered a URL which cannot be a base where a base url was required
    #[snafu(display("base url {url} cannot be a base"))]
    UrlCannotBeABase {
        /// The url which cannot be a base
        url: Url,
    },

    /// Couldn't parse a HTTP URI
    #[snafu(display("couldn't parse uri"))]
    ParseUri {
        /// The source invalid uri error
        source: InvalidUri,
    },

    /// A query string couldn't be created from the given type
    #[snafu(display("can't create query string: {message}"))]
    QueryString {
        /// A message describing the reason for this error
        message: String,
    },

    /// Couldn't create a query from a given string
    #[cfg(feature = "serde")]
    #[snafu(display("couldn't build query string from serde url params"))]
    SerdeUrlParams {
        /// The source of the serde_url_params error
        source: serde_url_params::Error,
    },

    /// serde_json error
    #[cfg(feature = "serde")]
    #[snafu(display("serde json error"))]
    Json {
        /// The source of the serde_json error
        source: serde_json::Error,
    },

    /// custom error returned e.g. by a custom trait implementation in a different crate
    #[snafu(display("{message}"))]
    Custom {
        /// The custom error message
        message: String,

        /// The location where the error happened
        location: Location,
    },
}

impl Error {
    /// Create a custom error
    #[track_caller]
    pub fn custom(message: String) -> Self {
        let location = std::panic::Location::caller();

        Error::Custom {
            message,
            location: Location::new(location.file(), location.line(), location.column()),
        }
    }

    /// Query whether the error is caused by an HTTP Unauthorized status code
    pub const fn is_unauthorized(&self) -> bool {
        self.is_specific_http_error_status(&StatusCode::UNAUTHORIZED)
    }

    /// Query whether the error is caused by a HTTP NotFound status code
    pub const fn is_not_found(&self) -> bool {
        self.is_specific_http_error_status(&StatusCode::NOT_FOUND)
    }

    /// Query whether the error is caused by a HTTP BadRequest status code
    pub const fn is_bad_request(&self) -> bool {
        self.is_specific_http_error_status(&StatusCode::BAD_REQUEST)
    }

    /// Query whether the error is caused by a HTTP InternalServerError status code
    pub const fn is_internal_server_error(&self) -> bool {
        self.is_specific_http_error_status(&StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Query whether the error is caused by a non-success HTTP status code
    pub const fn is_http_error_status(&self) -> bool {
        matches!(self, Self::NonSuccessStatus { .. })
    }

    /// Query whether the error is caused by a specific HTTP status code
    pub const fn is_specific_http_error_status(&self, specific_status: &StatusCode) -> bool {
        matches!(self, Self::NonSuccessStatus { status,.. } if status.as_u16() == specific_status.as_u16())
    }
}
