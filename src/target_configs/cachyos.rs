use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct CachyOSTarget {
    /// Fetch list of mirrors timeout in milliseconds
    #[arg(
        env = "RATE_MIRRORS_FETCH_MIRRORS_TIMEOUT",
        long,
        default_value = "15000"
    )]
    pub fetch_mirrors_timeout: u64,

    /// Either url or path to the primary CachyOS mirrors API source
    ///   (JSON format with per-mirror sync-delay metadata).
    #[arg(
        env = "RATE_MIRRORS_MIRRORS_API_SOURCE",
        long,
        default_value = "https://cachyos.org/archlinuxmirrorlist/api/cachyos-mirrors",
        verbatim_doc_comment
    )]
    pub mirrors_api_source: String,

    /// Either url or path to a fallback CachyOS mirrors API source, tried
    ///   when the primary API source fails to fetch.
    #[arg(
        env = "RATE_MIRRORS_MIRRORS_API_SOURCE_FALLBACK",
        long,
        default_value = "https://packages.cachyos.org/api/v1/mirrors",
        verbatim_doc_comment
    )]
    pub mirrors_api_source_fallback: String,

    /// Either url or path to the primary CachyOS plain pacman mirrorlist
    ///   source. Used when all mirrors API sources fail or return no
    ///   mirrors.
    #[arg(
        env = "RATE_MIRRORS_MIRRORLIST_SOURCE",
        long,
        default_value = "https://cachyos.org/archlinuxmirrorlist/api/cachyos-mirrorlist",
        verbatim_doc_comment
    )]
    pub mirrorlist_source: String,

    /// Either url or path to a fallback CachyOS plain pacman mirrorlist
    ///   source, tried when the primary mirrorlist source fails to fetch.
    #[arg(
        env = "RATE_MIRRORS_MIRRORLIST_SOURCE_FALLBACK",
        long,
        default_value = "https://raw.githubusercontent.com/CachyOS/CachyOS-PKGBUILDS/master/cachyos-mirrorlist/cachyos-mirrorlist",
        verbatim_doc_comment
    )]
    pub mirrorlist_source_fallback: String,

    /// Max acceptable delay in seconds since the mirror was last synced
    ///   (only applied to the JSON API source, which reports per-mirror)
    #[arg(
        env = "RATE_MIRRORS_MAX_DELAY",
        long,
        default_value = "86400",
        verbatim_doc_comment
    )]
    pub max_delay: i64,

    /// Path to be joined to a mirror url and used for speed testing
    ///   the file should be big enough to allow for testing high
    ///   speed connections
    #[arg(
        env = "RATE_MIRRORS_PATH_TO_TEST",
        long,
        default_value = "x86_64/cachyos/cachyos.files",
        verbatim_doc_comment
    )]
    pub path_to_test: String,

    /// Architecture
    #[arg(env = "RATE_MIRRORS_ARCH", long, default_value = "auto")]
    pub arch: String,

    /// comment prefix to use when outputting
    #[arg(env = "RATE_MIRRORS_COMMENT_PREFIX", long, default_value = "# ")]
    pub comment_prefix: String,
}
