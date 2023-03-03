#[derive(Clone, Debug)]
pub enum CrateVersion {
    Latest,
    Specific(semver::Version)
}

// Crates.io API
#[derive(Deserialize)]
struct CratesApiCrate {
    pub newest_version: String,
}

#[derive(Deserialize)]
struct CratesApiResponse {
    #[serde(rename="crate")]
    pub value: CratesApiCrate,
}

pub async fn get_latest_version(crt: &str) -> String {
    let api = reqwest::Client::new().get(
        format!("https://crates.io/api/v1/crates/{}", crt))
        .header("User-Agent", format!("foof/{} (https://github.com/ry00001/fluoroperoxide)", env!("CARGO_PKG_VERSION")))
        .send().await.unwrap();
    
    let body = api.text().await.unwrap();
    let json = serde_json::from_str::<CratesApiResponse>(&body).unwrap();
    json.value.newest_version
}