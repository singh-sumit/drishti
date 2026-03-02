use std::path::PathBuf;

use clap::Parser;
use drishti_daemon::{LogFormat, RunOptions, config::Config, run};
use tracing_subscriber::{EnvFilter, fmt};

#[derive(Debug, Parser)]
#[command(
    name = "drishti-daemon",
    version,
    about = "Drishti observability daemon"
)]
struct Cli {
    #[arg(long, default_value = "config/drishti.toml")]
    config: PathBuf,

    #[arg(long)]
    validate_config: bool,

    #[arg(long)]
    once: bool,

    #[arg(long, value_enum, default_value_t = LogFormat::Text)]
    log_format: LogFormat,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config_preview = Config::from_path(&cli.config).unwrap_or_default();
    init_tracing(cli.log_format, &config_preview.daemon.log_level);

    let options = RunOptions {
        config_path: cli.config,
        validate_config: cli.validate_config,
        once: cli.once,
    };

    if let Err(err) = run(options).await {
        eprintln!("drishti-daemon failed: {err:#}");
        std::process::exit(1);
    }
}

fn init_tracing(log_format: LogFormat, configured_level: &str) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(configured_level));

    let subscriber = fmt().with_env_filter(filter);

    match log_format {
        LogFormat::Text => subscriber.compact().init(),
        LogFormat::Json => subscriber.json().init(),
    }
}
