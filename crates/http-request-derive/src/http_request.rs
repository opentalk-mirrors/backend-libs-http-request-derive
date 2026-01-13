// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use bytes::Bytes;
use http::{HeaderMap, Uri};
use snafu::{OptionExt, ResultExt};
use url::Url;

use crate::{
    Error, FromHttpResponse, HttpRequestBody, HttpRequestQueryParams,
    error::{BuildRequestSnafu, ParseUriSnafu, UrlCannotBeABaseSnafu},
};

/// A trait implemented for types that are sent to the API as parameters
pub trait HttpRequest {
    /// The response type that is expected to the request
    type Response: FromHttpResponse;

    /// The query type that is sent to the API endpoint
    type Query: HttpRequestQueryParams;

    /// The body type that is sent to the API endpoint
    type Body: HttpRequestBody;

    /// The method used to send the data to the API endpoint
    const METHOD: http::Method;

    /// Get the API endpoint path relative to the base URL
    fn path(&self) -> String;

    /// Get query parameters for the `http::Request`
    fn query(&self) -> Option<&Self::Query> {
        None
    }

    /// Get the body for the `http::Request`
    fn body(&self) -> Option<&Self::Body> {
        None
    }

    /// Get the headers for the `http::Request`
    fn apply_headers(&self, headers: &mut HeaderMap) {
        if let Some(body) = self.body() {
            body.apply_headers(headers);
        }
    }

    /// Build a HTTP request from the request type
    fn to_http_request(&self, base_url: &Url) -> Result<http::request::Request<Vec<u8>>, Error> {
        let body = self
            .body()
            .map(HttpRequestBody::to_vec)
            .transpose()?
            .unwrap_or_default();

        let uri = {
            let mut url = base_url.clone();
            {
                let mut segments =
                    url.path_segments_mut()
                        .ok()
                        .with_context(|| UrlCannotBeABaseSnafu {
                            url: base_url.clone(),
                        })?;
                let _ = segments
                    .pop_if_empty()
                    .extend(self.path().split('/').skip_while(|s| s.is_empty()));
            }

            let query = self
                .query()
                .map(HttpRequestQueryParams::http_request_query_string)
                .transpose()?;
            if let Some(query) = query {
                let query = query.as_deref();
                url.set_query(query);
            }

            url.as_str().parse::<Uri>().context(ParseUriSnafu)?
        };

        let mut headers = HeaderMap::new();
        self.apply_headers(&mut headers);

        let mut builder = http::request::Request::builder()
            .method(Self::METHOD)
            .uri(uri);

        for (name, value) in &headers {
            builder = builder.header(name, value);
        }

        builder.body(body).context(BuildRequestSnafu)
    }

    /// Convert the response from a `http::Response`
    fn read_response(response: http::Response<Bytes>) -> Result<Self::Response, Error> {
        Self::Response::from_http_response(response)
    }
}
