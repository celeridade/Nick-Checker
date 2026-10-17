/**
 * L96A1 core data model + inline mock data (dev fallback).
 * Real engine: Rust API via the SSE adapter at /api/l96a1/search; mocks
 * below only render when the engine is offline. Categories mirror the
 * backend GROUPS (29 natives + mf:/wm: generated share them).
 */

export type Category =
  | "social"
  | "dev"
  | "gaming"
  | "streaming"
  | "music"
  | "community"
  | "archived"
  | "art"
  | "blog"
  | "business"
  | "coding"
  | "dating"
  | "finance"
  | "health"
  | "hobby"
  | "images"
  | "misc"
  | "news"
  | "political"
  | "shopping"
  | "tech"
  | "video"
  | "xx NSFW xx";

export const CATEGORY_LABELS: Record<Category, string> = {
  social: "Social",
  dev: "Dev",
  gaming: "Gaming",
  streaming: "Streaming",
  music: "Music",
  community: "Community",
  archived: "Archived",
  art: "Art",
  blog: "Blog",
  business: "Business",
  coding: "Coding",
  dating: "Dating",
  finance: "Finance",
  health: "Health",
  hobby: "Hobby",
  images: "Images",
  misc: "Misc",
  news: "News",
  political: "Political",
  shopping: "Shopping",
  tech: "Tech",
  video: "Video",
  "xx NSFW xx": "NSFW",
};

export interface SearchResult {
  platform: string;
  platformUrl: string;
  username: string;
  displayName: string | null;
  pfpUrl: string | null;
  bannerUrl: string | null;
  userId: string | null;
  bio: string | null;
  followers: number | null;
  following: number | null;
  posts: number | null;
  likes: number | null;
  isVerified: boolean | null;
  emails: string[];
  website: string | null;
  createdAt: string | null;
  lastActive: string | null;
  location: string | null;
  isPrivate: boolean;
  locked: boolean;
  extras: Record<string, string>;
  profileUrl: string;
  sourceQuery: string;
  duration: string;
  category: Category;
  hasChecker: boolean;
  displayOrder: number;
  /** Veio do cache de já-checados (sem rede nesta passada). */
  cached: boolean;
}

export interface WarnInfo {
  nick: string;
  platform: string;
  key: string;
  reason: string;
  evidence: string;
  latency_ms: number;
  proxy: string;
  attempt: number;
}

/* ------------------------------------------------------------------ */
/* Streaming event shapes (must match /api/l96a1/search byte-for-byte)  */
/* ------------------------------------------------------------------ */
export type StreamEvent =
  | { name: "connected"; data: string } // {"status":"connected"}
  | { name: "ratelimit"; data: string } // legado: motor não emite mais (audit 2026-10)
  | { name: "start"; data: string } // {"total":N}
  | { name: "result"; data: string } // SearchResult JSON
  | { name: "checked"; data: string } // {"total":N}
  | { name: "free"; data: string } // FreeNick JSON
  | { name: "warn"; data: string } // WarnInfo JSON {nick,platform,key,reason,evidence,latency_ms,proxy,attempt}
  | { name: "done"; data: string }; // {"type":"done","total":N,"hits":..,"rates":..,"errors":..,"unknowns":..}

const iso = (y: number, m: number, d: number) =>
  `${y}-${String(m).padStart(2, "0")}-${String(d).padStart(2, "0")}T00:00:00Z`;

const extra = (n: number): Record<string, string> => {
  const labels = [
    "Bio", "Avatar URL", "Banner URL", "Email", "Website", "Pronouns",
    "Reputation", "Badges", "Karma", "Level", "XP", "Clan", "Server count",
    "Starred repos", "Pinned items", "Top genres", "Playlists", "Achievements",
    "Guilds",
  ];
  const out: Record<string, string> = {};
  for (let i = 1; i <= n; i++) {
    out[String(i)] = `${labels[(i - 1) % labels.length]}: ••••••••`;
  }
  return out;
};

type Seed = Partial<SearchResult> & {
  platform: string;
  platformUrl: string;
  category: Category;
  hasChecker: boolean;
  displayOrder: number;
};

