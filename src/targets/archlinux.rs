use crate::config::{AppError, FetchMirrors, LogFormatter, fetch_json_or_file};
use crate::countries::Country;
use crate::mirror::Mirror;
use crate::target_configs::archlinux::{ArchMirrorsSortingStrategy, ArchTarget};
use crate::target_configs::fetch::FetchArgs;
use rand::prelude::SliceRandom;
use rand::rng;
use serde::Deserialize;
use std::fmt::Display;
use std::sync::mpsc;
use url::Url;

pub(crate) const ARCH_TIER_1_MIRROR_SOURCE: &str =
    "https://archlinux.org/mirrors/status/tier/1/json/";
const ARCH_CACHYOS_PROXY_TIER_1_SOURCE: &str = "https://cachyos.org/archlinuxmirrorlist/api/tier1";
const ARCH_CACHYOS_PROXY_STATUS_SOURCE: &str = "https://cachyos.org/archlinuxmirrorlist/api/status";
const ARCH_STATUS_SOURCE: &str = "https://archlinux.org/mirrors/status/json/";

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

pub(crate) fn fetch_archweb_mirrors(
    source: &str,
    fetch: &FetchArgs,
    completion: f64,
    max_delay: i64,
    sort: &ArchMirrorsSortingStrategy,
    path_to_test: &str,
    tx_progress: mpsc::Sender<String>,
) -> Result<Vec<Mirror>, AppError> {
    let mirrors_data: ArchMirrorsData = fetch_json_or_file(source, fetch, &tx_progress)?;
    process_archweb_mirrors(
        mirrors_data,
        completion,
        max_delay,
        sort,
        path_to_test,
        tx_progress,
    )
}

/// Filter, sort and convert already fetched archweb mirror data.
fn process_archweb_mirrors(
    mirrors_data: ArchMirrorsData,
    completion: f64,
    max_delay: i64,
    sort: &ArchMirrorsSortingStrategy,
    path_to_test: &str,
    tx_progress: mpsc::Sender<String>,
) -> Result<Vec<Mirror>, AppError> {
    tx_progress
        .send(format!("FETCHED MIRRORS: {}", mirrors_data.urls.len()))
        .unwrap();

    let mut mirrors: Vec<_> = mirrors_data
        .urls
        .into_iter()
        .filter(|mirror| {
            if let Some(completion_pct) = mirror.completion_pct {
                if let Some(delay) = mirror.delay {
                    return completion_pct >= completion && delay <= max_delay;
                }
            }
            false
        })
        .collect();

    match sort {
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
                if let Ok(url_to_test) = url.join(path_to_test) {
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

/// Resolve the (primary, fallback) mirror sources for the requested tier.
fn proxy_and_fallback_urls(fetch_first_tier_only: bool) -> (&'static str, &'static str) {
    if fetch_first_tier_only {
        (ARCH_CACHYOS_PROXY_TIER_1_SOURCE, ARCH_TIER_1_MIRROR_SOURCE)
    } else {
        (ARCH_CACHYOS_PROXY_STATUS_SOURCE, ARCH_STATUS_SOURCE)
    }
}

/// Fetch the archlinux mirror status.
fn fetch_mirrors_data(
    fetch_first_tier_only: bool,
    fetch: &FetchArgs,
    tx_progress: &mpsc::Sender<String>,
) -> Result<ArchMirrorsData, AppError> {
    let (primary, fallback) = proxy_and_fallback_urls(fetch_first_tier_only);
    match fetch_json_or_file::<ArchMirrorsData>(primary, fetch, tx_progress) {
        Ok(data) => Ok(data),
        Err(_err) => {
            tx_progress
                .send("Falling back mirrorlist url to archlinux".to_string())
                .unwrap();
            fetch_json_or_file::<ArchMirrorsData>(fallback, fetch, tx_progress)
        }
    }
}

impl FetchMirrors for ArchTarget {
    fn fetch_mirrors(&self, tx_progress: mpsc::Sender<String>) -> Result<Vec<Mirror>, AppError> {
        let mirrors_data =
            fetch_mirrors_data(self.fetch_first_tier_only, &self.fetch, &tx_progress)?;

        process_archweb_mirrors(
            mirrors_data,
            self.completion,
            self.max_delay,
            &self.sort_mirrors_by,
            &self.path_to_test,
            tx_progress,
        )
    }
}
