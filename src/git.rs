use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use git2::{Repository, Signature, Time};

pub struct GitHost {
    root: PathBuf,
}

impl GitHost {
    pub fn init(path: impl AsRef<Path>) -> Self {
        GitHost {
            root: path.as_ref().to_path_buf(),
        }
    }

    pub fn open(&self, mapset_id: i32) -> Result<Repository> {
        let path = self.root.join(mapset_id.to_string());

        let repo = if !path.exists() {
            debug!("[{}] creating empty repo", mapset_id);
            fs::create_dir_all(&path).context("creating dirs to repo")?;
            let repo = Repository::init(&path).context("initializing new repo")?;
            // create initial commit
            {
                let now = Utc::now();
                let time = Time::new(now.timestamp_millis(), 0);
                let sig = Signature::new("osugit", "git@osu.technology", &time)
                    .context("creating signature")?;
                let mut index = repo.index().context("opening index")?;
                let oid = index.write_tree().context("writing empty index to tree")?;
                let tree = repo.find_tree(oid).context("retrieving tree for index")?;
                repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
                    .context("creating initial commit")?;
                debug!("[{}] committed initial {}", mapset_id, oid);
            }
            repo
        } else {
            Repository::open(path).context("opening existing repo")?
        };

        Ok(repo)
    }
}