function seed(s: Seed): SearchResult {
  return {
    username: "adoecido",
    displayName: null,
    pfpUrl: null,
    bannerUrl: null,
    userId: null,
    bio: null,
    followers: 0,
    following: 0,
    posts: 0,
    likes: null,
    isVerified: null,
    emails: [],
    website: null,
    createdAt: null,
    lastActive: null,
    location: null,
    isPrivate: false,
    locked: true,
    extras: {},
    profileUrl: `${s.platformUrl}/adoecido`,
    sourceQuery: "",
    duration: `${80 + Math.floor(Math.random() * 0)}ms`,
    cached: false,
    ...s,
  };
}

/**
 * Avatar 2D da cabeça (determinístico, sem precisar de UUID).
 * - mc-heads aceita nick direto: https://mc-heads.net/avatar/<nick>
 * - minotar (fallback): https://minotar.net/helm/<nick>/100.png
 * Research: fingerprint sse-results usa `mc-heads.net/avatar/<uuid>` e
 * `s.namemc.com/2d/skin/face.png?id=<hash>&scale=8` (precisam de ID/hash
 * da página do NameMC; sem ID, o mc-heads por nick é o fallback).
 */
export function minecraftHeadUrl(username: string): string {
  return `https://mc-heads.net/avatar/${encodeURIComponent(username)}`;
}

export function namemcProfileUrl(username: string): string {
  return `https://namemc.com/profile/${encodeURIComponent(username)}`;
}

/** Perfil Discord é por ID: https://discord.com/users/<snowflake>. */
export function discordProfileUrl(userIdOrNick: string): string {
  return `https://discord.com/users/${encodeURIComponent(userIdOrNick)}`;
}

const SOON = false;
const OK = true;

/**
 * The 49-hit result set for `adoecido`, stored in stream arrival order.
 * displayOrder controls the final grid layout.
 */
