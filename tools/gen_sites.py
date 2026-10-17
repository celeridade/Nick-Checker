#!/usr/bin/env python3
"""Gera os ~1176 checkers de site como structs Rust research-ready.

Fontes: sites.json (Sherlock, 481) + wmn.json (Blackbird/WMN, ~695 uteis).
Destino: crates/l96a1-checkers/src/platforms/<cat>/{mf,wm}_<site>.rs
         + mods por categoria + platforms/generated.rs (registro).

Comportamento = paridade total com os antigos motores genericos
(manifest.rs/wmn.rs): mesmas keys (mf:/wm:), mesmos validators,
mesmas evidencias. Cada struct carrega os dados bakeds como consts +
bloco RESEARCH vazio para aprimoramento manual posterior.

Uso:  python tools/gen_sites.py [--check]
  --check: so relatorio (overlap, categorias, colisoes), sem escrever.
"""

import json
import re
import sys
from collections import Counter, OrderedDict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PLAT = ROOT / "crates" / "l96a1-checkers" / "src" / "platforms"
SITES_JSON = ROOT / "sites.json"
WMN_JSON = ROOT / "wmn.json"

NATIVES = ['tiktok', 'instagram', 'bluesky', 'x', 'telegram', 'snapchat',
    'letterboxd', 'pinterest', 'tumblr', 'threads', 'medium', 'guns.lol',
    'github', 'gitlab', 'reddit', 'discord', 'steam', 'lichess', 'chess.com',
    'roblox', 'minecraft', 'youtube', 'spotify', 'twitch', 'kick',
    'soundcloud', 'last.fm', 'myspace', 'imgur']

# Excluídos do registro (duplicata de nativa com sinal FALSO — provado em
# research ao vivo 2026-10; a nativa cobre a plataforma com honestidade).
# (kind, nome-do-catálogo) -> motivo curto (vai no relatório --check).
EXCLUDE = {
    ('mf', 'Twitter'): 'probe nitter morta (nitter apagou em 2024); nativa x cobre',
    ('mf', 'TikTok'): 'errorMsg da era do bloqueio da Índia; nativa cobre',
    ('mf', 'Reddit'): '403-perímetro sem marcador => falso Taken; nativa Unknown honesto',
    ('mf', 'threads'): 'título de login nos dois estados => falso Available; nativa cobre',
    ('mf', 'Spotify'): 'shell 200 idêntico nos estados => falso Taken; nativa Unknown honesto',
    ('mf', 'Snapchat'): 'missing serve 200+NOT_FOUND => falso Taken; nativa cobre',
    ('mf', 'YouTube'): 'consent wall serve 200 sem marcador => falso Taken; nativa cobre @+legado',
}

# 21 cats do WMN mantidos como groups. Ordem = prioridade das keywords.
WMN_CATS = ["social", "gaming", "tech", "hobby", "coding", "misc", "finance",
    "business", "music", "shopping", "images", "blog", "art", "dating",
    "political", "video", "news", "archived", "health", "xx NSFW xx"]

# Grupo das 29 nativas (norm -> group): duplicado nativa herda o grupo.
NATIVE_GROUPS = {'tiktok': 'social', 'instagram': 'social', 'bluesky': 'social',
    'x': 'social', 'telegram': 'social', 'snapchat': 'social',
    'letterboxd': 'social', 'pinterest': 'social', 'tumblr': 'social',
    'threads': 'social', 'medium': 'social', 'gunslol': 'social',
    'github': 'dev', 'gitlab': 'dev', 'reddit': 'dev', 'discord': 'gaming',
    'steam': 'gaming', 'lichess': 'gaming', 'chesscom': 'gaming',
    'roblox': 'gaming', 'minecraft': 'gaming', 'youtube': 'streaming',
    'spotify': 'streaming', 'twitch': 'streaming', 'kick': 'streaming',
    'soundcloud': 'music', 'lastfm': 'music', 'myspace': 'music',
    'imgur': 'community'}

