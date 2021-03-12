use std::collections::HashMap;

use anyhow::Result;
use reqwest::{Method, StatusCode};
use tokio::{
    fs::File,
    io::{AsyncWrite, AsyncWriteExt},
};

use super::models::*;
use super::{Osuapi, BASE_URL};

impl Osuapi {
    pub async fn search_beatmaps(&self, rank_status: RankStatus) -> Result<BeatmapSearch> {
        let token = self.fetch_token().await?;
        let req = self
            .http_client
            .request(Method::GET, BASE_URL.to_owned() + "/beatmapsets/search")
            .header("Authorization", "Bearer ".to_owned() + &token.access_token)
            .query(&[("s", rank_status)])
            .build()?;

        Ok(self.http_client.execute(req).await?.json().await?)
    }

    pub async fn download_beatmap_file<W>(&self, map_id: i32, mut writer: W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        let token = self.fetch_token().await?;
        let req = self
            .http_client
            .request(Method::GET, format!("https://osu.ppy.sh/osu/{}", map_id))
            .header("Authorization", "Bearer ".to_owned() + &token.access_token)
            .build()?;
        let mut res = self.http_client.execute(req).await?;
        if res.status() != StatusCode::OK {
            bail!("failed to download : {}", res.text().await?);
        } else {
            while let Some(chunk) = res.chunk().await? {
                writer.write_all(chunk.as_ref()).await?;
            }
        }
        Ok(())
    }

    pub async fn download_beatmapset<W>(&self, mapset_id: i32, mut writer: W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        let token = self.fetch_token().await?;
        let req = self
            .http_client
            .request(
                Method::GET,
                format!("{}/beatmapsets/{}/download", BASE_URL, mapset_id),
            )
            .header("Authorization", "Bearer ".to_owned() + &token.access_token)
            .build()?;
        let mut res = self.http_client.execute(req).await?;
        if res.status() != StatusCode::OK {
            bail!("failed to download : {}", res.text().await?);
        } else {
            while let Some(chunk) = res.chunk().await? {
                writer.write_all(chunk.as_ref()).await?;
            }
        }
        Ok(())
    }
}