export const RESULTS: SearchResult[] = [
  // --- grid order 0..9 (spec 6.5) arrive interleaved in stream order ---
  seed({ platform: "Xbox", platformUrl: "https://xbox.com", category: "gaming", hasChecker: OK, displayOrder: 7, lastActive: iso(2025, 10, 2), extras: extra(3), posts: null, duration: "141ms", profileUrl: "https://xbox.com/pt-BR/play/user/adoecido" }),
  seed({ platform: "AniList", platformUrl: "https://anilist.co", category: "hobby", hasChecker: SOON, displayOrder: 1, createdAt: iso(2026, 4, 18), lastActive: iso(2026, 4, 18), extras: extra(13), duration: "207ms" }),
  seed({ platform: "DockerHub", platformUrl: "https://hub.docker.com", category: "coding", hasChecker: SOON, displayOrder: 0, createdAt: iso(2026, 8, 4), extras: extra(4), duration: "96ms", profileUrl: "https://hub.docker.com/u/adoecido" }),
  seed({ platform: "Disqus", platformUrl: "https://disqus.com", category: "social", hasChecker: SOON, displayOrder: 8, displayName: "adoecido", createdAt: iso(2019, 7, 10), extras: extra(6), duration: "178ms" }),
  seed({ platform: "Sourcehut", platformUrl: "https://sr.ht", category: "dev", hasChecker: SOON, displayOrder: 2, extras: extra(1), duration: "88ms" }),
  seed({ platform: "Fragment", platformUrl: "https://fragment.com", category: "misc", hasChecker: SOON, displayOrder: 5, extras: extra(4), duration: "122ms" }),
  seed({ platform: "Matrix", platformUrl: "https://matrix.org", category: "tech", hasChecker: SOON, displayOrder: 3, displayName: "Adoecido", extras: extra(1), duration: "154ms", profileUrl: "https://matrix.to/#/@adoecido:matrix.org" }),
  seed({ platform: "Division2", platformUrl: "https://tracker.gg", category: "gaming", hasChecker: SOON, displayOrder: 6, extras: extra(19), duration: "231ms", profileUrl: "https://tracker.gg/division-2/profile/adoecido" }),
  seed({ platform: "Kofi", platformUrl: "https://ko-fi.com", category: "social", hasChecker: SOON, displayOrder: 48, extras: extra(2), duration: "104ms" }),
  seed({ platform: "DeviantArt", platformUrl: "https://deviantart.com", category: "images", hasChecker: SOON, displayOrder: 9, username: "Adoecido", location: "Brazil", createdAt: iso(2026, 1, 4), extras: extra(10), duration: "189ms" }),
  // --- remaining hits in stream order ---
  seed({ platform: "Letterboxd", platformUrl: "https://letterboxd.com", category: "social", hasChecker: OK, displayOrder: 10, followers: 4, following: 11, posts: 27, createdAt: iso(2021, 9, 2), extras: extra(3), duration: "166ms" }),
  seed({ platform: "GitLab", platformUrl: "https://gitlab.com", category: "dev", hasChecker: OK, displayOrder: 4, displayName: "adoecido .", createdAt: iso(2025, 12, 9), extras: extra(0), duration: "132ms" }),
  seed({ platform: "Medal", platformUrl: "https://medal.tv", category: "streaming", hasChecker: SOON, displayOrder: 11, followers: 1, posts: 3, extras: extra(5), duration: "145ms" }),
  seed({ platform: "R6", platformUrl: "https://r6.tracker.network", category: "gaming", hasChecker: SOON, displayOrder: 12, extras: extra(16), duration: "219ms", profileUrl: "https://r6.tracker.network/r6siege/profile/adoecido" }),
  seed({ platform: "Lichess", platformUrl: "https://lichess.org", category: "gaming", hasChecker: OK, displayOrder: 13, followers: 2, following: 5, posts: null, createdAt: iso(2020, 3, 14), lastActive: iso(2024, 11, 30), extras: extra(6), duration: "118ms", profileUrl: "https://lichess.org/@/adoecido" }),
  // Discord é por ID (snowflake), não por nick: https://discord.com/users/<id>
  // Ex. real @adoecido -> 399573634961768479 (abre o perfil com foto + banner sólido).
  seed({ platform: "Discord", platformUrl: "https://discord.com", category: "gaming", hasChecker: OK, displayOrder: 14, displayName: "Adoecido", createdAt: iso(2019, 2, 22), extras: extra(4), duration: "201ms", profileUrl: "https://discord.com/users/399573634961768479", userId: "399573634961768479" }),
  seed({ platform: "RegisteredEmails", platformUrl: "https://haveibeenpwned.com", category: "misc", hasChecker: OK, displayOrder: 15, displayName: "adoecido", extras: { "1": "adoecido@outlook.com", "2": "adoecido@gmail.com", "3": "adoecido@hotmail.com", "4": "adoecido@yahoo.com" }, duration: "312ms", profileUrl: "https://haveibeenpwned.com" }),
  seed({ platform: "Reddit", platformUrl: "https://reddit.com", category: "dev", hasChecker: OK, displayOrder: 16, displayName: "Adoecido", followers: 1, posts: 9, createdAt: iso(2020, 6, 4), lastActive: iso(2026, 8, 14), extras: extra(7), duration: "158ms", profileUrl: "https://reddit.com/user/adoecido" }),
  seed({ platform: "Pastebin", platformUrl: "https://pastebin.com", category: "tech", hasChecker: OK, displayOrder: 17, posts: 2, createdAt: iso(2018, 12, 1), extras: extra(3), duration: "92ms", profileUrl: "https://pastebin.com/u/adoecido" }),
  seed({ platform: "Poe", platformUrl: "https://poe.com", category: "tech", hasChecker: SOON, displayOrder: 18, extras: extra(2), duration: "124ms" }),
  seed({ platform: "GitHub", platformUrl: "https://github.com", category: "dev", hasChecker: OK, displayOrder: 19, displayName: "Adoecido", createdAt: iso(2025, 12, 9), lastActive: iso(2025, 10, 2), extras: extra(1), duration: "115ms" }),
  seed({ platform: "Unsplash", platformUrl: "https://unsplash.com", category: "images", hasChecker: OK, displayOrder: 20, followers: 0, posts: 6, createdAt: iso(2022, 4, 27), extras: extra(4), duration: "149ms", profileUrl: "https://unsplash.com/@adoecido" }),
  seed({ platform: "Trakt", platformUrl: "https://trakt.tv", category: "video", hasChecker: SOON, displayOrder: 21, extras: extra(8), duration: "172ms" }),
  seed({ platform: "Fortnite", platformUrl: "https://fortnitetracker.com", category: "gaming", hasChecker: SOON, displayOrder: 22, extras: extra(12), duration: "243ms", profileUrl: "https://fortnitetracker.com/profile/all/adoecido" }),
  seed({ platform: "SoundCloud", platformUrl: "https://soundcloud.com", category: "music", hasChecker: OK, displayOrder: 23, displayName: "Adoecido", followers: 3, following: 8, posts: 1, createdAt: iso(2021, 8, 19), lastActive: iso(2025, 12, 3), extras: extra(9), duration: "167ms" }),
  seed({ platform: "Roblox", platformUrl: "https://roblox.com", category: "gaming", hasChecker: OK, displayOrder: 24, displayName: "Adoecido", createdAt: iso(2016, 5, 30), lastActive: iso(2023, 7, 11), extras: extra(5), duration: "198ms" }),
  seed({ platform: "HuggingFace", platformUrl: "https://huggingface.co", category: "tech", hasChecker: SOON, displayOrder: 25, extras: extra(3), duration: "127ms" }),
  seed({ platform: "Twitter/X", platformUrl: "https://x.com", category: "social", hasChecker: OK, displayOrder: 26, displayName: "Xabi Cobelo", followers: 12, following: 34, posts: 58, createdAt: iso(2014, 8, 7), lastActive: iso(2026, 9, 30), location: "SP – Terra da Garoa", extras: extra(11), duration: "274ms", profileUrl: "https://x.com/adoecido" }),
  seed({ platform: "Twitch", platformUrl: "https://twitch.tv", category: "streaming", hasChecker: OK, displayOrder: 27, displayName: "Adoecido", followers: 6, posts: null, createdAt: iso(2017, 10, 15), lastActive: iso(2025, 9, 8), extras: extra(4), duration: "136ms" }),
  seed({ platform: "Genius", platformUrl: "https://genius.com", category: "music", hasChecker: SOON, displayOrder: 28, extras: extra(4), duration: "114ms", profileUrl: "https://genius.com/adoecido" }),
  seed({ platform: "Spotify", platformUrl: "https://open.spotify.com", category: "streaming", hasChecker: OK, displayOrder: 29, displayName: "Adriano", followers: 2, posts: null, createdAt: iso(2020, 1, 25), extras: extra(6), duration: "152ms", profileUrl: "https://open.spotify.com/user/adoecido" }),
  seed({ platform: "MySpace", platformUrl: "https://myspace.com", category: "music", hasChecker: OK, displayOrder: 30, displayName: "adoecido", createdAt: iso(2005, 11, 9), extras: extra(2), duration: "301ms" }),
  seed({ platform: "Wordpress", platformUrl: "https://wordpress.com", category: "blog", hasChecker: SOON, displayOrder: 31, posts: 4, extras: extra(2), duration: "109ms" }),
  seed({ platform: "Pinterest", platformUrl: "https://pinterest.com", category: "social", hasChecker: OK, displayOrder: 32, displayName: "Adoecido", followers: 1, posts: 14, createdAt: iso(2019, 11, 3), extras: extra(3), duration: "121ms" }),
  seed({ platform: "LastFM", platformUrl: "https://last.fm", category: "music", hasChecker: OK, displayOrder: 33, createdAt: iso(2018, 4, 16), lastActive: iso(2024, 6, 22), extras: extra(7), duration: "143ms", profileUrl: "https://last.fm/user/adoecido" }),
  seed({ platform: "YouTube", platformUrl: "https://youtube.com", category: "streaming", hasChecker: OK, displayOrder: 34, displayName: "Adoecido", followers: 9, posts: 3, createdAt: iso(2016, 8, 12), lastActive: iso(2026, 7, 2), extras: extra(8), duration: "184ms", profileUrl: "https://youtube.com/@adoecido" }),
  seed({ platform: "Xvideos", platformUrl: "https://xvideos.com", category: "xx NSFW xx", hasChecker: OK, displayOrder: 35, extras: extra(2), isPrivate: true, duration: "97ms" }),
  seed({ platform: "Codeberg", platformUrl: "https://codeberg.org", category: "coding", hasChecker: SOON, displayOrder: 36, extras: extra(2), duration: "105ms" }),
  seed({ platform: "Habbo", platformUrl: "https://habbo.com", category: "gaming", hasChecker: SOON, displayOrder: 37, displayName: "Adoecido", extras: extra(6), duration: "156ms", profileUrl: "https://habbo.com/profile/adoecido" }),
  seed({ platform: "Telegram", platformUrl: "https://t.me", category: "social", hasChecker: OK, displayOrder: 38, displayName: "Xabi Adoecido", lastActive: iso(2026, 5, 19), extras: extra(3), duration: "168ms", profileUrl: "https://t.me/adoecido" }),
  seed({ platform: "XDAForums", platformUrl: "https://xdaforums.com", category: "tech", hasChecker: SOON, displayOrder: 39, posts: 7, createdAt: iso(2015, 3, 9), extras: extra(5), duration: "133ms" }),
  seed({ platform: "Blogspot", platformUrl: "https://blogspot.com", category: "blog", hasChecker: SOON, displayOrder: 40, posts: 2, extras: extra(2), duration: "95ms" }),
  seed({ platform: "TikTok", platformUrl: "https://tiktok.com", category: "social", hasChecker: OK, displayOrder: 41, displayName: '" THEREAL FT', followers: 23, following: 51, posts: 12, createdAt: iso(2021, 2, 14), lastActive: iso(2026, 9, 12), extras: extra(9), duration: "221ms", profileUrl: "https://tiktok.com/@adoecido" }),
  seed({ platform: "Ebay", platformUrl: "https://ebay.com", category: "shopping", hasChecker: SOON, displayOrder: 42, followers: 0, createdAt: iso(2017, 6, 27), extras: extra(4), duration: "119ms", profileUrl: "https://ebay.com/usr/adoecido" }),
  seed({ platform: "RuneScape", platformUrl: "https://secure.runescape.com", category: "hobby", hasChecker: SOON, displayOrder: 43, extras: extra(7), duration: "147ms" }),
  // NameMC: link canônico https://namemc.com/profile/<nick> (+ `.1` no search).
  // pfp = cabeça 2D da 1ª skin: s.namemc.com/2d/skin/face.png?id=<hash>&scale=8
  // (research fingerprint sse-results; sem hash, fallback mc-heads por nick).
  seed({ platform: "NameMC", platformUrl: "https://namemc.com", category: "gaming", hasChecker: SOON, displayOrder: 44, username: "Adoecido", extras: extra(5), duration: "88ms", profileUrl: "https://namemc.com/profile/adoecido", pfpUrl: "https://s.namemc.com/2d/skin/face.png?id=0d28b84ca23c89f2&scale=8", userId: "f0a6febc-9dbf-4744-ba11-999b02649908" }),
  seed({ platform: "Pixabay", platformUrl: "https://pixabay.com", category: "images", hasChecker: SOON, displayOrder: 45, posts: 3, extras: extra(3), duration: "101ms", profileUrl: "https://pixabay.com/users/adoecido" }),
  seed({ platform: "GunsLol", platformUrl: "https://guns.lol", category: "social", hasChecker: OK, displayOrder: 46, displayName: "X", extras: extra(8), duration: "126ms", profileUrl: "https://guns.lol/adoecido" }),
  // Minecraft usa o mesmo perfil do NameMC. Sem foto? Usa a cabeça 2D:
  // mc-heads.net/avatar/<uuid|nick> (research fingerprint).
  seed({ platform: "Minecraft", platformUrl: "https://www.minecraft.net", category: "gaming", hasChecker: OK, displayOrder: 47, username: "Adoecido", createdAt: iso(2013, 5, 21), extras: extra(3), duration: "162ms", profileUrl: "https://namemc.com/profile/adoecido", pfpUrl: "https://mc-heads.net/avatar/f0a6febc-9dbf-4744-ba11-999b02649908", userId: "f0a6febc-9dbf-4744-ba11-999b02649908" }),
];