# Overrides exatos (nome do dict Sherlock) — casos que keyword erraria.
OVERRIDES = {
    '9GAG': 'social', 'Academia.edu': 'social', 'Archive.org': 'hobby',
    'Archive of Our Own': 'hobby', 'AuthorSTREAM': 'business',
    'BetaArchive': 'archived', 'CGTrader': 'shopping', 'Clozemaster': 'gaming',
    'Credly': 'business', 'F3.cool': 'social', 'Giphy': 'images',
    'Hashnode': 'blog', 'Holopin': 'coding', 'Itemfix': 'video',
    'LottieFiles': 'art', 'Memrise': 'hobby', 'MuseScore': 'music',
    'Nyaa.si': 'hobby', 'ObservableHQ': 'coding', 'Opennet': 'tech',
    'Patched': 'gaming', 'Platzi': 'hobby', 'RoyalCams': 'xx NSFW xx',
    'Scribd': 'hobby', 'Shelf': 'hobby', 'ShitpostBot5000': 'images',
    'Sketchfab': 'art', 'SpaceHey': 'social', 'Splice': 'music',
    'Status Cafe': 'social', 'Tellonym.me': 'social', 'Topmate': 'business',
    'Tuna': 'music', 'Twitter': 'social', 'Velog': 'blog', 'Wikipedia': 'hobby',
    'Wix': 'tech', 'Jimdo': 'tech', 'WebNode': 'tech', 'Carrd': 'tech',
    'ReverbNation': 'music', 'Airbit': 'music', 'AllMyLinks': 'social',
    'AniWorld': 'hobby', 'Anilist': 'hobby', 'Mydramalist': 'hobby',
    'GetMyUni': 'hobby', 'Harvard Scholar': 'business', 'Contently': 'business',
    'Sessionize': 'business', 'Moikrug': 'business', 'Fameswap': 'shopping',
    'SWAPD': 'shopping', 'Grailed': 'shopping', 'Chollometro': 'shopping',
    'Preisjaeger': 'shopping', 'Promodescuentos': 'shopping',
    'PepperNL': 'shopping', 'PepperPL': 'shopping', 'Sbazar.cz': 'shopping',
    'Pepperdeals': 'shopping', 'PepperealsUS': 'shopping',
    'Bazar.cz': 'shopping', 'Tiendanube': 'shopping', 'Empretienda AR': 'shopping',
    'Mercadolivre': 'shopping', 'Namuwiki': 'hobby', 'Livelib': 'hobby',
    'Akniga': 'hobby', 'Toster': 'coding', 'LOR': 'coding',
    'LinuxFR.org': 'coding', 'Habr': 'coding', 'Fixya': 'tech',
    'Needrom': 'tech', 'Prog.hu': 'coding', 'Programming.dev': 'coding',
    'Launchpad': 'coding', 'NotABug.org': 'coding', 'Packagist': 'coding',
    'RubyGems': 'coding', 'Gradle': 'coding', 'Replit.com': 'coding',
    'Vjudge': 'coding', 'DMOJ': 'coding', 'Cplusplus': 'coding',
    'Codolio': 'coding', 'CSSBattle': 'coding', 'CTAN': 'coding',
    'Laracast': 'coding', 'PentesterLab': 'coding', 'Write.as': 'blog',
    'Listed': 'blog', 'Znanylekarz.pl': 'health', '7Cups': 'health',
    'NitroType': 'gaming', 'Typeracer': 'gaming', 'CurseForge': 'gaming',
    'Polymart': 'gaming', 'Addons.wago.io': 'gaming', 'Star Citizen': 'gaming',
    'GaiaOnline': 'gaming', 'Blitz Tactics': 'gaming', 'Playstrategy': 'gaming',
    'Splits.io': 'gaming', 'PSNProfiles.com': 'gaming', 'VLR': 'gaming',
    'Ninja Kiwi': 'gaming', 'Realmeye': 'gaming', 'Exophase': 'gaming',
    'ColourLovers': 'art', 'ImgUp.cz': 'images', 'Jellyfin Weblate': 'tech',
    'PromoDJ': 'music', 'TheMovieDB': 'video', '2Dimensions': 'art',
    'AWS Skills Profile': 'business', 'Akniga': 'hobby',
    'Outgress': 'gaming', 'Giant Bomb': 'gaming', 'NationStates Nation': 'gaming',
    'NationStates Region': 'gaming', 'Speedrun.com': 'gaming',
    'Newgrounds': 'gaming', 'Mamot': 'social', 'Mastodon.xyz': 'social',
    'Mstdn.io': 'social', 'Fosstodon': 'social', 'LemmyWorld': 'social',
    'LessWrong': 'social', 'Kaskus': 'social', 'Nairaland.com': 'social',
    'Pikabu': 'social', 'Dcinside': 'social', 'Jbzd.com.pl': 'social',
    'D3RU': 'social', 'Spletnik': 'social', 'Bezuzyteczna': 'social',
    'IRC-Galleria': 'social', 'Couchsurfing': 'social', 'Ask Fedora': 'social',
    'Autofrage': 'social', 'Finanzfrage': 'social', 'Gesundheitsfrage': 'social',
    'Reisefrage': 'social', 'Gutefrage': 'social', 'Diskusjon.no': 'social',
    'BabyRu': 'social', 'Rumble': 'video', 'Odysee': 'video', 'Trovo': 'video',
    'SOOP': 'video', 'Clapper': 'video', 'Blipfoto': 'images',
    'Exposure': 'images', 'EyeEm': 'images', 'Rajce.net': 'images',
    'LinkedIn': 'business', 'Slack': 'business', 'Trawelling': 'hobby',
    'Drive2': 'hobby', 'MyMiniFactory': 'hobby', 'Football': 'hobby',
    'GoodReads': 'hobby', 'Instapaper': 'hobby', '1337x': 'tech',
    'Slashdot': 'tech',
}

