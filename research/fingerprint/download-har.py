"""
Extrator HAR offline-first para fingerprint.to (sem rate-limit).

Por que o script antigo tomava rate-limit:
  - Re-baixava cada URL via urllib com 500+ requests sequenciais,
    sem headers/cookies/sessao, sem delay, sem retry/backoff,
    sem deduplicacao (100x faviconV2 com ?url=... colidiam no mesmo path),
    e batia em dominios anti-bot (cloudflare, xboxlive, google, etc).

Novo comportamento (default = 100% offline, 0 requests):
  python download-har.py [arquivo.har] [saida/] [--skip-images] [--only-fingerprint]

  - Le response.content.text embutido no HAR e salva em disco.
  - Decodifica base64 automaticamente.
  - Nome inteligente: inclui hash da query p/ nao colidir (?url=... , ?_rsc=...).
  - Deduplica por URL (o HAR tem ~551 entries mas muitas repetidas).
  - Pula ruido (sentry, beacon, challenge oneshot vazio).
  - Gera resumos p/ engenharia de prompt:
      _har-meta/urls.txt            lista unica de URLs
      _har-meta/summary.json        contagens por dominio/mime/status
      _har-meta/sse-results.json    SSE /api/search parseado (49 hits p/ adoecido)
      _har-meta/apis.json           /api/rate-limit, /history, /data-search, /auth/me, etc
      _har-meta/site-map.md         mapa pronto p/ colar no prompt do website

Modo opcional com rede (so p/ o que NAO esta no HAR):
  python download-har.py fingerprint.to.har site-dump --refetch-missing --delay 2.5

  - So baixa entries sem content.text no HAR.
  - 1 request por vez + delay + jitter + backoff exponencial em 429 + Retry-After.
  - Session unica com UA real. Dominios de telemetria nunca sao refetchados.
"""
import argparse
import base64
import hashlib
import json
import os
import re
import sys
import time
import random
import urllib.parse
import urllib.request
import urllib.error

SKIP_SUBSTRINGS = (
    "sentry.io/api",              # telemetria, sempre bloqueado / inutil
    "static.cloudflareinsights.com/beacon",  # beacon vazio
    "cdn-cgi/challenge-platform/h/b/jsd/oneshot",  # challenge vazio
    "cdn-cgi/challenge-platform/h/b/fo/",           # challenge ofuscado, vazio/inutil p/ prompt
)

# dominios que SEMPRE dao 429/bot-wall se refetchados sem cookies reais
NO_REFETCH_DOMAINS = (
    "challenges.cloudflare.com",
    "images-eds-ssl.xboxlive.com",
    "pbs.twimg.com",
    "t2.gstatic.com",  # favicon: refetch = centenas de hits rapidos -> 429
)


def sanitize(name: str) -> str:
    return re.sub(r'[<>:"|?*]', "_", name)


def local_path_for_url(url: str, output: str) -> str:
    parsed = urllib.parse.urlparse(url)
    path = urllib.parse.unquote(parsed.path or "/")
    if not path or path.endswith("/"):
        path += "index.html"
    # query vira sufixo com hash curto p/ evitar colisao:
    # ex: /faviconV2?url=https://reddit.com&size=32 -> faviconV2.q-a1b2c3d4.png
    #     /dashboard/search?_rsc=xxx           -> search.rsc-xxx.dat
    #     /api/search?query=adoecido...       -> search.q-xxx.sse
    query = parsed.query
    path = sanitize(path)
    local = os.path.join(output, sanitize(parsed.netloc), path.lstrip("/"))

    if query:
        qhash = hashlib.sha1(query.encode()).hexdigest()[:8]
        qs = urllib.parse.parse_qs(query)
        hint = ""
        # hints legiveis p/ os casos que importam p/ prompt
        if "url" in qs and qs["url"]:
            try:
                host = urllib.parse.urlparse(qs["url"][0]).netloc or qs["url"][0][:30]
                hint = ".u-" + sanitize(host)[:30]
            except Exception:
                pass
        elif "_rsc" in qs:
            hint = ".rsc-" + sanitize(qs["_rsc"][0][:12])
        elif "query" in qs:
            hint = ".q-" + sanitize(qs["query"][0][:24])
        elif "size" in qs:
            hint = ".s" + sanitize(qs["size"][0][:6])
        root, ext = os.path.splitext(local)
        # preserva extensao real; se nao houver, adivinha pelos casos do fingerprint.to
        if not ext:
            if "_rsc=" in query:
                ext = ".rsc.txt"
            elif "/api/search?" in url:
                ext = ".sse.txt"
            elif "faviconV2" in url:
                ext = ".png"
        local = f"{root}{hint}.q-{qhash}{ext}"
    return local