export const TOTAL_PLATFORMS = 1199; // 29 natives + 1170 mf:/wm: (audit 2026-10)

/** Build the exact event sequence for both the simulator and the SSE route. */
export function buildStreamEvents(query: string): StreamEvent[] {
  const events: StreamEvent[] = [
    { name: "connected", data: JSON.stringify({ status: "connected" }) },
    { name: "start", data: JSON.stringify({ total: TOTAL_PLATFORMS }) },
  ];
  let checked = 0;
  const step = Math.floor(TOTAL_PLATFORMS / RESULTS.length);
  for (const r of RESULTS) {
    const result = adaptResultToQuery(r, query);
    events.push({ name: "result", data: JSON.stringify(result) });
    checked = Math.min(TOTAL_PLATFORMS, checked + step);
    events.push({ name: "checked", data: JSON.stringify({ total: checked }) });
  }
  events.push({ name: "done", data: JSON.stringify({ type: "done", total: TOTAL_PLATFORMS }) });
  return events;
}

/**
 * O mock era hardcoded para `adoecido`: pesquisar qualquer outro nick ainda
 * devolvia `@adoecido` em todos os cards. Agora o resultado é adaptado para
 * a query (username, profileUrl, displayName, emails e avatares genéricos).
 */
export function adaptResultToQuery(r: SearchResult, rawQuery: string): SearchResult {
  const q = (rawQuery ?? "").trim().replace(/^@/, "");
  if (!q || q.toLowerCase() === "adoecido") return { ...r, sourceQuery: rawQuery };
  const out = { ...r, username: q, sourceQuery: rawQuery };
  // profileUrl: troca o nick embutido (inclui caso Matrix @nick:matrix.org)
  out.profileUrl = r.profileUrl.replace(/adoecido/gi, q);
  // Discord: só conhecemos o snowflake do @adoecido; resto usa padrão por nick
  if (r.platform.toLowerCase() === "discord") {
    out.profileUrl = discordProfileUrl(q);
    out.userId = null;
  }
  // Minecraft/NameMC: sem UUID/hash do usuário pesquisado, usa cabeça 2D por nick
  if (r.platform.toLowerCase() === "minecraft" || r.platform.toLowerCase() === "namemc") {
    out.profileUrl = namemcProfileUrl(q);
    out.pfpUrl = minecraftHeadUrl(q);
    out.userId = null;
  }
  // displayName: se menciona adoecido, troca; senão limpa (era nome de outra pessoa)
  if (out.displayName) {
    if (/adoecido/i.test(out.displayName)) {
      out.displayName = out.displayName.replace(/adoecido/gi, q);
    } else {
      out.displayName = null;
    }
  }
  // location/emails do fixture são do @adoecido — não valem p/ outro nick
  out.location = null;
  out.emails = [];
  if (r.platform === "RegisteredEmails") {
    out.displayName = q;
    out.extras = {
      "1": `${q}@outlook.com`,
      "2": `${q}@gmail.com`,
      "3": `${q}@hotmail.com`,
      "4": `${q}@yahoo.com`,
    };
  } else if (out.extras) {
    const fixed: Record<string, string> = {};
    for (const [k, v] of Object.entries(out.extras)) fixed[k] = v.replace(/adoecido/gi, q);
    out.extras = fixed;
  }
  return out;
}

