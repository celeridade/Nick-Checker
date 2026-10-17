//! L96A1 checkers — registro central de plataformas + resolução de query `-p`.
//!
//! - [`platforms::native`] — 29 checkers hand-written + ~1176 gerados de
//!   manifest/WMN (`tools/gen_sites.py`), organizados por categoria
//!   (`social`, `dev`, `gaming`, … — ver [`GROUPS`]);
//! - [`platforms::site_probe`] — probe compartilhado dos gerados.

pub mod common;
pub mod platforms;

use crate::platforms::native;

// API pública histórica: `l96a1_checkers::<plataforma>` (as 29 nativas).
pub use platforms::native::*;
use l96a1_core::Platform;
use std::sync::Arc;

pub fn all() -> Vec<Arc<dyn Platform>> {
    let mut v: Vec<Arc<dyn Platform>> = vec![
        // social
        Arc::new(native::tiktok::Tiktok),
        Arc::new(native::instagram::Instagram),
        Arc::new(native::bluesky::Bluesky),
        Arc::new(native::x::X),
        Arc::new(native::telegram::Telegram),
        Arc::new(native::snapchat::Snapchat),
        Arc::new(native::letterboxd::Letterboxd),
        Arc::new(native::pinterest::Pinterest),
        Arc::new(native::tumblr::Tumblr),
        Arc::new(native::threads::Threads),
        Arc::new(native::medium::Medium),
        Arc::new(native::guns_lol::GunsLol),
        // dev
        Arc::new(native::github::Github),
        Arc::new(native::gitlab::Gitlab),
        Arc::new(native::reddit::Reddit),
        // gaming
        Arc::new(native::discord::Discord),
        Arc::new(native::steam::Steam),
        Arc::new(native::lichess::Lichess),
        Arc::new(native::chess_com::ChessCom),
        Arc::new(native::roblox::Roblox),
        Arc::new(native::minecraft::Minecraft),
        // streaming
        Arc::new(native::youtube::Youtube),
        Arc::new(native::spotify::Spotify),
        Arc::new(native::twitch::Twitch),
        Arc::new(native::kick::Kick),
        // music
        Arc::new(native::soundcloud::Soundcloud),
        Arc::new(native::lastfm::Lastfm),
        Arc::new(native::myspace::Myspace),
        // community
        Arc::new(native::imgur::Imgur),
    ];
    v.extend(crate::platforms::generated::all_generated());
    v
}

/// Checkers gerados de manifest/WMN (tools/gen_sites.py, ~1176).
/// Mesmas keys `mf:`/`wm:`, validators e evidências dos motores antigos.
pub fn all_generated() -> Vec<Arc<dyn Platform>> {
    crate::platforms::generated::all_generated()
}

/// Grupos endereçáveis: 6 legados + 17 cats dos gerados.
/// (`manifest`/`wmn` saíram: viraram structs com o cat de cada site.)
pub const GROUPS: [&str; 23] = [
    "social",
    "dev",
    "gaming",
    "streaming",
    "music",
    "community",
    "archived",
    "art",
    "blog",
    "business",
    "coding",
    "dating",
    "finance",
    "health",
    "hobby",
    "images",
    "misc",
    "news",
    "political",
    "shopping",
    "tech",
    "video",
    "xx NSFW xx",
];

/// Resolve `all`, `social,dev`, `github,tiktok`, `all,^x` para a lista.
/// `extra` = checkers adicionais do chamador (somados aos nativos+gerados).
pub fn resolve(query: &str) -> Vec<Arc<dyn Platform>> {
    resolve_with(query, &[])
}

pub fn resolve_with(query: &str, extra: &[Arc<dyn Platform>]) -> Vec<Arc<dyn Platform>> {
    let mut all = all();
    all.extend(extra.iter().cloned());
    let q = query.trim().to_lowercase();
    if q.is_empty() || q == "all" {
        return all;
    }
    let mut include_groups: Vec<String> = Vec::new();
    let mut include_keys: Vec<String> = Vec::new();
    let mut exclude: Vec<String> = Vec::new();
    for part in q.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(k) = part.strip_prefix('^') {
            exclude.push(k.to_string());
        } else if GROUPS.contains(&part) {
            include_groups.push(part.to_string());
        } else {
            include_keys.push(part.to_string());
        }
    }
    let use_all = include_groups.is_empty() && include_keys.is_empty();
    all.into_iter()
        .filter(|p| {
            if exclude.iter().any(|e| e == p.key()) {
                return false;
            }
            use_all
                || include_groups.iter().any(|g| g == p.group())
                || include_keys.iter().any(|k| k == p.key())
        })
        .collect()
}