def decode_content(content: dict):
    """Retorna (bytes|None, era_base64). None = entry sem body no HAR."""
    text = content.get("text")
    if text is None or text == "":
        return None, False
    if content.get("encoding") == "base64":
        try:
            return base64.b64decode(text), True
        except Exception as e:
            return None, False
    # texto puro (html/css/js/json/RSC/SSE/svg)
    mime = content.get("mimeType", "")
    charset = "utf-8"
    m = re.search(r"charset=([\w-]+)", mime)
    if m:
        charset = m.group(1)
    try:
        return text.encode(charset, errors="replace"), False
    except Exception:
        return text.encode("utf-8", errors="replace"), False


def parse_sse(text: str):
    """Parseia o text/event-stream do /api/search em eventos + results."""
    events = []
    results = []
    # SSE: blocos separados por linha em branco
    for block in text.split("\n\n"):
        ev = None
        data = None
        for line in block.splitlines():
            if line.startswith("event:"):
                ev = line[len("event:"):].strip()
            elif line.startswith("data:"):
                chunk = line[len("data:"):].strip()
                data = chunk if data is None else data + "\n" + chunk
        if ev and data is not None:
            events.append({"event": ev, "data_raw": data[:5000]})
            if ev == "result":
                try:
                    results.append(json.loads(data))
                except Exception:
                    events[-1]["parse_error"] = True
    return events, results


