use crate::config::{AppError, FetchMirrors, LogFormatter};
use crate::mirror::Mirror;
use crate::target_configs::cachyos::CachyOSTarget;
use reqwest;
use std::fmt::Display;
use std::sync::mpsc;
use std::time::Duration;
use tokio::runtime::Runtime;
use url::Url;

impl LogFormatter for CachyOSTarget {
    fn format_comment(&self, message: impl Display) -> String {
        format!("{}{}", self.comment_prefix, message)
    }

    fn format_mirror(&self, mirror: &Mirror) -> String {
        let arch = if self.arch == "auto" {
            "$arch"
        } else {
            &self.arch
        };

        format!("Server = {}{}/$repo", mirror.url, arch)
    }
}

impl FetchMirrors for CachyOSTarget {
    fn fetch_mirrors(&self, _tx_progress: mpsc::Sender<String>) -> Result<Vec<Mirror>, AppError> {
        let output = Runtime::new()
            .unwrap()
            .block_on(async { fetch_mirrors_data(self.fetch_mirrors_timeout).await })?;

        let urls = output
            .lines()
            .filter(|line| !line.starts_with('#'))
            .map(|line| line.replace("Server = ", "").replace("$arch/$repo", ""))
            .filter(|line| !line.is_empty())
            .filter_map(|line| Url::parse(&line).ok());

        let result: Vec<_> = urls
            .map(|url| {
                let url_to_test = url
                    .join(&self.path_to_test)
                    .expect("failed to join path_to_test");
                Mirror {
                    country: None,
                    url,
                    url_to_test,
                }
            })
            .collect();

        Ok(result)
    }
}

async fn fetch_mirrors_from_url(
    client: &reqwest::Client,
    url: &str,
    timeout: Duration,
) -> Result<String, reqwest::Error> {
    client
        .get(url)
        .timeout(timeout)
        .send()
        .await?
        .text_with_charset("utf-8")
        .await
}

async fn fetch_mirrors_data(fetch_mirrors_timeout: u64) -> Result<String, AppError> {
    let (primary_url, fallback_url) = {
        (
            "https://cachyos.org/archlinuxmirrorlist/api/cachyos-mirrorlist",
            "https://raw.githubusercontent.com/CachyOS/CachyOS-PKGBUILDS/master/cachyos-mirrorlist/cachyos-mirrorlist",
        )
    };

    let client = reqwest::Client::new();
    let timeout = Duration::from_millis(fetch_mirrors_timeout);

    // try to use cachyos proxy first, and then fallback to github one
    let primary_res = fetch_mirrors_from_url(&client, primary_url, timeout).await;
    if let Err(_err) = primary_res {
        println!("# Falling back mirrorlist url");
        Ok(fetch_mirrors_from_url(&client, fallback_url, timeout).await?)
    } else {
        // result is already checked
        Ok(primary_res.unwrap())
    }
}