const normTok = (s: string) => s.toLowerCase().replace(/[^a-z0-9]/g, "");

/** Aliases key <-> nome exibido (ex. "x" <-> "Twitter/X"). */
const PLAT_ALIASES: Record<string, string[]> = {
  x: ["twitterx", "twitter"],
  twitter: ["x", "twitterx"],
  twitterx: ["x", "twitter"],
  guns_lol: ["gunslol"],
  gunslol: ["guns_lol"],
  gunslol2: ["guns_lol"],
  chesscom: ["chess"],
  chess: ["chesscom"],
  lastfm: ["lastfm"],
  "last_fm": ["lastfm"],
  r6: ["r6"],
  division2: ["division2"],
  "twitter/x": ["x", "twitter"],
};

function expandTok(t: string): string[] {
  const n = normTok(t);
  return [n, ...(PLAT_ALIASES[n] ?? [])];
}

/** Espelha o resolve_with do Rust (grupos + keys + ^exclude) p/ o mock local. */
export function filterResultsByPlatforms(results: SearchResult[], query: string): SearchResult[] {
  const q = (query ?? "").trim().toLowerCase();
  if (!q || q === "all") return results;
  const groups = ["social", "dev", "gaming", "streaming", "music", "community", "archived", "art", "blog", "business", "coding", "dating", "finance", "health", "hobby", "images", "misc", "news", "political", "shopping", "tech", "video", "xx nsfw xx"];
  const exclude: string[] = [];
  const incGroups: string[] = [];
  const incKeys: string[] = [];
  for (const part of q.split(",").map((s) => s.trim()).filter(Boolean)) {
    if (part.startsWith("^")) exclude.push(normTok(part.slice(1)));
    else if (groups.includes(part.toLowerCase())) incGroups.push(part.toLowerCase());
    else incKeys.push(normTok(part));
  }
  const useAll = incGroups.length === 0 && incKeys.length === 0;
  const incExpanded = incKeys.flatMap(expandTok);
  return results.filter((r) => {
    const rNorm = normTok(r.platform);
    const rCat = r.category.toLowerCase();
    const rKeys = [rNorm, ...(PLAT_ALIASES[rNorm] ?? [])];
    if (exclude.some((e) => rKeys.includes(e) || rCat === e)) return false;
    if (useAll) return true;
    if (incGroups.includes(rCat)) return true;
    return incExpanded.some((k) => rKeys.includes(k));
  });
}

