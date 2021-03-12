use std::collections::HashMap;

use anyhow::Result;
use reqwest::Method;

use super::models::*;
use super::{Osuapi, BASE_URL};

impl Osuapi {
    pub async fn search_beatmaps(&self, rank_status: RankStatus) -> Result<BeatmapSearch> {
        let token = self.fetch_token().await?;
        let req = self
            .http_client
            .request(Method::GET, BASE_URL.to_owned() + "/beatmapsets/search")
            .header("authorization", "Bearer ".to_owned() + &token.access_token)
            .query(&[("s", rank_status)])
            .build()?;
        debug!("request: {:?}", req);

        Ok(self.http_client.execute(req).await?.json().await?)
    }
}
