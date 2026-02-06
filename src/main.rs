use clap::Parser;
use directories::ProjectDirs;
use std::io::Write;
use anyhow::Context;
use figment::{Figment, providers::{Format, Toml, Serialized}};

use crate::config::{Config, ConfigCliOverride};

mod config;
mod oidc;
mod ssh;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, global = true, help = "Enable verbose output")]
    verbose: bool,
    #[command(subcommand)]
    command: ssh::Commands,
    #[command(flatten)]
    pub config_overrides: ConfigCliOverride,
}

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format(|buf, record| {
            writeln!(buf, "{}", record.args())
        })
        .init();

    let cli = Cli::parse();

    let proj_dirs = ProjectDirs::from("ch", "cscs", "cscs-key")
        .context("Could not determine configuration directory")?;
    let config_dir = proj_dirs.config_dir();
    let config_file_path = config_dir.join("config.toml");

    //let config = config::Config::load()?;
    let config: Config = Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file(config_file_path))
        .merge(Serialized::defaults(&cli.config_overrides))
        .extract()?;

    if cli.verbose {
        println!("Verbose output ...");
    }

    ssh::run(&cli.command, &config)?;

    Ok(())
}
