// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use http::HeaderMap;
use http_request_derive::HttpRequest;
use serde::Deserialize;
use url::Url;

#[derive(HttpRequest)]
#[http_request(method = "GET", response = MyResponse, path = "/books/{id}/abstract")]
struct MyRequest {
    #[http_request(query)]
    query: String,

    id: usize,
}

#[derive(HttpRequest)]
#[http_request(method="GET",response=MyResponse,path="/hello/world")]
struct AnotherRequest(
    #[http_request(query)] String,
    #[http_request(body)] String,
    #[http_request(header)] HeaderMap,
);

#[derive(Deserialize)]
struct MyResponse {}

fn main() {
    let base_url = Url::parse("https://example.com/").expect("parse url");
    let request = MyRequest {
        query: "limit=10".to_string(),
        id: 4,
    };
    let http_request = request
        .to_http_request(&base_url)
        .expect("couldn't build http request");

    // This should usually be used to send the request
    println!("{:#?}", http_request);
}