# (keyword, cat) — primeira que bater vence. Especificas antes das gerais.
KEYWORDS = [
    (["porn", "xxx", "chaturbate", "myfreecams", "stripchat", "bongacams",
      "livejasmin", "camsoda", "rule34", "e621", "paheal", "gelbooru",
      "danbooru", "redgifs", "motherless", "imagefap", "xbooru", "yandere",
      "konachan", "e926", "hentai", "camgirl", " escort"], "xx NSFW xx"),
    (["archive", "abandonware"], "archived"),
    (["tinder", "bumble", "hinge", "okcupid", "plentyoffish", "badoo",
      "tagged", "skout", "meetme", "pof", "eharmony", "zoosk"], "dating"),
    (["dating", "flirt"], "dating"),
    (["etsy", "aliexpress", "shein", "temu", "olx", "avito", "vinted",
      "depop", "stockx", "dhgate", "wish", "jumia", "wallapop", "youla",
      "klekt", "restocks", "goat", "shop", "store", "market", "ebay",
      "amazon", "cart", "deal", "coupon", "auction"], "shopping"),
    (["paypal", "stripe", "wise", "revolut", "venmo", "cashapp", "zelle",
      "robinhood", "etoro", "coinbase", "binance", "kraken", "glassnode",
      "bank", "money", "coin", "crypto", "invest", "forex", "wallet",
      "cash", "bet", "casino"], "finance"),
    (["glassdoor", "indeed", "monster", "dice", "angel", "wellfound",
      "ycombinator", "producthunt", "crunchbase", "pitchbook", "hunter",
      "rocketreach", "clearbit", "job", "career", "hire", "company",
      "corp", "estate", "realty", "business"], "business"),
    (["strava", "fitbit", "myfitnesspal", "webmd", "goodrx", "doctolib",
      "zocdoc", "practo", "headspace", "betterhelp", "talkspace",
      "health", "fitness", "doctor", "diet", "pharma", "hospital"], "health"),
    (["unsplash", "pexels", "pixabay", "flickr", "500px", "imgbb",
      "postimages", "catbox", "pomf", "litterbox", "photo", "image",
      "wallpaper", "gallery", "png"], "images"),
    (["artstation", "behance", "dribbble", "cgsociety", "conceptart",
      "figma", "canva", "krita", "aseprite", "pixeljoint", "opengameart",
      "deviant", "dribbble"], "art"),
    (["substack", "ghost", "telegra.ph", "livejournal", "dreamwidth",
      "wattpad", "bloglovin", "blog"], "blog"),
    (["history"], "hobby"),
    (["wordpress", "journal", "diary", "story", "quote"], "blog"),
    (["bloomberg", "reuters", "associated press"], "news"),
    (["news", "press", "gazette", "herald", "tribune"], "news"),
    (["dailymotion", "vimeo", "bitchute", "bilibili", "rutube", "vkvideo",
      "tube"], "video"),
    (["bandcamp", "audius", "mixcloud", "datpiff", "spinrilla", "audiomack",
      "tidal", "deezer", "pandora", "shazam", "tunein", "somafm", "napster",
      "tabs", "lyrics", "chord", "metrolyrics", "musixmatch", "songtexte",
      "music", "audio", "song", "radio", "band", "sound", "beat",
      "lastfm", "discogs", "genius"], "music"),
    (["starcraft", "warcraft", "blizzard", "speedrun", "tracker",
      "leaderboard", "achievement", "trophy", "esports", "tournament",
      "faceit", "esea", "challonge", "battlefy", "toornament", "liquipedia",
      "dotabuff", "opendota", "hltv", "newbee", "minecraft", "roblox",
      "steam", "xbox", "chess", "gaming", "game", "playstation", "nintendo",
      "leagueoflegends", "fortnite", "valorant", "overwatch", "dota",
      "runescape", "habbo", "ogame", "travian", "kongregate", "armorgames",
      "newgrounds", "miniclip", "gamejolt", "nexusmods", "moddb",
      "itch.io"], "gaming"),
    (["leetcode", "hackerrank", "codewars", "exercism", "codeforces",
      "topcoder", "codechef", "atcoder", "brilliant", "datacamp",
      "codecademy", "freecodecamp", "sitepoint", "css-tricks", "smashing",
      "github", "gitlab", "stackoverflow", "npm", "pypi", "docker",
      "kernel", "code", "coding", "developer", "hack", "ctf", "exploit",
      "kaggle", "source", "arduino", "raspberry", "bugbounty",
      "bitbucket", "gitea", "codeberg", "savannah", "sourcehut",
      "readthedocs", "gitbook", "gitconnected"], "coding"),
    (["virustotal", "urlscan", "abuseipdb", "shodan", "censys", "greynoise",
      "crt.sh", "mxtoolbox", "whois", "filehost", "mega", "mediafire",
      "gofile", "wetransfer", "sendspace", "1fichier", "uptobox",
      "rapidgator", "nitroflare", "turbobit", "sourceforge",
      "filehippo", "softonic", "tech", "soft", "cloud", "server", "vps",
      "domain", "vpn", "proxy", "api", "cyber", "security", "phone",
      "mobile", "android", "mac", "windows", "torrent", "usenet",
      "seedbox"], "tech"),
    (["quora", "stackexchange", "4chan", "8kun", "2ch", "5ch", "futaba",
      "yikyak", "whisper", "reddit", "tumblr", "twitter", "mastodon",
      "lemmy", "misskey", "pleroma", "diaspora", "friendica", "hubzilla",
      "gnusocial", "post", "social", "chat", "forum", "community",
      "friend", "meet", "talk", "message", "comment", "discuss", "meme",
      "board"], "social"),
    (["design", "draw", "paint", "pixel", "vector"], "art"),
    (["udemy", "coursera", "skillshare", "masterclass", "khan", "edx",
      "treehouse", "fanfic", "archiveofourown",
      "baking", "brewing", "knitting", "woodwork",
      "astronomy", "birding", "geocache", "craft", "cook", "recipe",
      "garden", "pet", "moto", "bike", "sport", "fish", "hunt", "camping",
      "beer", "coffee", "book", "library"], "hobby"),
]


