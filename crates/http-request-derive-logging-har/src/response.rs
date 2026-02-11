// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use bytes::Bytes;

pub(super) fn http_response_to_har_response(
    http_response: &http::Response<Bytes>,
) -> har::v1_3::Response {
    let status = http_response.status().as_u16().into();
    let status_text = http_response.status().to_string();
    let http_version = format!("{:?}", http_response.version());
    let cookies = vec![];
    let headers: Vec<har::v1_3::Headers> = http_response
        .headers()
        .iter()
        .map(|(k, v)| har::v1_3::Headers {
            name: k.to_string(),
            value: v.to_str().unwrap_or_default().to_string(),
            comment: None,
        })
        .collect();
    let body = http_response.body();
    let text = if body.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(body).to_string())
    };

    let content = har::v1_3::Content {
        size: body.len() as i64,
        compression: None,
        mime_type: None,
        text,
        encoding: None,
        comment: None,
    };
    let redirect_url = None;
    let headers_size = headers.len() as i64;
    let body_size = body.len() as i64;
    let comment = None;
    let headers_compression = None;

    har::v1_3::Response {
        status,
        status_text,
        http_version,
        cookies,
        headers,
        content,
        redirect_url,
        headers_size,
        body_size,
        comment,
        headers_compression,
    }
}
