# fingerprint.to — mapa extraido do HAR (offline, sem rate-limit)

- entries: 551 | extraidos: 69 | dedup url: 321 | dedup conteudo: 0 | ruido pulado: 15
- SSE /api/search: 49 results (Xbox, AniList, DockerHub, Disqus, Sourcehut, Fragment, Matrix, Division2, Kofi, DeviantArt, Letterboxd, GitLab, Medal, R6, Lichess...)

## APIs (conteudo em `_har-meta/apis/`)
- `account/sessions` <- https://fingerprint.to/api/account/sessions
- `auth/me` <- https://fingerprint.to/api/auth/me
- `auth/turnstile` <- https://fingerprint.to/api/auth/turnstile
- `data-search` <- https://fingerprint.to/api/data-search
- `history` <- https://fingerprint.to/api/history?page=1
- `rate-limit` <- https://fingerprint.to/api/rate-limit
- `search` <- https://fingerprint.to/api/search?query=adoecido&type=username&turnstile=1.6r-mHV14XirP5gPyWWfKAaiAc

## Arquitetura inferida do SSE
`GET /api/search?query={u}&type=username&turnstile=...` -> `text/event-stream` com eventos `connected -> ratelimit -> start{total} -> loop(result+checked) -> done`. Cada `result` tem `{platform, platformUrl, username, displayName?, pfpUrl?, userId?, followers/following/posts?, createdAt?, lastActive?, profileUrl, extras{...}, locked}`.

## Grupos L96A1 sugeridos (mapeamento UI -> backend Rust)
- social: Twitter/X, Reddit, Pinterest, Telegram, Myspace, Wordpress, Blogspot, Disqus, Habbo...
- dev: GitHub, GitLab, Codeberg, Sourcehut, HuggingFace, DockerHub...
- gaming: Xbox, Roblox, Minecraft/NameMC, R6/Division2-tracker.gg, Fortnite-tracker, Lichess...
- streaming/music: YouTube, Twitch, Spotify, SoundCloud, LastFM, Unsplash, Pixabay...
- community/misc: Pastebin, Poe, Trakt, Letterboxd, AniList, Kofi, DeviantArt, Medal, Ebay, TikTok, Xvideos, GunsLol, Matrix, Fragment, RuneScape, XDAForums, Genius...
