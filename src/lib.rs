//! `HttpRequest` derive

#![deny(
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_interfaces,
    private_bounds,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

#[allow(unused_extern_crates)]
extern crate self as http_request_derive;

mod error;
mod from_http_response;
mod http_request;
mod http_request_body;
mod http_request_query_params;

#[doc(hidden)]
pub mod __exports {
    pub use http;
}

pub use error::Error;
pub use from_http_response::FromHttpResponse;
pub use http_request::HttpRequest;
pub use http_request_body::HttpRequestBody;
pub use http_request_query_params::HttpRequestQueryParams;

pub use http_request_derive_macros::HttpRequest;

#[cfg(test)]
mod tests {
    use http::HeaderValue;
    use pretty_assertions::assert_eq;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use url::Url;

    use crate::HttpRequest;

    #[test]
    fn simple_get_request() {
        #[derive(HttpRequest)]
        #[http_request(method="GET", response = String, path = "/info")]
        struct SimpleGetRequest;

        let base_url = Url::parse("https://example.com/").expect("parse url");

        let request = SimpleGetRequest;

        let http_request = request
            .to_http_request(&base_url)
            .expect("build http request");

        assert_eq!(&http_request.uri().to_string(), "https://example.com/info");
        assert_eq!(http_request.method(), http::Method::GET);
        assert!(http_request.into_body().is_empty());
    }

    #[test]
    fn simple_json_post_request() {
        #[derive(Serialize)]
        struct MyRequestQuery {
            start_x: usize,
            end_x: usize,
            start_y: usize,
            end_y: usize,
        }

        #[derive(Serialize)]
        struct MyRequestBody {
            x: usize,
            y: usize,
        }

        #[derive(HttpRequest)]
        #[http_request(method = "POST", response = MyResponse, path = "/books/{id}/abstract")]
        struct MyRequest {
            #[http_request(query)]
            query: MyRequestQuery,

            #[http_request(body)]
            body: MyRequestBody,

            id: usize,
        }

        #[derive(Deserialize)]
        struct MyResponse {}

        let base_url = Url::parse("https://example.com/").expect("parse url");
        let query = MyRequestQuery {
            start_x: 10,
            end_x: 50,
            start_y: 20,
            end_y: 30,
        };
        let body = MyRequestBody { x: 10, y: 20 };
        let request = MyRequest {
            query,
            body,
            id: 55,
        };

        let http_request = request
            .to_http_request(&base_url)
            .expect("build http request");

        assert_eq!(
            &http_request.uri().to_string(),
            "https://example.com/books/55/abstract?start_x=10&end_x=50&start_y=20&end_y=30"
        );
        assert_eq!(http_request.method(), http::Method::POST);
        assert_eq!(http_request.headers().len(), 1);
        assert_eq!(
            http_request.headers().get("content-type"),
            Some(&HeaderValue::from_str("application/json").unwrap())
        );

        let http_request_body: serde_json::Value =
            serde_json::from_slice(&http_request.into_body()).expect("json");
        assert_eq!(http_request_body, json!({"x": 10, "y": 20}));
    }
}
