// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use snafu::Snafu;

/// Errors that result from performing HTTP requests through the [`crate::ReqwestClient`].
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ReqwestClientError {
    /// The conversion from a [`http_request_derive::HttpRequest`] to a [`http::Request`] has failed.
    #[snafu(display("Error converting http-request-derive HttpRequest to http Request"))]
    ConvertToHttpRequest {
        /// The source error
        source: http_request_derive::Error,
    },

    /// The conversion from a [`http::Request`] to a [`reqwest::Request`] has failed.
    #[snafu(display("Error converting http Request to reqwest Request"))]
    ConvertToReqwestRequest {
        /// The source error
        source: reqwest::Error,
    },

    /// The request execution has failed.
    #[snafu(display("Request execution error"))]
    RequestExecution {
        /// The source error
        source: reqwest::Error,
    },

    /// Retrieval of the response body from the [`reqwest::Response`] has failed.
    #[snafu(display("Failed to retrieve response body"))]
    RetrieveResponseBody {
        /// The source error
        source: reqwest::Error,
    },

    /// Building the [`http::Response`] has failed.
    #[snafu(display("Failed to store body from response into http Response"))]
    BuildHttpResponseBody {
        /// The source error
        source: http::Error,
    },

    /// Reading the response from the [`http::Response`] has failed.
    #[snafu(display("Failed to read the response for the specific request"))]
    ReadResponse {
        /// The source error
        source: http_request_derive::Error,
    },
}
