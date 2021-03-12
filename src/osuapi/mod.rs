mod beatmaps;
pub mod models;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use anyhow::Result;
use futures::future::Future;
use parking_lot::RwLock;
use reqwest::{header::HeaderName, Client, Method};
use serde::ser::Serialize;
use tokio::sync::Semaphore;

use crate::config::Config;

const BASE_URL: &str = "https://osu.ppy.sh/api/v2";
const TOKEN_ENDPOINT: &str = "https://osu.ppy.sh/oauth/token";

#[derive(Clone)]
pub struct Osuapi {
    http_client: Arc<Client>,
    lock: Arc<Semaphore>,
    config: Arc<Config>,
    is_fetching_token: Arc<AtomicBool>,
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
            is_fetching_token: Arc::new(AtomicBool::new(false)),
            token: Arc::new(RwLock::new(None)),
        }
    }

    /// Returns the token if it exists, or else fetches a new one
    pub async fn fetch_token(&self) -> Result<OsuToken> {
        // if it's fetching, let the current fetch complete first, it may finish faster
        self.is_fetching_token
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed);

        {
            let mut expired = false;
            if let Some((instant, token)) = self.token.read().as_ref() {
                // quickly check expiry time
                let mut expires = instant.clone();
                expires += Duration::from_secs(token.expires_in);
                if expires < Instant::now() {
                    expired = true;
                } else {
                    return Ok(token.clone());
                }
            }

            if expired {
                self.token.write().take();
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
            .header("content-type", "application/x-www-form-urlencoded")
            .form(&form)
            .build()?;
        debug!("request: {:?}", req);
        debug!(
            "request body: {:?}",
            req.body()
                .and_then(|s| s.as_bytes())
                .map(|s| std::str::from_utf8(s))
        );

        let res = self.http_client.execute(req).await?;
        let token: OsuToken = res.json().await?;

        {
            let mut token_ref = self.token.write();
            *token_ref = Some((now, token.clone()));
        }

        Ok(token)
    }

    // /// guard that locks requests to make sure it's under request limit
    // async fn request<F, R, Fut>(&self, f: F) -> Result<R>
    // where
    //     F: Fn(Arc<Client>) -> Fut,
    //     Fut: Future<Output = R>,
    // {
    //     let token = self.fetch_token();

    //     // TODO: try_acquire with exponential backoff?
    //     self.lock.acquire().await?;

    //     let res = f(self.http_client.clone()).await;

    //     Ok(res)
    // }
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenRequest<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    grant_type: &'static str,
    scope: &'static str,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct OsuToken {
    token_type: String,
    expires_in: u64,
    access_token: String,
}
