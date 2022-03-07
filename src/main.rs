extern crate core;

use std::io::BufReader;
use std::time::Instant;
use std::{env, io};

use aws_sdk_s3::Region;
use chrono::{Local, SecondsFormat};
use clap::Parser;
use log::LevelFilter;

use crate::runner::{RunConfig, RunResult, Runner};

mod error;
mod runner;

const AWS_PROFILE_ENV_KEY: &str = "AWS_PROFILE";

#[derive(Parser, Debug)]
#[clap(version, about)]
struct Args {
    /// AWS profile. default is
    #[clap(short, long, name = "AWS_PROFILE", env = AWS_PROFILE_ENV_KEY)]
    profile: String,

    /// S3 Region.  default is
    #[clap(short, long, name = "S3_REGION", env = "AWS_DEFAULT_REGION")]
    region: String,

    /// Show copy verbose in process
    #[clap(short, long)]
    verbose: bool,

    /// run sequential (may take a very long time)
    #[clap(long)]
    sync: bool,

    /// upper limit of concurrent request
    #[clap(short, long, name = "LIMIT", default_value_t = 128)]
    max_pending: usize,
}

#[tokio::main]
async fn main() {
    let start = Instant::now();

    let args: Args = Args::parse();
    init_logger();

    env::set_var(AWS_PROFILE_ENV_KEY, args.profile);
    let aws_config = aws_config::from_env()
        .region(Region::new(args.region))
        .load()
        .await;

    let run_config = RunConfig {
        sync: args.sync,
        show_verbose: args.verbose,
        max_pending: args.max_pending,
    };
    let runner = Runner::new(&aws_config, run_config);

    let RunResult { ok_cnt, err_cnt } = runner.run(BufReader::new(io::stdin())).await;

    let elapsed = start.elapsed();
    let elapsed_sec = elapsed.as_secs_f64().round();
    println!();
    println!("Copied: {ok_cnt}  Errors: {err_cnt}  took {elapsed_sec}s");
}

fn init_logger() {
    use std::io::Write;

    env::set_var("RUST_LOG", LevelFilter::Error.as_str().to_lowercase());
    env_logger::builder()
        .format(|buf, record| {
            let ts = Local::now();
            let level = record.level();
            let level_style = buf.default_level_style(level);
            writeln!(
                buf,
                "{ts} {level} [{target}] {args} ({file}:{line})",
                ts = ts.to_rfc3339_opts(SecondsFormat::Millis, false),
                level = level_style.value(level),
                target = record.target(),
                args = level_style.value(record.args()),
                file = record.file().unwrap_or("unknown"),
                line = record.line().unwrap_or(0),
            )
        })
        .init();
}