def norm(s):
    return ''.join(c for c in s.lower() if c.isalnum())


def mf_key(name):
    return 'mf:' + name.lower().replace(' ', '-').replace('_', '-')


def wm_key(name):
    return 'wm:' + name.lower().replace(' ', '-').replace('_', '-')


def mod_name(prefix, name, taken):
    base = re.sub(r'[^a-z0-9]+', '_', name.lower()).strip('_').strip('_')
    base = re.sub(r'_+', '_', base) or 'site'
    cand = f'{prefix}_{base}'
    if cand not in taken:
        return cand
    i = 2
    while f'{cand}_{i}' in taken:
        i += 1
    return f'{cand}_{i}'


def struct_name(mod):
    return ''.join(p[:1].upper() + p[1:] for p in mod.split('_'))


def rust_str(s):
    out = []
    for ch in s:
        if ch == '"':
            out.append('\\"')
        elif ch == '\\':
            out.append('\\\\')
        elif ch == '\n':
            out.append('\\n')
        elif ch == '\r':
            out.append('\\r')
        elif ch == '\t':
            out.append('\\t')
        else:
            o = ord(ch)
            out.append(ch if o >= 32 else '\\u{%x}' % o)
    return '"' + ''.join(out) + '"'


def folder_of(cat):
    f = re.sub(r'[^a-z0-9]+', '', cat.lower())
    return {'xxnsfwxx': 'nsfw'}.get(f, f)


