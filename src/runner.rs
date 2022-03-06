use std::io;

use anyhow::{bail, Result};
use chrono::{Local, SecondsFormat};
use csv::Trim;
use futures::{stream, StreamExt};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::Deserialize;

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
            .trim(Trim::All)
            .has_headers(false)
            .comment(Some(b'#'))
            .from_reader(rdr);

        let copy_results = self.for_each_copy(rdr.deserialize()).await;

        let err_cnt = copy_results.iter().filter(|&t| t.is_err()).count();

        RunResult {
            ok_cnt: copy_results.len() - err_cnt,
            err_cnt,
        }
    }

    async fn for_each_copy<T>(&self, input_records: T) -> Vec<Result<()>>
    where
        T: Iterator<Item = csv::Result<Record>>,
    {
        let copy_results =
            stream::iter((1_u64..).zip(input_records)).map(|(row, csv_parse_result)| async move {
                let copy_result = self.try_copy(csv_parse_result).await;
                if let Err(e) = &copy_result {
                    log::error!("{row}: {e}");
                }
                copy_result
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

    async fn try_copy(&self, csv_parse_result: csv::Result<Record>) -> Result<()> {
        match csv_parse_result {
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
            Err(e) => {
                bail!(e)
            }
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

        let copy_object_request = self
            .s3_client
            .copy_object()
            .copy_source(utf8_percent_encode(src_input, NON_ALPHANUMERIC).to_string())
            .bucket(dst_bucket)
            .key(dst_object_key);

        copy_object_request.send().await?;

        Ok(())
    }
}
