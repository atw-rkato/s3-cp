use std::fmt::Debug;
use std::io;

use chrono::{Local, SecondsFormat};
use csv::Trim;
use futures::{stream, StreamExt};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::Deserialize;

use crate::error::Error;

type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Deserialize)]
struct Record {
    src_bucket: String,
    src_object_key: String,
    dst_bucket: String,
    dst_object_key: Option<String>,
}

#[derive(Debug)]
pub struct RunConfig {
    pub sync: bool,
    pub show_verbose: bool,
    pub max_pending: usize,
}

#[derive(Debug)]
pub struct RunResult {
    pub ok_cnt: usize,
    pub err_cnt: usize,
}

#[derive(Debug)]
pub struct Runner {
    s3_client: aws_sdk_s3::Client,
    config: RunConfig,
}

impl Runner {
    pub fn new(aws_config: &aws_config::Config, config: RunConfig) -> Self {
        Self {
            s3_client: aws_sdk_s3::Client::new(aws_config),
            config,
        }
    }

    pub async fn run<R: io::Read>(&self, rdr: R) -> RunResult {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .trim(Trim::All)
            .from_reader(rdr);

        let copy_results = self.for_each_copy(rdr.deserialize()).await;

        let err_cnt = copy_results.iter().filter(|&t| t.is_err()).count();

        RunResult {
            ok_cnt: copy_results.len() - err_cnt,
            err_cnt,
        }
    }

    async fn for_each_copy<T>(&self, record_parse_results: T) -> Vec<Result<()>>
    where
        T: Iterator<Item = csv::Result<Record>>,
    {
        let rows = 1_u64..;
        let copy_results =
            stream::iter(rows.zip(record_parse_results)).map(|(row, parse_result)| async move {
                let parse_and_copy_result = match parse_result {
                    Ok(Record {
                        src_bucket,
                        src_object_key,
                        dst_bucket,
                        dst_object_key,
                    }) => {
                        let src_input = src_bucket + "/" + &src_object_key;
                        let dst_object_key = dst_object_key.unwrap_or(src_object_key);
                        self.copy_object(&src_input, dst_bucket, dst_object_key)
                            .await
                    }
                    Err(e) => Err(Error::CSVParseError(e)),
                };

                if let Err(e) = &parse_and_copy_result {
                    log::error!("{row}: {}", e.display());
                }
                parse_and_copy_result
            });

        if self.config.sync || self.config.max_pending <= 1 {
            copy_results.buffered(1).collect().await
        } else {
            copy_results
                .buffer_unordered(self.config.max_pending)
                .collect()
                .await
        }
    }

    async fn copy_object(
        &self,
        src_input: &str,
        dst_bucket: impl Into<String>,
        dst_object_key: impl Into<String>,
    ) -> Result<()> {
        let dst_bucket = dst_bucket.into();
        let dst_object_key = dst_object_key.into();

        if self.config.show_verbose {
            let now = Local::now();
            let ts = now.to_rfc3339_opts(SecondsFormat::Millis, false);
            println!("{ts} copy from {src_input} to {dst_bucket}/{dst_object_key}");
        }

        let src_input_encoded = utf8_percent_encode(src_input, NON_ALPHANUMERIC).to_string();
        let copy_object_request = self
            .s3_client
            .copy_object()
            .copy_source(src_input_encoded)
            .bucket(dst_bucket)
            .key(dst_object_key);

        if let Err(e) = copy_object_request.send().await {
            Err(Error::S3CopyError(e))
        } else {
            Ok(())
        }
    }
}
