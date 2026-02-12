// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::sync::Arc;

use bytes::Bytes;
use tokio::sync::RwLock;

/// A middleware for logging HTTP requests.
#[derive(Clone)]
pub struct HttpLogger {
    backend: Arc<RwLock<dyn HttpLoggerBackend>>,
}

impl HttpLogger {
    /// Create a new HTTP logger from a backend
    pub fn new<T: HttpLoggerBackend + 'static>(backend: T) -> Self {
        Self {
            backend: Arc::new(RwLock::new(backend)),
        }
    }

    /// Create a new HTTP logger from a backend that is encapsulated in an `Arc<Rwlock<_>>`.
    ///
    /// This can be used in order to maintain access to the backend inside the `RwLock`
    /// even after passing it into the `HttpLogger`.
    pub fn new_with_locked<T: HttpLoggerBackend + 'static>(backend: Arc<RwLock<T>>) -> Self {
        Self { backend }
    }

    /// Log a HTTP request.
    pub async fn log_request(
        &self,
        start_time: std::time::SystemTime,
        request: &http::Request<Vec<u8>>,
        response: Option<&http::Response<Bytes>>,
    ) {
        let mut lock = self.backend.write().await;
        lock.log_request(start_time, request, response).await;
    }
}

impl std::fmt::Debug for HttpLogger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExampleLogger").field("inner", &()).finish()
    }
}

/// Implement this trait for middlewares that can log HTTP traffic.
#[async_trait::async_trait(?Send)]
pub trait HttpLoggerBackend {
    /// Log a HTTP request.
    async fn log_request(
        &mut self,
        start_time: std::time::SystemTime,
        request: &http::Request<Vec<u8>>,
        response: Option<&http::Response<Bytes>>,
    );
}