/** Mock respeitando plataformas (o bug era ignorar e sempre checar tudo). */
export function buildStreamEventsFiltered(query: string, platformsQuery: string): StreamEvent[] {
  const filtered = filterResultsByPlatforms(RESULTS, platformsQuery);
  const total = filtered.length;
  const events: StreamEvent[] = [
    { name: "connected", data: JSON.stringify({ status: "connected" }) },
    { name: "start", data: JSON.stringify({ total }) },
  ];
  let checked = 0;
  for (const r of filtered) {
    const result = adaptResultToQuery(r, query);
    events.push({ name: "result", data: JSON.stringify(result) });
    checked += 1;
    events.push({ name: "checked", data: JSON.stringify({ total: checked }) });
  }
  events.push({ name: "done", data: JSON.stringify({ type: "done", total }) });
  return events;
}

/* ------------------------------------------------------------------ */
/* Aggregator panel data (spec 6.5)                                     */
/* ------------------------------------------------------------------ */
export const AGGREGATIONS = {
  pictures: { count: 42, shown: 12 },
  banners: { count: 9, shown: 4 },
  usernames: ["adoecido"],
  names: ["adoecido", "Xabi Cobelo", "adoecido .", "Xabi Adoecido", "Adriano", '" THEREAL FT', "X"],
  locations: ["Brazil", "Spain", "SP – Terra da Garoa"],
  emails: ["adoecido@outlook.com", "adoecido@gmail.com", "adoecido@hotmail.com", "adoecido@yahoo.com"],
};

