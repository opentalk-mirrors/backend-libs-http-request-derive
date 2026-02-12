// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

pub(super) fn http_request_to_har_request(
    http_request: &http::Request<Vec<u8>>,
) -> har::v1_3::Request {
    let mut parts = http_request.uri().clone().into_parts();

    let query_string = parts
        .path_and_query
        .as_ref()
        .and_then(|paq| paq.query().map(extract_query))
        .unwrap_or_default();
    let path = parts.path_and_query.as_ref().map(|paq| paq.path());
    let path_and_query = path
        .map(http::uri::PathAndQuery::try_from)
        .transpose()
        .expect("This should be a valid path");

    parts.path_and_query = path_and_query;
    let url = http::Uri::from_parts(parts)
        .expect("valid uri parts expected")
        .to_string();

    let method = http_request.method().to_string();
    let http_version = format!("{:?}", http_request.version());
    let cookies = vec![];
    let headers: Vec<har::v1_3::Headers> = http_request
        .headers()
        .iter()
        .map(|(k, v)| har::v1_3::Headers {
            name: k.to_string(),
            value: v.to_str().unwrap_or_default().to_string(),
            comment: None,
        })
        .collect();
    let body = String::from_utf8_lossy(http_request.body()).to_string();
    let body_size = body.len() as i64;
    let post_data = if !body.is_empty() {
        Some(har::v1_3::PostData {
            // TODO: set correct mime type
            mime_type: "application/json".to_string(),
            text: Some(body),
            params: None,
            comment: None,
            encoding: None,
        })
    } else {
        None
    };
    let headers_size = headers.len() as i64;
    let comment = None;
    let headers_compression = None;

    har::v1_3::Request {
        method,
        url,
        http_version,
        cookies,
        headers,
        query_string,
        post_data,
        headers_size,
        body_size,
        comment,
        headers_compression,
    }
}

fn extract_query(query: &str) -> Vec<har::v1_3::QueryString> {
    let parts = query.split('&');

    let mut query_strings = Vec::new();

    for part in parts {
        match part.split_once('=') {
            Some((k, v)) => query_strings.push(har::v1_3::QueryString {
                name: k.to_string(),
                value: v.to_string(),
                comment: None,
            }),
            None => {
                if !part.is_empty() {
                    query_strings.push(har::v1_3::QueryString {
                        name: part.to_string(),
                        value: "".to_string(),
                        comment: None,
                    });
                }
            }
        }
    }

    query_strings
}
