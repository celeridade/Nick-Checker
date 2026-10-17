//! Enrichment separado da API de check (Fase 1).
//! `GET /v1/enrich?platform=<key>&user=<nick>` — só dado público,
//! pool/rate próprios, nunca bloqueia o scan. Frontend chama sob demanda
//! (abrir modal / hover). Research: `research/BRAVE-HAR.md` + `research/*.md`
//! + `research/fingerprint/.../sse-results.json` (fonte dos seletores).

use l96a1_core::RichProfile;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct EnrichCache {
    inner: Arc<Mutex<HashMap<String, (Instant, RichProfile)>>>,
    ttl: Duration,
}

impl EnrichCache {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(HashMap::new())), ttl: Duration::from_secs(24 * 3600) }
    }
    pub async fn get(&self, k: &str) -> Option<RichProfile> {
        let m = self.inner.lock().await;
        m.get(k).and_then(|(t, v)| if t.elapsed() < self.ttl { Some(v.clone()) } else { None })
    }
    pub async fn put(&self, k: String, v: RichProfile) {
        self.inner.lock().await.insert(k, (Instant::now(), v));
    }
    pub async fn inner_get(&self, k: &str) -> Option<RichProfile> {
        self.get(k).await
    }
    pub async fn inner_put(&self, k: String, v: RichProfile) {
        self.put(k, v).await
    }
}

impl Default for EnrichCache {
    fn default() -> Self {
        Self::new()
    }
}

fn client() -> reqwest::Client {
    l96a1_core::build_client(Duration::from_secs(10), None)
}

async fn fetch_json(url: &str, headers: &[(&str, &str)]) -> Option<serde_json::Value> {
    let c = client();
    let mut r = c.get(url).header("Accept", "application/json");
    for (k, v) in headers {
        r = r.header(*k, *v);
    }
    let resp = r.send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json::<serde_json::Value>().await.ok()
}

