# http-request-derive rust library crate

Attention: this crate is still under development.

You can derive `HttpRequest` on a struct and annotate it with some attributes,
so that it can be used to build a
[`http::Request`](https://docs.rs/http/latest/http/request/struct.Request.html)
which can then be sent to a server. In addition, a response type can be read from
the received
[`http::Response`](https://docs.rs/http/latest/http/response/struct.Response.html).
