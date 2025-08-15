use crate::config::{AppError, FetchMirrors, LogFormatter};
use crate::countries::Country;
use crate::mirror::Mirror;
use crate::target_configs::archlinux::{ArchMirrorsSortingStrategy, ArchTarget};
use rand::prelude::SliceRandom;
use rand::rng;
use reqwest;
use serde::Deserialize;
use std::fmt::Display;
use std::sync::mpsc;
use std::time::Duration;
use tokio::runtime::Runtime;
use url::Url;

#[derive(Deserialize, Debug, Clone)]
pub struct ArchMirror {
    #[allow(dead_code)]
    protocol: String,
    url: String,
    score: Option<f64>,
    delay: Option<i64>,
    // active: bool,
    country_code: String,
    completion_pct: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct ArchMirrorsData {
    urls: Vec<ArchMirror>,
}

impl LogFormatter for ArchTarget {
    fn format_comment(&self, message: impl Display) -> String {
        format!("{}{}", self.comment_prefix, message)
    }

    fn format_mirror(&self, mirror: &Mirror) -> String {
        format!("Server = {}$repo/os/$arch", &mirror.url)
    }
}

impl FetchMirrors for ArchTarget {
    fn fetch_mirrors(&self, tx_progress: mpsc::Sender<String>) -> Result<Vec<Mirror>, AppError> {
        let mirrors_data = Runtime::new().unwrap().block_on(async {
            fetch_mirrors_data(self.fetch_first_tier_only, self.fetch_mirrors_timeout).await
        })?;

        tx_progress
            .send(format!("FETCHED MIRRORS: {}", mirrors_data.urls.len()))
            .unwrap();

        let mut mirrors: Vec<_> = mirrors_data
            .urls
            .into_iter()
            .filter(|mirror| {
                if let Some(completion_pct) = mirror.completion_pct {
                    if let Some(delay) = mirror.delay {
                        return completion_pct >= self.completion && delay <= self.max_delay;
                    }
                }
                false
            })
            .collect();

        match &self.sort_mirrors_by {
            ArchMirrorsSortingStrategy::Random => {
                let mut _rng = rng();
                mirrors.shuffle(&mut _rng);
            }
            ArchMirrorsSortingStrategy::DelayDesc => {
                mirrors.sort_unstable_by(|a, b| b.delay.partial_cmp(&a.delay).unwrap());
            }
            ArchMirrorsSortingStrategy::DelayAsc => {
                mirrors.sort_unstable_by(|a, b| a.delay.partial_cmp(&b.delay).unwrap());
            }
            ArchMirrorsSortingStrategy::ScoreDesc => {
                mirrors.sort_unstable_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            }
            ArchMirrorsSortingStrategy::ScoreAsc => {
                mirrors.sort_unstable_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
            }
        };

        let result: Vec<_> = mirrors
            .into_iter()
            .filter_map(|m| {
                if let Ok(url) = Url::parse(&m.url) {
                    if let Ok(url_to_test) = url.join(&self.path_to_test) {
                        return Some(Mirror {
                            country: Country::from_str(&m.country_code),
                            url,
                            url_to_test,
                        });
                    }
                };
                None
            })
            .collect();

        Ok(result)
    }
}

async fn fetch_mirrors_from_url(
    client: &reqwest::Client,
    url: &str,
    timeout: Duration,
) -> Result<ArchMirrorsData, reqwest::Error> {
    client
        .get(url)
        .timeout(timeout)
        .send()
        .await?
        .json::<ArchMirrorsData>()
        .await
}

async fn fetch_mirrors_data(
    fetch_first_tier_only: bool,
    fetch_mirrors_timeout: u64,
) -> Result<ArchMirrorsData, AppError> {
    let (primary_url, fallback_url) = if fetch_first_tier_only {
        (
            "https://cachyos.org/archlinuxmirrorlist/api/tier1",
            "https://archlinux.org/mirrors/status/tier/1/json/",
        )
    } else {
        (
            "https://cachyos.org/archlinuxmirrorlist/api/status",
            "https://archlinux.org/mirrors/status/json/",
        )
    };

    let client = reqwest::Client::new();
    let timeout = Duration::from_millis(fetch_mirrors_timeout);

    // try to use cachyos proxy first, and then fallback to archlinux one
    let primary_res = fetch_mirrors_from_url(&client, primary_url, timeout).await;
    if let Err(_err) = primary_res {
        println!("# Falling back mirrorlist url to archlinux");
        Ok(fetch_mirrors_from_url(&client, fallback_url, timeout).await?)
    } else {
        // result is already checked
        Ok(primary_res.unwrap())
    }
}