def main():
    ap = argparse.ArgumentParser(description="Extrator HAR offline-first (sem rate-limit)")
    ap.add_argument("har_file", nargs="?", default="fingerprint.to.har")
    ap.add_argument("output", nargs="?", default="site-dump")
    ap.add_argument("--skip-images", action="store_true",
                    help="Nao extrai imagens/fonts (so html/css/js/json/RSC/SSE). Bom p/ base de prompt.")
    ap.add_argument("--only-fingerprint", action="store_true",
                    help="Extrai apenas fingerprint.to (+ RSC/API), ignora terceiros (avatars/favicons).")
    ap.add_argument("--refetch-missing", action="store_true",
                    help="Baixa via rede SOMENTE entries sem body no HAR, com throttle anti-429.")
    ap.add_argument("--delay", type=float, default=2.5,
                    help="Delay base entre requests no refetch (default 2.5s + jitter).")
    ap.add_argument("--timeout", type=int, default=30)
    args = ap.parse_args()

    with open(args.har_file, "r", encoding="utf-8") as f:
        har = json.load(f)
    entries = har["log"]["entries"]
    print(f"HAR: {args.har_file} | entries: {len(entries)}")

    meta_dir = os.path.join(args.output, "_har-meta")
    os.makedirs(meta_dir, exist_ok=True)
    apis_dir = os.path.join(args.output, "_har-meta", "apis")
    os.makedirs(apis_dir, exist_ok=True)

    seen_url = set()
    seen_hash = set()  # dedup de conteudo identico (favicons repetidos)
    stats = {"total": len(entries), "extracted": 0, "dedup_url": 0,
             "dedup_content": 0, "skipped_noise": 0, "skipped_filter": 0,
             "empty_no_body": [], "refetched": 0, "refetch_failed": []}
    domain_count = {}
    mime_count = {}
    sse_results_all = []
    api_dump = {}

    for i, entry in enumerate(entries, 1):
        req = entry.get("request", {})
        resp = entry.get("response", {})
        url = req.get("url", "")
        if not url.startswith(("http://", "https://")):
            continue
        status = resp.get("status", 0)
        content = resp.get("content", {})
        mime = content.get("mimeType", "x-unknown")

        host = urllib.parse.urlparse(url).netloc
        domain_count[host] = domain_count.get(host, 0) + 1
        mime_count[mime] = mime_count.get(mime, 0) + 1

        if any(s in url for s in SKIP_SUBSTRINGS):
            stats["skipped_noise"] += 1
            continue
        if url in seen_url:
            stats["dedup_url"] += 1
            continue
        seen_url.add(url)

        if args.only_fingerprint and "fingerprint.to" not in url:
            stats["skipped_filter"] += 1
            continue
        if args.skip_images and (mime.startswith("image/") or mime.startswith("font/") or "/octet-stream" in mime):
            stats["skipped_filter"] += 1
            continue

        data, _was_b64 = decode_content(content)
        if data is None:
            # sem body no HAR (302/401/challenge/beacon) -> so refetch se pedido
            stats["empty_no_body"].append({"url": url, "status": status, "mime": mime})
            continue

        # dedup por hash de conteudo (o HAR repete ~100 favicons/avatars)
        h = hashlib.sha1(data).hexdigest()
        if h in seen_hash:
            stats["dedup_content"] += 1
            continue
        seen_hash.add(h)

        local = local_path_for_url(url, args.output)
        os.makedirs(os.path.dirname(local), exist_ok=True)
        with open(local, "wb") as f:
            f.write(data)
        stats["extracted"] += 1

        # captura APIs importantes p/ o prompt (sempre em _har-meta/apis/)
        if "fingerprint.to/api/" in url and status == 200:
            api_dump[url.split("?")[0].split("/api/")[-1]] = api_dump.get(url.split("?")[0].split("/api/")[-1], url)
            try:
                with open(os.path.join(apis_dir, sanitize(url.split("?")[0].split("/api/")[-1].replace("/", "_")) + ".txt"), "wb") as f:
                    f.write(data)
            except Exception:
                pass
        # captura o SSE de busca (o artefato mais valioso p/ engenharia)
        if "/api/search?query=" in url and status == 200:
            try:
                txt = data.decode("utf-8", errors="replace")
                _events, results = parse_sse(txt)
                sse_results_all = results
                with open(os.path.join(meta_dir, "sse-events.json"), "w", encoding="utf-8") as f:
                    json.dump(_events, f, ensure_ascii=False, indent=1)
                with open(os.path.join(meta_dir, "sse-results.json"), "w", encoding="utf-8") as f:
                    json.dump(results, f, ensure_ascii=False, indent=1)
            except Exception as e:
                print(f"  [aviso] falha ao parsear SSE: {e}")

    # ---------- refetch opcional (throttled) ----------
    if args.refetch_missing:
        print(f"\n--refetch-missing: {len(stats['empty_no_body'])} sem body. Delay {args.delay}s + jitter, backoff em 429.")
        opener = urllib.request.build_opener()
        opener.addheaders = [("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36")]
        for item in list(stats["empty_no_body"]):
            url = item["url"]
            host = urllib.parse.urlparse(url).netloc
            if any(d in host for d in NO_REFETCH_DOMAINS) or any(s in url for s in SKIP_SUBSTRINGS):
                continue
            local = local_path_for_url(url, args.output)
            if os.path.exists(local):
                continue
            delay = args.delay + random.uniform(0, 1.5)
            print(f"  [refetch] +{delay:.1f}s {url[:110]}")
            time.sleep(delay)
            ok = False
            for attempt in range(4):
                try:
                    with opener.open(url, timeout=args.timeout) as r:
                        if r.status == 429:
                            wait = int(r.headers.get("Retry-After", 15)) + attempt * 10
                            print(f"    429 -> espera {wait}s (tentativa {attempt+1}/4)")
                            time.sleep(wait)
                            continue
                        data = r.read()
                    os.makedirs(os.path.dirname(local), exist_ok=True)
                    with open(local, "wb") as f:
                        f.write(data)
                    stats["refetched"] += 1
                    ok = True
                    break
                except urllib.error.HTTPError as e:
                    if e.code == 429:
                        wait = int((e.headers or {}).get("Retry-After", 15)) + attempt * 10
                        print(f"    429 -> espera {wait}s (tentativa {attempt+1}/4)")
                        time.sleep(wait)
                        continue
                    print(f"    HTTP {e.code} -> desiste")
                    break
                except Exception as e:
                    print(f"    ERRO: {e} -> desiste")
                    break
            if not ok:
                stats["refetch_failed"].append(url)

    # ---------- artefatos p/ prompt ----------
    with open(os.path.join(meta_dir, "urls.txt"), "w", encoding="utf-8") as f:
        for u in sorted(seen_url):
            f.write(u + "\n")

    summary = {
        "har_file": args.har_file,
        "stats": {k: (len(v) if isinstance(v, list) else v) for k, v in stats.items()},
        "by_domain_top20": dict(sorted(domain_count.items(), key=lambda x: -x[1])[:20]),
        "by_mime": mime_count,
        "sse": {
            "n_results": len(sse_results_all),
            "platforms": [r.get("platform") for r in sse_results_all],
        },
    }
    with open(os.path.join(meta_dir, "summary.json"), "w", encoding="utf-8") as f:
        json.dump(summary, f, ensure_ascii=False, indent=1)

    # mapa markdown pronto p/ colar no prompt do website
    plats = [r.get("platform") for r in sse_results_all]
    with open(os.path.join(meta_dir, "site-map.md"), "w", encoding="utf-8") as f:
        f.write("# fingerprint.to — mapa extraido do HAR (offline, sem rate-limit)\n\n")
        f.write(f"- entries: {len(entries)} | extraidos: {stats['extracted']} | "
                f"dedup url: {stats['dedup_url']} | dedup conteudo: {stats['dedup_content']} | "
                f"ruido pulado: {stats['skipped_noise']}\n")
        f.write(f"- SSE /api/search: {len(sse_results_all)} results "
                f"({', '.join(plats[:15])}{'...' if len(plats) > 15 else ''})\n\n")
        f.write("## APIs (conteudo em `_har-meta/apis/`)\n")
        for k, v in sorted(api_dump.items()):
            f.write(f"- `{k}` <- {v[:100]}\n")
        f.write("\n## Arquitetura inferida do SSE\n")
        f.write("`GET /api/search?query={u}&type=username&turnstile=...` -> "
                "`text/event-stream` com eventos `connected -> ratelimit -> start{total} -> "
                "loop(result+checked) -> done`. Cada `result` tem "
                "`{platform, platformUrl, username, displayName?, pfpUrl?, userId?, "
                "followers/following/posts?, createdAt?, lastActive?, profileUrl, extras{...}, locked}`.\n")
        f.write("\n## Grupos L96A1 sugeridos (mapeamento UI -> backend Rust)\n")
        f.write("- social: Twitter/X, Reddit, Pinterest, Telegram, Myspace, Wordpress, Blogspot, Disqus, Habbo...\n")
        f.write("- dev: GitHub, GitLab, Codeberg, Sourcehut, HuggingFace, DockerHub...\n")
        f.write("- gaming: Xbox, Roblox, Minecraft/NameMC, R6/Division2-tracker.gg, Fortnite-tracker, Lichess...\n")
        f.write("- streaming/music: YouTube, Twitch, Spotify, SoundCloud, LastFM, Unsplash, Pixabay...\n")
        f.write("- community/misc: Pastebin, Poe, Trakt, Letterboxd, AniList, Kofi, DeviantArt, Medal, Ebay, TikTok, Xvideos, GunsLol, Matrix, Fragment, RuneScape, XDAForums, Genius...\n")

    print(f"\nOK: extraidos={stats['extracted']} dedup_url={stats['dedup_url']} "
          f"dedup_content={stats['dedup_content']} ruido={stats['skipped_noise']} "
          f"sem_body={len(stats['empty_no_body'])} refetched={stats['refetched']}")
    print(f"SSE results: {len(sse_results_all)} plataformas: {', '.join(plats[:12])}{'...' if len(plats) > 12 else ''}")
    print(f"Saida: {os.path.abspath(args.output)} (ver _har-meta/site-map.md)")


if __name__ == "__main__":
    main()
