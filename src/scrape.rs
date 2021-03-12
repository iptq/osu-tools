use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use tokio::time;
use chrono_humanize::HumanTime;

use crate::osuapi::{models::RankStatus, Osuapi};

pub struct Scraper {
    last_update_time: DateTime<Utc>,
    osuapi: Osuapi,
}

impl Scraper {
    pub fn new(osuapi: Osuapi) -> Self {
        Scraper {
            last_update_time: Utc::now(),
            osuapi,
        }
    }

    pub async fn main_loop(&mut self) -> Result<()> {
        loop {
            self.scrape_pending_beatmaps().await?;
            time::sleep(Duration::seconds(30).to_std()?).await;
        }
    }

    pub async fn scrape_pending_beatmaps(&mut self) -> Result<()> {
        debug!("scraping pending beatmaps");
        let mut res = self.osuapi.search_beatmaps(RankStatus::Pending).await?;

        debug!("new maps this round (since {:?} {}):", self.last_update_time, HumanTime::from(self.last_update_time));
        let mut new_last_updated = self.last_update_time;
        res.beatmapsets.sort_by_key(|b| b.last_updated.clone());
        for beatmap in res.beatmapsets {
            let beatmap_last_updated =
                DateTime::parse_from_rfc3339(&beatmap.last_updated)?.with_timezone(&Utc);

            // skip maps that have already been checked
            if beatmap_last_updated < self.last_update_time {
                continue;
            }

            if beatmap_last_updated > new_last_updated {
                new_last_updated = beatmap_last_updated;
            }

            debug!(" - {:?}", beatmap);
        }

        self.last_update_time = new_last_updated;
        debug!("updated last_updated to {:?}", new_last_updated);

        Ok(())
    }
}