def load_sources():
    sites = json.load(open(SITES_JSON, encoding='utf-8'))
    sites = {k: v for k, v in sites.items()
             if k != '$schema' and isinstance(v, dict) and v.get('url')}
    wmn = json.load(open(WMN_JSON, encoding='utf-8'))['sites']
    # mesma ordem/dedup do loader antigo: sort por nome, primeiro vence
    wmn_sorted = sorted([s for s in wmn if isinstance(s, dict)],
                        key=lambda s: s.get('name', ''))
    seen = set()
    wmn_u = []
    for s in wmn_sorted:
        n = s.get('name', '')
        if n and '{account}' in s.get('uri_check', '') and n not in seen:
            seen.add(n)
            wmn_u.append(s)
    return sites, wmn_u


OVR = {k.lower(): v for k, v in OVERRIDES.items()}


def sherlock_cat(name, entry, wm_by_norm):
    if name.lower() in OVR:
        return OVR[name.lower()], 'override'
    n = norm(name)
    if n in NATIVE_GROUPS:
        return NATIVE_GROUPS[n], 'native-inherit'
    if entry.get('isNSFW'):
        return 'xx NSFW xx', 'nsfw-flag'
    if n in wm_by_norm:
        return wm_by_norm[n], 'wmn-crossref'
    hay = (name + ' ' + entry.get('url', '')).lower()
    for kws, cat in KEYWORDS:
        if any(k in hay for k in kws):
            return cat, 'keyword'
    return 'misc', 'fallback'


MF_TEMPLATE = '''//! {name} (Sherlock manifest): {url_main}
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: {cat} | via: {via} | NSFW: {nsfw} | sample: {sample}
use async_trait::async_trait;
use l96a1_core::{{CheckCtx, CheckOutcome, UsernameValidator}};
use crate::platforms::site_probe;

pub struct {struct_};

pub const URL: &str = {url};
pub const URL_MAIN: &str = {url_main_rs};
pub const SAMPLE: Option<&str> = {sample_opt};
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {{
    error_type: site_probe::SiteErrorType::{etype},
    error_msg: &[{emsg}],
    error_code: &[{ecode}],
    regex: {regex},
    method: {method},
    payload: {payload},
    headers: &[{headers}],
    url_probe: {probe},
    error_url: {error_url},
}};

#[async_trait]
impl l96a1_core::Platform for {struct_} {{
    fn key(&self) -> &'static str {{
        {rkey}
    }}
    fn name(&self) -> &'static str {{
        {rname}
    }}
    fn group(&self) -> &'static str {{
        {rcat}
    }}
    fn profile(&self) -> l96a1_core::PlatformProfile {{
        l96a1_core::PlatformProfile {{ max_concurrent: 24, needs_proxy: false, sticky_proxy: false }}
    }}
    fn profile_url(&self, user: &str) -> String {{
        site_probe::interpolate(URL, user)
    }}
    fn validator(&self) -> UsernameValidator {{
        site_probe::validator_for(BAKED.regex)
    }}
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {{
        site_probe::probe_manifest(&BAKED, URL, user, ctx).await
    }}
}}
'''

WM_TEMPLATE = '''//! {name} (Blackbird/WMN): cat {cat}
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: {known}
use async_trait::async_trait;
use l96a1_core::{{CheckCtx, CheckOutcome, UsernameValidator}};
use crate::platforms::site_probe;

pub struct {struct_};

pub const URI: &str = {uri};
pub const SAMPLE: Option<&str> = {sample_opt};
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {{
    e_code: {ecode},
    e_string: {estring},
    m_code: {mcode},
    m_string: {mstring},
}};

#[async_trait]
impl l96a1_core::Platform for {struct_} {{
    fn key(&self) -> &'static str {{
        {rkey}
    }}
    fn name(&self) -> &'static str {{
        {rname}
    }}
    fn group(&self) -> &'static str {{
        {rcat}
    }}
    fn profile(&self) -> l96a1_core::PlatformProfile {{
        l96a1_core::PlatformProfile {{ max_concurrent: 24, needs_proxy: false, sticky_proxy: false }}
    }}
    fn profile_url(&self, user: &str) -> String {{
        URI.replace("{{account}}", user)
    }}
    fn validator(&self) -> UsernameValidator {{
        UsernameValidator::new(1, 100, r"^.*$")
    }}
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {{
        site_probe::probe_wmn(&BAKED, URI, user, ctx).await
    }}
}}
'''


