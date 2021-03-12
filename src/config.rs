use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub debug: bool,
    pub repos: PathBuf,
    pub db_path: PathBuf,

    pub oauth_client_id: String,
    pub oauth_client_secret: String,

    pub host: String,
    pub port: u16,
    pub session_secret: String,
}
