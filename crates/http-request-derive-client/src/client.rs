// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use http_request_derive::HttpRequest;

/// A client that can execute [`http_request_derive::HttpRequest`]s.
#[async_trait::async_trait(?Send)]
pub trait Client {
    /// An error that can be returned during request execution by the [`Client`].
    type ClientError: std::error::Error;

    /// Execute a [`http_request_derive::HttpRequest`], and read the typed response.
    async fn execute<R: HttpRequest + Send>(
        &self,
        request: R,
    ) -> Result<R::Response, Self::ClientError>;
}