def build_entries():
    sites, wmn_u = load_sources()
    wm_by_norm = {}
    for s in wmn_u:
        wm_by_norm.setdefault(norm(s['name']), s.get('cat', 'misc') or 'misc')
    entries = []  # (cat, mod, struct, key, kind, ctxdict)
    taken_mods = set()
    report = {'misc_fallback': [], 'collisions': [], 'native_dupes': [], 'excluded': []}
    nat = set(norm(n) for n in NATIVES)

    for name in sorted(sites):
        if ('mf', name) in EXCLUDE:
            report['excluded'].append(('mf', name, EXCLUDE[('mf', name)]))
            continue
        e = sites[name]
        cat, via = sherlock_cat(name, e, wm_by_norm)
        if via == 'fallback':
            report['misc_fallback'].append(name)
        mod = mod_name('mf', name, taken_mods)
        if mod != 'mf_' + re.sub(r'[^a-z0-9]+', '_', name.lower()).strip('_').strip('_'):
            report['collisions'].append(('mf', name, mod))
        taken_mods.add(mod)
        key = mf_key(name)
        if norm(name) in nat:
            report['native_dupes'].append(('mf', name, key))
        et = {'status_code': 'StatusCode', 'response_url': 'ResponseUrl'}.get(
            e.get('errorType') if isinstance(e.get('errorType'), str) else 'message',
            'Message')
        em = e.get('errorMsg')
        em = [em] if isinstance(em, str) else ([x for x in em] if isinstance(em, list) else [])
        em = [x for x in em if isinstance(x, str)]
        ec = e.get('errorCode')
        ec = [ec] if isinstance(ec, (int, float)) else ([int(x) for x in ec if isinstance(x, (int, float))] if isinstance(ec, list) else [])
        entries.append(('mf', cat, via, mod, struct_name(mod), key, name, {
            'url': e.get('url', ''), 'url_main': e.get('urlMain', ''),
            'etype': et, 'emsg': em, 'ecode': ec,
            'regex': e.get('regexCheck'),
            'method': str(e.get('request_method') or 'GET').upper(),
            'payload': e.get('request_payload'),
            'headers': e.get('headers') or {},
            'probe': e.get('urlProbe'),
            'error_url': e.get('errorUrl'),
            'nsfw': bool(e.get('isNSFW')),
            'sample': e.get('username_claimed'),
        }))

    for s in wmn_u:
        name = s['name']
        if ('wm', name) in EXCLUDE:
            report['excluded'].append(('wm', name, EXCLUDE[('wm', name)]))
            continue
        cat = s.get('cat') or 'misc'
        if not isinstance(cat, str) or not cat:
            cat = 'misc'
        mod = mod_name('wm', name, taken_mods)
        taken_mods.add(mod)
        key = wm_key(name)
        if norm(name) in nat:
            report['native_dupes'].append(('wm', name, key))
        known = s.get('known') or []
        known = [k for k in known if isinstance(k, str)][:4]
        entries.append(('wm', cat, 'wmn-cat', mod, struct_name(mod), key, name, {
            'uri': s.get('uri_check', ''),
            'ecode': int(s.get('e_code', 200)) if str(s.get('e_code', '')).lstrip('-').isdigit() else 200,
            'estring': str(s.get('e_string', '')),
            'mcode': int(s.get('m_code', 200)) if str(s.get('m_code', '')).lstrip('-').isdigit() else 200,
            'mstring': str(s.get('m_string', '')),
            'known': known,
            'sample': known[0] if known else None,
        }))
    return entries, report


