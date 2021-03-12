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

use anyhow::Result;
use tokio::sync::oneshot;

use crate::config::Config;
use crate::osuapi::Osuapi;
use crate::scrape::Scraper;

#[tokio::main]
async fn main() -> Result<()> {
    stderrlog::new()
        .module(module_path!())
        .verbosity(5)
        .init()
        .unwrap();

    let (exit_tx, exit_rx) = oneshot::channel::<()>();

    let mut file = File::open("config.toml")?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    let config: Config = toml::from_slice(&contents)?;

    let osuapi = Osuapi::new(config.clone());

    let mut scraper = Scraper::new(osuapi.clone(), config.clone());
    tokio::spawn(async move {
        scraper.main_loop().await.unwrap();
    });

    exit_rx.await?;
    Ok(())
}
