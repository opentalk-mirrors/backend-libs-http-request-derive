use bytes::Bytes;
use http::uri::InvalidUri;
use snafu::Snafu;
use url::Url;

/// Errors that originate from this crate
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    /// Encountered a `401 UNAUTHORIZED` http status code
    #[snafu(display("trying to perform an unauthorized request"))]
    Unauthorized,

    /// Encountered a non-success http status code that was not handled otherwise
    #[snafu(display("server returned a non-success http status code {status}"))]
    NonSuccessStatus {
        /// The returned status code.
        status: http::StatusCode,

        /// The data returned from the request
        data: Bytes,
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
}
