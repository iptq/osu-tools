mod beatmaps;
pub mod models;

use std::mem;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use anyhow::Result;
use futures::future::Future;
use reqwest::{header::HeaderName, Client, Method};
use serde::ser::Serialize;
use tokio::sync::{RwLock, Semaphore};

use crate::config::Config;

const BASE_URL: &str = "https://osu.ppy.sh/api/v2";
const TOKEN_ENDPOINT: &str = "https://osu.ppy.sh/oauth/token";

#[derive(Clone)]
pub struct Osuapi {
    http_client: Arc<Client>,
    lock: Arc<Semaphore>,
    config: Arc<Config>,
    is_fetching_token: Arc<Semaphore>,
    token: Arc<RwLock<Option<(Instant, OsuToken)>>>,
}

impl Osuapi {
    pub fn new(config: Config) -> Self {
        // the OSU api caps requests at 1000 with burst limit of 1200
        let lock = Semaphore::new(1000);

        Osuapi {
            http_client: Arc::new(Client::new()),
            lock: Arc::new(lock),
            config: Arc::new(config),
            is_fetching_token: Arc::new(Semaphore::new(1)),
            token: Arc::new(RwLock::new(None)),
        }
    }

    /// Returns the token if it exists, or else fetches a new one
    pub async fn fetch_token(&self) -> Result<OsuToken> {
        // if it's fetching, let the current fetch complete first, it may finish faster
        let permit = self.is_fetching_token.acquire().await?;

        // check the currently stored value for the token
        {
            let mut expired = false;
            if let Some((instant, token)) = self.token.read().await.as_ref() {
                // quickly check expiry time
                let mut expires = instant.clone();
                expires += Duration::from_secs(token.expires_in);
                if expires < Instant::now() {
                    expired = true;
                } else {
                    return Ok(token.clone());
                }
            }

            // if it's expired, "take" the current token out and store a new one
            if expired {
                self.token.write().await.take();
            }
        }

        let now = Instant::now();
        let form = TokenRequest {
            client_id: &self.config.oauth_client_id,
            client_secret: &self.config.oauth_client_secret,
            grant_type: "client_credentials",
            scope: "public",
        };
        let req = self
            .http_client
            .request(Method::POST, TOKEN_ENDPOINT)
            .header("content-type", "application/json")
            .json(&form)
            .build()?;

        let res = self.http_client.execute(req).await?;
        let token: OsuToken = res.json().await?;

        {
            let mut token_ref = self.token.write().await;
            *token_ref = Some((now, token.clone()));
        }

        // release the lock on the token
        mem::drop(permit);

        Ok(token)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenRequest<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    grant_type: &'static str,
    scope: &'static str,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OsuToken {
    token_type: String,
    expires_in: u64,
    access_token: String,
}