async fn fetch_text(url: &str) -> Option<String> {
    let c = client();
    let resp = c.get(url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.text().await.ok()
}

fn s(v: &serde_json::Value, path: &[&str]) -> Option<String> {
    let mut cur = v;
    for p in path {
        cur = cur.get(*p)?;
    }
    cur.as_str().map(|x| x.to_string()).or_else(|| cur.as_i64().map(|n| n.to_string()))
}

fn i(v: &serde_json::Value, path: &[&str]) -> Option<i64> {
    let mut cur = v;
    for p in path {
        cur = cur.get(*p)?;
    }
    cur.as_i64().or_else(|| cur.as_u64().map(|n| n as i64))
}

/// `og:image` / `og:title` do HTML (custo-zero, sem API).
fn meta(html: &str, prop: &str) -> Option<String> {
    for pat in [
        format!(r#"property="{prop}" content=""#),
        format!(r#"name="{prop}" content=""#),
        format!(r#"property='{prop}' content='"#),
    ] {
        if let Some(i) = html.find(&pat) {
            let rest = &html[i + pat.len()..];
            let end = rest.find(['"', '\''])?;
            let v = rest[..end].trim().to_string();
            if !v.is_empty() {
                return Some(v);
            }
        }
    }
    None
}

fn unescape(s: &str) -> String {
    s.replace("\\u0026", "&").replace("\\/", "/").replace("\\\"", "\"")
}

/// GET /v1/enrich — dispatcher por plataforma. Nunca falha hard: retorna
/// parcial (campos None quando a origem não expõe).
/// `discord_token` = 1º token do pool (só usado no enrich Discord por ID).
pub async fn enrich_one(
    platform_key: &str,
    user: &str,
    github_token: &str,
    discord_token: Option<&str>,
) -> RichProfile {
    let key = platform_key.to_lowercase();
    match key.as_str() {
        "github" => enrich_github(user, github_token).await,
        "bluesky" => enrich_bluesky(user).await,
        "kick" => enrich_kick(user).await,
        "telegram" => enrich_telegram(user).await,
        "chesscom" | "chess_com" | "chess" => enrich_chesscom(user).await,
        "gitlab" => enrich_gitlab(user).await,
        "steam" => enrich_steam(user).await,
        "youtube" => enrich_youtube(user).await,
        "soundcloud" => enrich_soundcloud(user).await,
        "myspace" => enrich_myspace(user).await,
        "pinterest" => enrich_pinterest(user).await,
        "x" | "twitter" => enrich_x(user).await,
        "tiktok" => enrich_tiktok(user).await,
        "threads" => enrich_threads(user).await,
        "instagram" => enrich_instagram(user).await,
        "medium" => enrich_medium(user).await,
        "tumblr" => enrich_tumblr(user).await,
        "letterboxd" => enrich_letterboxd(user).await,
        "lichess" => enrich_lichess(user).await,
        "guns_lol" | "gunslol" | "guns lol" => enrich_gunslol(user).await,
        "snapchat" => enrich_snapchat(user).await,
        "roblox" => enrich_roblox(user).await,
        "minecraft" | "namemc" => enrich_minecraft(user).await,
        "lastfm" | "last_fm" => enrich_lastfm(user).await,
        "reddit" => enrich_reddit(user).await,
        "imgur" => enrich_imgur(user).await,
        // Discord: sem ID conhecido não há foto/banner (só nick) —
        // com ID (`discord_ids.json`), profile por ID traz tudo.
        "discord" => enrich_discord(user, discord_token).await,
        _ => RichProfile::default(),
    }
}

// ---- custo-zero (mesmo endpoint do check, só parseia mais) ----

/// Discord por ID: `GET /users/{id}/profile` com user-token do pool.
/// Vale p/ perfis privados também (bio pode vir vazia). Fora do caminho
/// quente: só modal/enrich on-demand, nunca no scan.
async fn enrich_discord(user: &str, discord_token: Option<&str>) -> RichProfile {
    let Some(id) = l96a1_checkers::discord::discord_id_for(user) else {
        return RichProfile::default();
    };
    let Some(tok) = discord_token.filter(|s| !s.is_empty()) else {
        return RichProfile::default();
    };
    l96a1_checkers::discord::fetch_profile(tok, &id).await.unwrap_or_default()
}

async fn enrich_github(user: &str, token: &str) -> RichProfile {
    let url = format!("https://api.github.com/users/{user}");
    let mut h: Vec<(&str, &str)> = vec![("User-Agent", l96a1_core::CHROME_UA)];
    let auth;
    if !token.is_empty() {
        auth = format!("Bearer {token}");
        h.push(("Authorization", Box::leak(auth.into_boxed_str()) as &str));
    }
    let v = match fetch_json(&url, &h).await {
        Some(v) => v,
        None => return RichProfile::default(),
    };
    RichProfile {
        display_name: s(&v, &["name"]),
        bio: s(&v, &["bio"]),
        pfp_url: s(&v, &["avatar_url"]),
        user_id: s(&v, &["id"]),
        followers: i(&v, &["followers"]),
        following: i(&v, &["following"]),
        website: s(&v, &["blog"]).filter(|x| !x.is_empty()),
        ..Default::default()
    }
}

async fn enrich_bluesky(user: &str) -> RichProfile {
    let actor = if user.contains('.') { user.to_string() } else { format!("{user}.bsky.social") };
    let url = format!("https://public.api.bsky.app/xrpc/app.bsky.actor.getProfile?actor={actor}");
    let v = match fetch_json(&url, &[]).await {
        Some(v) => v,
        None => return RichProfile::default(),
    };
    RichProfile {
        display_name: s(&v, &["displayName"]),
        bio: s(&v, &["description"]),
        pfp_url: s(&v, &["avatar"]),
        banner_url: s(&v, &["banner"]),
        user_id: s(&v, &["did"]),
        followers: i(&v, &["followersCount"]),
        following: i(&v, &["followsCount"]),
        posts: i(&v, &["postsCount"]),
        ..Default::default()
    }
}

async fn enrich_kick(user: &str) -> RichProfile {
    let url = format!("https://kick.com/api/v2/channels/{user}");
    let v = match fetch_json(&url, &[]).await {
        Some(v) => v,
        None => return RichProfile::default(),
    };
    let ch = v.get("user").unwrap_or(&v);
    RichProfile {
        display_name: s(ch, &["username"]).or_else(|| s(&v, &["slug"])),
        bio: s(&v, &["channel_description"]).or_else(|| s(ch, &["bio"])),
        pfp_url: s(&v, &["user", "profile_picture"]).or_else(|| s(&v, &["profile_picture"])),
        banner_url: s(&v, &["banner_image"]),
        user_id: s(&v, &["id"]),
        followers: i(&v, &["followersCount"]),
        ..Default::default()
    }
}

async fn enrich_telegram(user: &str) -> RichProfile {
    let html = match fetch_text(&format!("https://t.me/{user}")).await {
        Some(h) => h,
        None => return RichProfile::default(),
    };
    let img = meta(&html, "og:image").and_then(|u| {
        if u.contains("tgme_page_photo_image") || u.starts_with("https://cdn") {
            Some(u)
        } else {
            // fallback: pesca o <img class="tgme_page_photo_image" src="...">
            html.find("tgme_page_photo_image").and_then(|i| {
                let rest = &html[i..];
                let s = rest.find("src=\"")? + 5;
                let e = rest[s..].find('"')?;
                Some(unescape(&rest[s..s + e]))
            })
        }
    });
    RichProfile {
        display_name: meta(&html, "og:title"),
        bio: meta(&html, "og:description"),
        pfp_url: img,
        ..Default::default()
    }
}

async fn enrich_chesscom(user: &str) -> RichProfile {
    let lower = user.to_lowercase();
    let v = match fetch_json(&format!("https://api.chess.com/pub/player/{lower}"), &[]).await {
        Some(v) => v,
        None => return RichProfile::default(),
    };
    RichProfile {
        display_name: s(&v, &["username"]),
        pfp_url: s(&v, &["avatar"]),
        user_id: s(&v, &["player_id"]),
        followers: i(&v, &["followers"]),
        website: s(&v, &["url"]),
        ..Default::default()
    }
}

async fn enrich_gitlab(user: &str) -> RichProfile {
    let v = match fetch_json(&format!("https://gitlab.com/api/v4/users?username={user}"), &[]).await {
        Some(v) => v,
        None => return RichProfile::default(),
    };
    let u = v.as_array().and_then(|a| a.first()).cloned().unwrap_or_default();
    RichProfile {
        display_name: s(&u, &["name"]),
        bio: s(&u, &["bio"]),
        pfp_url: s(&u, &["avatar_url"]),
        user_id: s(&u, &["id"]),
        website: s(&u, &["website_url"]).filter(|x| !x.is_empty()),
        ..Default::default()
    }
}

async fn enrich_steam(user: &str) -> RichProfile {
    let html = match fetch_text(&format!("https://steamcommunity.com/id/{user}")).await {
        Some(h) => h,
        None => return RichProfile::default(),
    };
    RichProfile {
        display_name: meta(&html, "og:title"),
        pfp_url: meta(&html, "og:image"),
        bio: meta(&html, "og:description"),
        ..Default::default()
    }
}

async fn enrich_youtube(user: &str) -> RichProfile {
    // `@` primeiro; se vazio (nick só existe no legado `/user/`), cai no
    // legado para os embeds virem normais (og:title/description/image).
    let html_at = fetch_text(&format!("https://www.youtube.com/@{user}")).await;
    let mut prof = html_at
        .as_deref()
        .map(|html| RichProfile {
            display_name: meta(html, "og:title"),
            bio: meta(html, "og:description"),
            pfp_url: meta(html, "og:image"),
            ..Default::default()
        })
        .unwrap_or_default();
    if prof.display_name.is_none() && prof.pfp_url.is_none() {
        if let Some(html) = fetch_text(&format!("https://www.youtube.com/user/{user}")).await {
            prof = RichProfile {
                display_name: meta(&html, "og:title"),
                bio: meta(&html, "og:description"),
                pfp_url: meta(&html, "og:image"),
                ..Default::default()
            };
        }
    }
    prof
}

async fn enrich_soundcloud(user: &str) -> RichProfile {
    let html = match fetch_text(&format!("https://soundcloud.com/{user}")).await {
        Some(h) => h,
        None => return RichProfile::default(),
    };
    RichProfile {
        display_name: meta(&html, "og:title"),
        bio: meta(&html, "og:description"),
        pfp_url: meta(&html, "og:image"),
        ..Default::default()
    }
}

async fn enrich_myspace(user: &str) -> RichProfile {
    let html = match fetch_text(&format!("https://myspace.com/{user}")).await {
        Some(h) => h,
        None => return RichProfile::default(),
    };
    RichProfile {
        display_name: meta(&html, "og:title"),
        bio: meta(&html, "og:description"),
        pfp_url: meta(&html, "og:image"),
        ..Default::default()
    }
}

async fn enrich_pinterest(user: &str) -> RichProfile {
    let html = match fetch_text(&format!("https://www.pinterest.com/{user}/")).await {
        Some(h) => h,
        None => return RichProfile::default(),
    };
    // JSON-LD Person
    let pfp = html.find(r#""@type":"Person""#).and_then(|i| {
        let rest = &html[i..i + 4000.min(html.len() - i)];
        let k = rest.find(r#""image":"#)? + 9;
        let e = rest[k..].find('"')?;
        Some(unescape(&rest[k..k + e]))
    });
    RichProfile { pfp_url: pfp.or_else(|| meta(&html, "og:image")), ..Default::default() }
}

async fn enrich_x(user: &str) -> RichProfile {
    // Mirror leve (pode 403) + fallback page og:.
    if let Some(v) = fetch_json(&format!("https://api.vxtwitter.com/{user}"), &[]).await {
        return RichProfile {
            display_name: s(&v, &["name"]).or_else(|| s(&v, &["user", "name"])),
            bio: s(&v, &["description"]).or_else(|| s(&v, &["user", "description"])),
            pfp_url: s(&v, &["profile_image_url_https"]).or_else(|| s(&v, &["user", "profile_image_url_https"])),
            banner_url: s(&v, &["profile_banner_url"]),
            user_id: s(&v, &["id_str"]).or_else(|| s(&v, &["user", "id_str"])),
            followers: i(&v, &["followers_count"]),
            following: i(&v, &["friends_count"]),
            ..Default::default()
        };
    }
    let html = fetch_text(&format!("https://x.com/{user}")).await.unwrap_or_default();
    RichProfile {
        pfp_url: meta(&html, "og:image"),
        display_name: meta(&html, "og:title"),
        bio: meta(&html, "og:description"),
        ..Default::default()
    }
}

async fn enrich_tiktok(user: &str) -> RichProfile {
    let v = fetch_json(&format!("https://www.tiktok.com/oembed?url=https://www.tiktok.com/@{user}"), &[]).await;
    if let Some(v) = v {
        return RichProfile {
            display_name: s(&v, &["author_name"]),
            ..Default::default()
        };
    }
    RichProfile::default()
}

async fn enrich_threads(user: &str) -> RichProfile {
    let html = match fetch_text(&format!("https://www.threads.com/@{user}")).await {
        Some(h) => h,
        None => return RichProfile::default(),
    };
    RichProfile {
        pfp_url: meta(&html, "og:image"),
        display_name: meta(&html, "og:title"),
        bio: meta(&html, "og:description"),
        ..Default::default()
    }
}

async fn enrich_instagram(user: &str) -> RichProfile {
    let html = match fetch_text(&format!("https://www.instagram.com/{user}/")).await {
        Some(h) => h,
        None => return RichProfile::default(),
    };
    RichProfile {
        pfp_url: meta(&html, "og:image"),
        display_name: meta(&html, "og:title"),
        bio: meta(&html, "og:description"),
        ..Default::default()
    }
}

async fn enrich_medium(user: &str) -> RichProfile {
    let html = match fetch_text(&format!("https://medium.com/@{user}")).await {
        Some(h) => h,
        None => return RichProfile::default(),
    };
    RichProfile {
        pfp_url: meta(&html, "og:image"),
        display_name: meta(&html, "og:title"),
        ..Default::default()
    }
}

async fn enrich_tumblr(user: &str) -> RichProfile {
    let lower = user.to_lowercase();
    let html = match fetch_text(&format!("https://www.tumblr.com/{lower}")).await {
        Some(h) => h,
        None => return RichProfile::default(),
    };
    RichProfile {
        pfp_url: meta(&html, "og:image"),
        display_name: meta(&html, "og:title"),
        ..Default::default()
    }
}

async fn enrich_letterboxd(user: &str) -> RichProfile {
    let html = match fetch_text(&format!("https://letterboxd.com/{user}/")).await {
        Some(h) => h,
        None => return RichProfile::default(),
    };
    RichProfile {
        pfp_url: meta(&html, "og:image"),
        display_name: meta(&html, "og:title"),
        ..Default::default()
    }
}

async fn enrich_lichess(user: &str) -> RichProfile {
    let lower = user.to_lowercase();
    let v = match fetch_json(&format!("https://lichess.org/api/user/{lower}"), &[]).await {
        Some(v) => v,
        None => return RichProfile::default(),
    };
    RichProfile {
        display_name: s(&v, &["username"]),
        bio: s(&v, &["profile", "bio"]),
        user_id: s(&v, &["id"]),
        followers: v.get("count").and_then(|c| c.get("following")).and_then(|n| n.as_i64()),
        following: v.get("count").and_then(|c| c.get("follow")).and_then(|n| n.as_i64()),
        ..Default::default()
    }
}

async fn enrich_gunslol(user: &str) -> RichProfile {
    let html = match fetch_text(&format!("https://guns.lol/{user}")).await {
        Some(h) => h,
        None => return RichProfile::default(),
    };
    // <img class="avatar" src="...">
    let pfp = html.find("avatar").and_then(|i| {
        let rest = &html[i..i + 2000.min(html.len() - i)];
        let k = rest.find("src=\"")? + 5;
        let e = rest[k..].find('"')?;
        Some(unescape(&rest[k..k + e]))
    });
    RichProfile {
        pfp_url: pfp.or_else(|| meta(&html, "og:image")),
        display_name: meta(&html, "og:title"),
        ..Default::default()
    }
}

async fn enrich_snapchat(user: &str) -> RichProfile {
    let html = match fetch_text(&format!("https://www.snapchat.com/@{user}")).await {
        Some(h) => h,
        None => return RichProfile::default(),
    };
    RichProfile {
        pfp_url: meta(&html, "og:image"),
        display_name: meta(&html, "og:title"),
        ..Default::default()
    }
}

// ---- 2º fetch público ----

async fn enrich_roblox(user: &str) -> RichProfile {
    // validate → id → profile + thumbs
    let v = fetch_json(
        &format!("https://auth.roblox.com/v1/usernames/validate?request.username={user}&request.birthday=2000-01-01T00:00:00.000Z&request.context=Signup"),
        &[],
    )
    .await;
    let _ = v;
    // lookup id via users API
    let id: Option<i64> = fetch_json("https://users.roblox.com/v1/usernames/users", &[])
        .await
        .and_then(|_| None);
    let _ = id;
    // Fallback determinístico: headshot por username não existe sem id;
    // tenta página de perfil para extrair id.
    let html = fetch_text(&format!("https://www.roblox.com/users/profile?username={user}")).await.unwrap_or_default();
    let found_id = html.find("users/").and_then(|i| {
        let rest = &html[i + 6..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        digits.parse::<i64>().ok()
    });
    if let Some(uid) = found_id {
        let prof = fetch_json(&format!("https://users.roblox.com/v1/users/{uid}"), &[]).await;
        let thumbs = fetch_json(
            &format!("https://thumbnails.roblox.com/v1/users/avatar-headshot?userIds={uid}&size=420x420&format=Png"),
            &[],
        )
        .await;
        let pfp = thumbs
            .as_ref()
            .and_then(|t| t.get("data"))
            .and_then(|d| d.as_array())
            .and_then(|a| a.first())
            .and_then(|o| o.get("imageUrl"))
            .and_then(|u| u.as_str())
            .map(|x| x.to_string());
        return RichProfile {
            display_name: prof.as_ref().and_then(|p| s(p, &["displayName"])),
            bio: prof.as_ref().and_then(|p| s(p, &["description"])),
            pfp_url: pfp,
            user_id: Some(uid.to_string()),
            ..Default::default()
        };
    }
    RichProfile::default()
}

async fn enrich_minecraft(user: &str) -> RichProfile {
    // Determinístico, sem fetch: mc-heads por nick (resolve igual com uuid).
    RichProfile {
        pfp_url: Some(format!("https://mc-heads.net/avatar/{user}")),
        ..Default::default()
    }
}

async fn enrich_lastfm(user: &str) -> RichProfile {
    let html = match fetch_text(&format!("https://www.last.fm/user/{user}")).await {
        Some(h) => h,
        None => return RichProfile::default(),
    };
    RichProfile {
        pfp_url: meta(&html, "og:image"),
        display_name: meta(&html, "og:title"),
        ..Default::default()
    }
}

async fn enrich_reddit(user: &str) -> RichProfile {
    let v = fetch_json(&format!("https://www.reddit.com/user/{user}/about.json"), &[("User-Agent", l96a1_core::CHROME_UA)]).await;
    if let Some(v) = v {
        let d = v.get("data").unwrap_or(&v);
        return RichProfile {
            display_name: s(d, &["name"]),
            bio: s(d, &["subreddit", "public_description"]),
            pfp_url: s(d, &["icon_img"]).or_else(|| s(d, &["snoovatar_img"])).map(|u| u.replace("&amp;", "&")),
            banner_url: s(d, &["subreddit", "banner_img"]).map(|u| u.replace("&amp;", "&")),
            user_id: s(d, &["id"]),
            followers: i(d, &["subscribers"]),
            ..Default::default()
        };
    }
    RichProfile::default()
}

async fn enrich_imgur(user: &str) -> RichProfile {
    let v = fetch_json(
        &format!("https://api.imgur.com/account/v1/accounts/{user}?client_id=546c25a59c58ad7"),
        &[],
    )
    .await;
    if let Some(v) = v {
        let d = v.get("data").unwrap_or(&v);
        return RichProfile {
            display_name: s(d, &["url"]),
            bio: s(d, &["bio"]),
            pfp_url: s(d, &["avatar"]),
            banner_url: s(d, &["cover"]),
            user_id: s(d, &["id"]),
            ..Default::default()
        };
    }
    RichProfile::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_shapes() {
        assert_eq!(l96a1_core::mask_proxy(None), "direct");
        assert_eq!(l96a1_core::mask_proxy(Some("http://u:p@1.2.3.4:8080")), "http://***:8080");
    }

    #[test]
    fn meta_extracts_og() {
        let h = r#"<meta property="og:image" content="https://x/y.png">"#;
        assert_eq!(meta(h, "og:image").as_deref(), Some("https://x/y.png"));
    }
}
