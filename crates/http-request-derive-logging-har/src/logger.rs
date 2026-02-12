// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{fs::create_dir_all, path::Path};

use bytes::Bytes;
use http_request_derive_logging::HttpLoggerBackend;
use jiff::{Zoned, fmt::strtime};
use log::info;
use snafu::{ResultExt as _, Snafu};

use crate::{request::http_request_to_har_request, response::http_response_to_har_response};

/// A logger for storing har files
#[derive(Debug)]
pub struct HarLogger {
    har: har::v1_3::Log,
}

impl HarLogger {
    /// Create a new [`HarLogger`].
    pub fn new(name: String, version: String) -> Self {
        Self {
            har: har::v1_3::Log {
                creator: har::v1_3::Creator {
                    name,
                    version,
                    comment: None,
                },
                pages: Some(Vec::new()),
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Snafu)]
pub enum HarLoggerDumpError {
    #[snafu(display("Directory {path:?} could not be created"))]
    CreateDirectory {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    HarExport {
        source: har::Error,
    },
    #[snafu(display("File {path:?} could not be written"))]
    WriteFile {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
}

impl HarLogger {
    /// Write the collected har data into a file.
    pub fn write_to_file(&self, path: &Path) -> Result<(), HarLoggerDumpError> {
        info!("Dumping har file to {path:?}");

        if let Some(parent) = path.parent() {
            create_dir_all(parent).context(CreateDirectorySnafu {
                path: parent.to_path_buf(),
            })?;
        }

        let har = har::Har {
            log: har::Spec::V1_3(self.har.clone()),
        };

        let dump = har::to_json(&har).context(HarExportSnafu)?;
        std::fs::write(path, dump).context(WriteFileSnafu {
            path: path.to_path_buf(),
        })?;
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl HttpLoggerBackend for HarLogger {
    /// Log a HTTP request.
    async fn log_request(
        &mut self,
        start_time: std::time::SystemTime,
        request: &http::Request<Vec<u8>>,
        response: Option<&http::Response<Bytes>>,
    ) {
        let start = Zoned::try_from(start_time).expect("valid start time expected");
        let duration = start.duration_until(&Zoned::now());

        let har_request = http_request_to_har_request(request);
        let har_response = response
            .map(http_response_to_har_response)
            .unwrap_or_default();

        self.har.entries.push(har::v1_3::Entries {
            pageref: None,
            started_date_time: strtime::format("%Y-%m-%dT%H:%M:%S%.f%:z", &start)
                .expect("valid time format required"),
            time: duration.as_millis_f64(),
            request: har_request,
            response: har_response,
            cache: har::v1_3::Cache::default(),
            timings: har::v1_3::Timings::default(),
            server_ip_address: None,
            connection: None,
            comment: None,
        });
    }
}
