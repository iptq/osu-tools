#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_repr;

mod config;
mod db;
mod git;
mod osuapi;
mod scrape;
mod web;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use anyhow::{Context, Result};
use structopt::StructOpt;
use tokio::sync::oneshot;

use crate::config::Config;
use crate::osuapi::Osuapi;
use crate::scrape::Scraper;

#[derive(StructOpt)]
struct Opt {
    /// Path to the config file (defaults to config.toml in current dir)
    #[structopt(short = "c", long = "config")]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    stderrlog::new()
        .module(module_path!())
        .verbosity(5)
        .init()
        .unwrap();

    let (exit_tx, exit_rx) = oneshot::channel::<()>();

    let config_path = opt.config.unwrap_or_else(|| PathBuf::from("config.toml"));
    let mut file = File::open(&config_path)
        .with_context(|| format!("could not open config file {:?}", config_path))?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    let config: Config = toml::from_slice(&contents)?;

    let osuapi = Osuapi::new(config.clone());

    let mut scraper = Scraper::new(osuapi.clone(), config.clone());
    tokio::spawn(async move {
        scraper.main_loop().await.unwrap();
    });

    tokio::spawn(async move {
        web::run_web(config.clone());
    });

    exit_rx.await?;
    Ok(())
}