export const FIRST_SEEN = iso(2005, 11, 9);
export const LAST_SEEN = iso(2026, 9, 30);

export function fmtDate(isodate: string | null): string | null {
  if (!isodate) return null;
  const d = new Date(isodate);
  if (Number.isNaN(d.getTime())) return null;
  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
    timeZone: "UTC",
  }).format(d);
}

export function fmtCount(n: number | null): string {
  if (n === null || n === undefined) return "0";
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

/** Favicon direto por host (evita o globo do Google quando a URL é inválida). */
const FAVICON_DIRECT: Array<[string, string]> = [
  ["discord.com", "https://discord.com/assets/favicon.ico"],
  ["namemc.com", "https://namemc.com/favicon.ico"],
  ["minecraft.net", "https://www.minecraft.net/favicon.ico"],
  ["github.com", "https://github.com/favicon.ico"],
  ["x.com", "https://x.com/favicon.ico"],
  ["twitch.tv", "https://twitch.tv/favicon.ico"],
];

export function faviconUrl(platformUrl: string): string {
  const u = (platformUrl || "").toLowerCase();
  // Legado do checker: "(pomelo)" não é URL — cai no logo do Discord.
  if (!u.startsWith("http") || u.includes("pomelo")) {
    if (u.includes("pomelo")) return FAVICON_DIRECT[0][1];
  }
  for (const [host, icon] of FAVICON_DIRECT) {
    if (u.includes(host)) return icon;
  }
  return `https://t2.gstatic.com/faviconV2?client=SOCIAL&type=FAVICON&url=${encodeURIComponent(
    platformUrl
  )}&size=32`;
}

/** Cor sólida de fallback p/ banner (sempre tem imagem, nunca vazio). */
export function bannerFallback(platform: string): string {
  const p = (platform || "").toLowerCase();
  if (p === "discord") return "#5865F2";
  // Cor determinística por plataforma (tom escuro, clean).
  let h = 0;
  for (let i = 0; i < p.length; i++) h = (h * 31 + p.charCodeAt(i)) % 360;
  return `hsl(${h}, 28%, 14%)`;
}