def emit_site(entry):
    kind, cat, via, mod, struct_, key, name, d = entry
    folder = folder_of(cat)
    if kind == 'mf':
        emsg = ', '.join(rust_str(m) for m in d['emsg'])
        ecode = ', '.join(str(c) for c in d['ecode'])
        regex = rust_str(d['regex']) if d['regex'] else 'None'
        if d['regex']:
            regex = f'Some({regex})'
        payload = ('Some(' + rust_str(json.dumps(d['payload'], ensure_ascii=False)) + ')'
                   if d['payload'] is not None else 'None')
        headers = ', '.join(f'({rust_str(k)}, {rust_str(v)})'
                            for k, v in sorted(d['headers'].items())
                            if isinstance(v, str))
        probe = rust_str(d['probe']) if d['probe'] else 'None'
        if d['probe']:
            probe = f'Some({probe})'
        eurl = d.get('error_url')
        error_url = f'Some({rust_str(eurl)})' if isinstance(eurl, str) and eurl else 'None'
        sample = d['sample'] if isinstance(d['sample'], str) and d['sample'] else None
        body = MF_TEMPLATE.format(
            name=name, url_main=d['url_main'] or d['url'], cat=cat, via=via,
            nsfw=str(d['nsfw']).lower(), sample=sample or '-',
            struct_=struct_, url=rust_str(d['url']),
            url_main_rs=rust_str(d['url_main'] or d['url']),
            sample_opt=f'Some({rust_str(sample)})' if sample else 'None',
            etype=d['etype'], emsg=emsg, ecode=ecode, regex=regex,
            method=rust_str(d['method']), payload=payload, headers=headers,
            probe=probe, error_url=error_url,
            rkey=rust_str(key), rname=rust_str(name),
            rcat=rust_str(cat))
        return folder, mod, body
    else:
        known = ', '.join(d['known']) if d['known'] else '-'
        sample = d['sample']
        body = WM_TEMPLATE.format(
            name=name, cat=cat, known=known, struct_=struct_,
            uri=rust_str(d['uri']),
            sample_opt=f'Some({rust_str(sample)})' if sample else 'None',
            ecode=d['ecode'], estring=rust_str(d['estring']),
            mcode=d['mcode'], mstring=rust_str(d['mstring']),
            rkey=rust_str(key), rname=rust_str(name), rcat=rust_str(cat))
        return folder, mod, body


