#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeatmapSearch {
    pub beatmapsets: Vec<Beatmapset>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Beatmapset {
    pub id: i32,
    pub artist: String,
    pub artist_unicode: String,
    pub title: String,
    pub title_unicode: String,
    pub creator: String,
    pub user_id: i32,

    pub covers: BeatmapCovers,
    #[serde(default)]
    pub beatmaps: Vec<Beatmap>,
    #[serde(default)]
    pub last_updated: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Beatmap {
    pub id: i32,
    pub difficulty_rating: f64,
    pub version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeatmapCovers {
    pub cover: String,
    #[serde(rename = "cover@2x")]
    pub cover_2x: String,

    pub card: String,
    #[serde(rename = "card@2x")]
    pub card_2x: String,

    pub slimcover: String,
    #[serde(rename = "slimcover@2x")]
    pub slimcover_2x: String,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum RankStatus {
    #[serde(rename = "graveyard")]
    Graveyard = -2,
    Wip = -1,
    #[serde(rename = "pending")]
    Pending = 0,
    #[serde(rename = "ranked")]
    Ranked = 1,
    Approved = 2,
    #[serde(rename = "qualified")]
    Qualified = 3,
    #[serde(rename = "loved")]
    Loved = 4,
}
