use serde::Deserialize;

#[derive(Deserialize)]
pub struct Release {
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Deserialize)]
pub struct ReleaseAsset {
    pub url: String,
}