def write_all(check_only=False):
    entries, report = build_entries()
    by_folder = OrderedDict()
    for entry in entries:
        folder, mod, body = emit_site(entry)
        # corrige placeholders de formato vindos do .format
        by_folder.setdefault(folder, []).append((mod, body, entry))
    print(f'sites: {len(entries)} (mf={sum(1 for e in entries if e[0]=="mf")} '
          f'wm={sum(1 for e in entries if e[0]=="wm")})')
    print(f'pastas: {len(by_folder)} {sorted(by_folder)}')
    print(f'native_dupes (prefixados, revisar manual): {len(report["native_dupes"])}')
    print(f'excluidos (EXCLUDE, nativa cobre): {len(report["excluded"])}')
    for k, n, why in report['excluded']:
        print(f'  {k}:{n} — {why}')
    print(f'misc_fallback sherlock: {len(report["misc_fallback"])}')
    for n in report['misc_fallback']:
        print(f'  misc: {n}')
    print(f'colisoes de modulo: {len(report["collisions"])}')
    for k, n, m in report['collisions'][:20]:
        print(f'  {k} {n} -> {m}')
    print('por cat:', dict(Counter(e[1] for e in entries)))
    if check_only:
        return
    # 1. arquivos dos sites
    for folder, items in by_folder.items():
        d = PLAT / folder
        d.mkdir(exist_ok=True)
        for mod, body, _ in items:
            (d / f'{mod}.rs').write_text(body, encoding='utf-8')
    # 2. mods por categoria: preserva os hand-written (29 nativas) + anexa gerados
    for folder, items in by_folder.items():
        mods = sorted(mod for mod, _, _ in items)
        target = PLAT / 'native' / f'{folder}.rs'
        kept = []
        if target.exists():
            for ln in target.read_text(encoding='utf-8').splitlines():
                s = ln.strip()
                if (s.startswith('#[path') or s.startswith('pub mod ')) and \
                        not re.search(r'\b(mf|wm)_', s):
                    kept.append(ln)
        lines = [f'//! Checkers — categoria `{folder}` (nativas + geradas: ver tools/gen_sites.py).', '']
        lines += kept
        if kept:
            lines.append('')
        lines.append('// --- gerados (tools/gen_sites.py) ---')
        for mod in mods:
            lines.append(f'#[path = "../{folder}/{mod}.rs"]')
            lines.append(f'pub mod {mod};')
        target.write_text('\n'.join(lines) + '\n', encoding='utf-8')
    # 3. native.rs (mods)
    folders = sorted(by_folder)
    lines = ['//! Checkers nativos (29 hand-written + ~1176 gerados de manifest/WMN).',
             '//! Gerados: ver tools/gen_sites.py. Um modulo por site, pastas por categoria.', '']
    for f in folders:
        lines.append(f'pub mod {f};')
    lines += ['', '// Re-exports das 29 nativas (API publica historica).']
    for line in open(PLAT / 'native.rs', encoding='utf-8').read().splitlines():
        if line.startswith('pub use '):
            lines.append(line)
    (PLAT / 'native.rs').write_text('\n'.join(lines) + '\n', encoding='utf-8')
    # 4. generated.rs (registro)
    glines = ['//! Registro dos checkers gerados (tools/gen_sites.py).',
              'use l96a1_core::Platform;',
              'use std::sync::Arc;', '',
              'pub fn all_generated() -> Vec<Arc<dyn Platform>> {', '    vec![']
    for entry in sorted(entries, key=lambda e: (e[1], e[4])):
        _k, cat, _v, mod, struct_, _key, _n, _d = entry
        glines.append(f'        Arc::new(crate::platforms::native::{folder_of(cat)}::{mod}::{struct_}),')
    glines += ['    ]', '}']
    (PLAT / 'generated.rs').write_text('\n'.join(glines) + '\n', encoding='utf-8')
    # 6. teste de integridade PERMANENTE (API publica; sobrevive a limpeza)
    import re as _re
    bad_samples = []
    tlines = ['//! Integridade dos ~1176 checkers gerados (tools/gen_sites.py).',
              'use l96a1_core::Platform;',
              'use std::collections::HashSet;', '',
              'const SAMPLES: &[(&str, &str)] = &[', ]
    for entry in sorted(entries, key=lambda e: e[4]):
        _k, _c, _v, _m, _s, key, _n, d = entry
        sample = d.get('sample')
        if isinstance(sample, str) and sample:
            # sample que o proprio regex do site rejeita nao entra (quirk do catalogo)
            rx = d.get('regex') if _k == 'mf' else None
            if rx:
                try:
                    ok = bool(_re.search(rx, sample))
                except Exception:
                    ok = True
                if not ok:
                    bad_samples.append((key, sample, rx))
                    continue
            tlines.append(f'    ({rust_str(key)}, {rust_str(sample)}),')
    print(f'samples invalidos (fora da tabela): {len(bad_samples)}')
    for k, s, r in bad_samples[:20]:
        print(f'  {k} sample={s} regex={r}')
    tlines += ['];', '',
               '#[test]',
               'fn generated_keys_unique_and_shaped() {',
               '    let all = l96a1_checkers::all();',
               f'    assert_eq!(all.len(), {29 + len(entries)}, "29 nativas + geradas");',
               '    let mut seen = HashSet::new();',
               '    for p in &all {',
               '        assert!(seen.insert(p.key().to_string()), "key duplicada: {}", p.key());',
               '        assert!(!p.key().is_empty() && !p.name().is_empty());',
               '        assert!(l96a1_checkers::GROUPS.contains(&p.group()), "grupo fora do GROUPS: {}", p.group());',
               '    }',
               '}',
               '',
               '#[test]',
               'fn generated_samples_validate_and_link() {',
               '    let all = l96a1_checkers::all();',
               '    for (key, sample) in SAMPLES {',
               '        let p = all.iter().find(|p| p.key() == *key).unwrap_or_else(|| panic!("key sumiu: {key}"));',
               '        assert!(p.validator().normalize(sample).is_some(), "{key} rejeita sample {sample}");',
               '        let url = p.profile_url(sample);',
               '        // interpolado (contem o nick) ou estatico (3 sites sem placeholder, igual ao motor antigo)',
               '        assert!(url.contains(sample) || url == p.profile_url("ProbeNick123"),',
               '            "{key} profile_url inconsistente");',
               '    }',
               '}']
    tdir = ROOT / 'crates' / 'l96a1-checkers' / 'tests'
    tdir.mkdir(exist_ok=True)
    (tdir / 'generated_sites.rs').write_text('\n'.join(tlines) + '\n', encoding='utf-8')
    print('arquivos escritos OK')


if __name__ == '__main__':
    write_all(check_only='--check' in sys.argv)
