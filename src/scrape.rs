use std::io::{Seek, SeekFrom};
use std::mem;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use chrono_humanize::HumanTime;
use futures::future;
use git2::{IndexAddOption, Signature, Time};
use tempfile::{NamedTempFile, TempDir, TempPath};
use tokio::{fs::File, io, time};
use zip::ZipArchive;

use crate::config::Config;
use crate::git::GitHost;
use crate::osuapi::{
    models::{Beatmap, Beatmapset, RankStatus},
    Osuapi,
};

pub struct Scraper {
    last_update_time: DateTime<Utc>,
    osuapi: Osuapi,
    config: Config,
    git: GitHost,
}

impl Scraper {
    pub fn new(osuapi: Osuapi, config: Config) -> Self {
        let git = GitHost::init(&config.repos);
        Scraper {
            last_update_time: Utc::now(),
            osuapi,
            config,
            git,
        }
    }

    pub async fn main_loop(&mut self) -> Result<()> {
        loop {
            match self.scrape_pending_beatmaps().await {
                Ok(_) => {}
                Err(err) => {
                    error!("error while scraping pending beatmaps : {}", err);
                    error!("osu {:?}", err);
                }
            };
            time::sleep(Duration::seconds(30).to_std()?).await;
        }
    }

    pub async fn scrape_pending_beatmaps(&mut self) -> Result<()> {
        debug!("scraping pending beatmaps");
        let res = self.osuapi.search_beatmaps(RankStatus::Pending).await?;

        let mut new_last_updated = self.last_update_time;
        let futs = res
            .beatmapsets
            .iter()
            .map(|b| {
                let last_updated = DateTime::parse_from_rfc3339(&b.last_updated)
                    .unwrap()
                    .with_timezone(&Utc);
                (b, last_updated)
            })
            .filter(|(_, last_updated)| *last_updated >= self.last_update_time)
            .map(|(b, last_updated)| {
                if last_updated > new_last_updated {
                    new_last_updated = last_updated;
                }

                update_beatmap_git(&self.osuapi, &self.git, b)
            });
        future::try_join_all(futs).await?;

        self.last_update_time = new_last_updated;
        debug!("updated last_updated to {:?}", new_last_updated);

        Ok(())
    }
}

async fn update_beatmap_git(osuapi: &Osuapi, git: &GitHost, mapset: &Beatmapset) -> Result<()> {
    debug!(
        "[{}] {} - {} by {}",
        mapset.id, mapset.artist, mapset.title, mapset.creator
    );
    // first, download the map
    // let temp_file = NamedTempFile::new().context("creating temp file")?;
    // let temp_path = temp_file.into_temp_path();
    // let mut temp_file = File::open(&temp_path).await.context("opening temp file")?;
    // osuapi
    //     .download_beatmapset(map.id, &mut temp_file)
    //     .await
    //     .context("downloading map to temp file")?;
    // debug!("[{}] downloaded map to {:?}", map.id, temp_path);

    // then, unpack the map contents into the git working tree
    let repo = git.open(mapset.id).context("opening git repo")?;
    // let repo_path = repo.path().to_path_buf();
    let repo_path = repo.workdir().unwrap().to_path_buf();
    assert!(!repo.is_bare());

    async fn osu(
        osuapi: &Osuapi,
        repo_path: PathBuf,
        mapset: &Beatmapset,
        map: &Beatmap,
    ) -> Result<()> {
        let path = repo_path.clone().join(format!("{}.osu", map.id));
        let mut file = File::create(&path)
            .await
            .context("creating beatmap file for download")?;
        osuapi
            .download_beatmap_file(map.id, &mut file)
            .await
            .context("downloading beatmap file")?;
        debug!("[{}] downloaded {} to {:?}", mapset.id, map.id, path);
        Ok(())
    }

    let futs = mapset
        .beatmaps
        .iter()
        .map(|b| osu(osuapi, repo_path.clone(), mapset, b));
    future::try_join_all(futs).await?;
    // let mut temp_file = temp_file.into_std().await;
    // temp_file.seek(SeekFrom::Start(0))?;
    // let mut zip = ZipArchive::new(temp_file).context("opening zip archive")?;
    // zip.extract(repo.path()).context("extracting zip archive")?;
    // debug!("[{}] extracted to work tree {:?}", map.id, repo.path());

    // now, check everything into git
    let mut index = repo.index().context("opening repo index")?;
    index
        .add_all(&["."], IndexAddOption::DEFAULT, None)
        .context("adding all files to index")?;
    let oid = index.write_tree().context("writing index to tree")?;
    debug!("[{}] checked into git", mapset.id);

    // commit to the repo
    let head = repo.head().context("retrieving repo head")?;
    let head_commit = head.peel_to_commit()?;
    let last_updated = DateTime::parse_from_rfc3339(&mapset.last_updated)
        .unwrap()
        .with_timezone(&Utc);
    let time = Time::new(last_updated.timestamp_millis(), 0);
    let sig = Signature::new(&mapset.creator, "git@osu.technology", &time)?;
    let tree = repo.find_tree(oid)?;
    let commit = repo.commit(head.name(), &sig, &sig, "update", &tree, &[&head_commit])?;
    debug!("[{}] committed as {}", mapset.id, commit);

    Ok(())
}
