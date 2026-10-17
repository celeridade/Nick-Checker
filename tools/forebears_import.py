#!/usr/bin/env python3
"""L96A1 Forebears importer (Fase 1).

Baixa /<pais>/forenames e /<pais>/surnames do forebears.io e gera, sem
quebrar a organizacao atual do dataset:

  dataset/names/forebears/<pais>-forenames.tsv   (canonico, igual ao site)
  dataset/names/forebears/<pais>-forenames.txt    (usernames em ordem de rank)
  dataset/names/forebears/<pais>-surnames.tsv
  dataset/names/forebears/<pais>-surnames.txt
  dataset/names/forebears/manifest.json          (versao, fonte, checksum...)

Colunas do TSV (igual ao site + username ascii):
  rank, gender, gender_pct, name, slug, username, incidence, frequency_1x, source_url

  - gender: m | f | u (u = ambiguo/nao detectado; o markdown perde o rotulo
    m/f da barra visual, so o HTML tem class="m"/"f" + larguras px)
  - username: name transliterado p/ ASCII username-safe (Joao -> joao),
    lowercase, mantido [a-z0-9]; linha descartada se vazio.

Uso:
  py tools/forebears_import.py --countries brazil,japan,united-states,spain
  py tools/forebears_import.py --countries brazil --kinds forenames --delay 5
  py tools/forebears_import.py --via direct --countries japan --delay 8
  py tools/forebears_import.py --from-markdown seed.md --country brazil --kind forenames
  py tools/forebears_import.py --from-reader-file page.md --country brazil --kind surnames
  py tools/forebears_import.py --from-html page.html --country brazil --kind surnames
  py tools/forebears_import.py --from-html-dir ./saved --out dataset/names/forebears

Rotas (--via):
  reader (default): via r.jina.ai (IP de saida diferente, sem JS, markdown
    em tabela). Se rate-limitar, troque p/ direct ou rode mais tarde.
  direct: forebears.io direto com UA rotativo + delay + retry.
  manual: abra a pagina no seu navegador, salve como HTML e use
    --from-html/--from-html-dir (arquivos `<pais>-<kind>.html`).

Somente stdlib. Respeita o site: delay entre requests, retries com backoff,
resume automatico (pula artefato ja completo no manifest, salvo --overwrite).
"""

import argparse
import datetime
import hashlib
import json
import os
import random
import re
import sys
import time
import unicodedata
import urllib.parse
import urllib.request

UAS = [
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:127.0) Gecko/20100101 Firefox/127.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0 Safari/537.36",
]
BASE = "https://forebears.io"
READER = "https://r.jina.ai/https://forebears.io"

HTML_ROW_SPLIT = re.compile(r"<tr[^>]*>\s*<td[^>]*>", re.IGNORECASE)
HTML_TAG = re.compile(r"<[^>]+>")
HREF_RE = re.compile(r'href="[^"]*/(forenames|surnames)/([^"/]+)"[^>]*>([^<]+)</a>', re.IGNORECASE)
RANK_RE = re.compile(r"^(\d+)\s*</td>", re.IGNORECASE)
INCIDENCE_RE = re.compile(r"</a>\s*</td>\s*<td[^>]*>\s*([\d,]+)\s*</td>", re.IGNORECASE)
FREQ_RE = re.compile(r"1\s*:\s*([\d,]+)")
GENDER_100_RE = re.compile(r'>(\d{1,3})%<\s*', re.IGNORECASE)
GENDER_CLASS_RE = re.compile(r'class="(m|f)"', re.IGNORECASE)
GENDER_WIDTH_RE = re.compile(r"width\s*:\s*(\d+)\s*px", re.IGNORECASE)
UNIQUE_RE = re.compile(r"knows about\s+([\d,]+)\s+unique\s+(forenames|surnames)", re.IGNORECASE)
NEXTPAGE_RE = re.compile(r'href="([^"]*[?&]page=(\d+)[^"]*)"', re.IGNORECASE)

# markdown do webfetch: "1\n\n100%\n\n[Maria](forenames/maria)\n\n14,524,659\n\n1:15"
MD_ROW_RE = re.compile(
    r"(\d+)\s+(\d{1,3})%\s+\[([^\]]+)\]\(forenames/([^)]+)\)\s+([\d,]+)\s+1:([\d,]+)"
)
MD_ROW_SURN_RE = re.compile(
    r"(\d+)\s+(\d{1,3})%\s+\[([^\]]+)\]\(surnames/([^)]+)\)\s+([\d,]+)\s+1:([\d,]+)"
)

# tabela markdown do reader (r.jina.ai): linhas "| 1 | [da Silva](.../surnames/da_silva) | 12,585,868 | 1:17 |"
# forenames tem coluna extra de genero; surnames nao tem.
READER_LINK_RE = re.compile(r"\[([^\]]+)\]\((https?://[^)]*/(forenames|surnames)/([^)]+))\)")


def parse_reader_table(text, kind):
    """Parser da tabela markdown do reader. Retorna (rows, unique_total)."""
    rows = []
    m = re.search(r"knows about\s+([\d,]+)\s+unique\s+(forenames|surnames)", text, re.IGNORECASE)
    unique_total = int(m.group(1).replace(",", "")) if m else None
    for line in text.splitlines():
        line = line.strip()
        if not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.strip("|").split("|")]
        if len(cells) < 4:
            continue
        if not re.fullmatch(r"\d+", cells[0]):
            continue  # header/separador
        lm = READER_LINK_RE.search(line)
        if not lm or lm.group(3).lower() != kind:
            continue
        disp = lm.group(1).strip()
        slug = urllib.parse.unquote(lm.group(4).strip())
        nums = re.findall(r"[\d,]+", cells[-2])
        incidence = int(nums[0].replace(",", "")) if nums else 0
        fm = re.search(r"1\s*:\s*([\d,]+)", cells[-1])
        freq = fm.group(1).replace(",", "") if fm else ""
        gender, pct = "u", 0
        if len(cells) >= 5:  # tabela com coluna de genero (forenames)
            gm = re.search(r"(\d{1,3})\s*%", cells[1] if kind == "forenames" else "")
            pm = re.search(r"(\d{1,3})\s*%", " ".join(cells[1:-2]))
            if pm:
                pct = int(pm.group(1))
            # rotulo m/f nao sobrevive ao markdown; fica 'u' (igual seed)
        rows.append(
            {
                "rank": int(cells[0]),
                "gender": gender,
                "gender_pct": pct,
                "name": disp,
                "slug": slug,
                "incidence": incidence,
                "frequency_1x": freq,
            }
        )
    if not rows:
        # fallback: linhas TSV puras sem links "RANK\tNome\t1,234\t1:56"
        # (o reader as vezes devolve a tabela sem hyperlinks)
        for line in text.splitlines():
            parts = line.rstrip("\n").split("\t")
            if len(parts) != 4 or not re.fullmatch(r"\d+", parts[0].strip()):
                continue
            disp = parts[1].strip()
            if not disp or disp.lower() in ("surname", "forename"):
                continue
            nums = re.findall(r"[\d,]+", parts[2])
            fm = re.search(r"1\s*:\s*([\d,]+)", parts[3])
            rows.append(
                {
                    "rank": int(parts[0].strip()),
                    "gender": "u",
                    "gender_pct": 0,
                    "name": disp,
                    "slug": ascii_username(disp),
                    "incidence": int(nums[0].replace(",", "")) if nums else 0,
                    "frequency_1x": fm.group(1).replace(",", "") if fm else "",
                }
            )
    rows.sort(key=lambda r: r["rank"])
    return rows, unique_total


def ascii_username(name):
    nk = unicodedata.normalize("NFKD", name)
    out = []
    for ch in nk.lower():
        if "a" <= ch <= "z" or "0" <= ch <= "9":
            out.append(ch)
        # diacriticos viram base + combining mark (Mn) -> ignora o mark
        elif unicodedata.category(ch) == "Mn":
            continue
        # separadores viram nada em nomes (ex: "De Souza" -> "desouza")
        elif ch in (" ", "-", "'", "."):
            continue
        # resto descarta
    return "".join(out)


def parse_html_rows(html, kind):
    """Retorna (rows, unique_total_or_None). rows: dicts sem rank final."""
    rows = []
    skipped = 0
    m = UNIQUE_RE.search(html)
    unique_total = int(m.group(1).replace(",", "")) if m else None
    body = html
    tb = re.search(r"<tbody>(.*)</tbody>", html, re.IGNORECASE | re.DOTALL)
    if tb:
        body = tb.group(1)
    parts = HTML_ROW_SPLIT.split(body)
    for raw in parts[1:]:
        try:
            rk = RANK_RE.search(raw)
            rank = int(rk.group(1)) if rk else None
            hm = HREF_RE.search(raw)
            if not hm or hm.group(1).lower() != kind:
                skipped += 1
                continue
            slug, disp = hm.group(2), hm.group(3).strip()
            im = INCIDENCE_RE.search(raw)
            incidence = int(im.group(1).replace(",", "")) if im else 0
            fm = FREQ_RE.search(HTML_TAG.sub(" ", raw))
            freq = fm.group(1).replace(",", "") if fm else ""
            # genero: classe m/f + pct; markdown-like "100%" sozinho = dominante sem rotulo
            gm = GENDER_CLASS_RE.search(raw)
            pm = GENDER_100_RE.search(raw)
            pct = int(pm.group(1)) if pm else 0
            if gm:
                gender = gm.group(1).lower()
            else:
                gender = "u"
            rows.append(
                {
                    "rank": rank or 0,
                    "gender": gender,
                    "gender_pct": pct,
                    "name": disp,
                    "slug": slug,
                    "incidence": incidence,
                    "frequency_1x": freq,
                }
            )
        except Exception:
            skipped += 1
    # rank sequencial se ausente
    for i, r in enumerate(rows, 1):
        if not r["rank"]:
            r["rank"] = i
    rows.sort(key=lambda r: r["rank"])
    return rows, unique_total, skipped


def parse_markdown_rows(text, kind):
    pat = MD_ROW_RE if kind == "forenames" else MD_ROW_SURN_RE
    rows = []
    for rank, pct, disp, slug, inc, freq in pat.findall(text):
        rows.append(
            {
                "rank": int(rank),
                "gender": "u",  # markdown nao preserva o rotulo m/f da barra
                "gender_pct": int(pct),
                "name": disp,
                "slug": slug,
                "incidence": int(inc.replace(",", "")),
                "frequency_1x": freq.replace(",", ""),
            }
        )
    rows.sort(key=lambda r: r["rank"])
    return rows


def fetch(url, timeout=45, retries=5):
    """GET com UA rotativo e backoff exponencial + jitter."""
    last = None
    for i in range(retries):
        try:
            ua = random.choice(UAS)
            req = urllib.request.Request(url, headers={
                "User-Agent": ua,
                "Accept": "text/html,application/xhtml+xml",
                "Accept-Language": "en-US,en;q=0.9",
            })
            with urllib.request.urlopen(req, timeout=timeout) as h:
                return h.read().decode("utf-8", errors="replace")
        except Exception as e:  # noqa: BLE001 - rede instavel/rate-limit, tenta de novo
            last = e
            time.sleep((2 ** i) + random.uniform(0, 2))
    raise RuntimeError("fetch falhou %s: %s" % (url, last))


def manifest_done(outdir, country, kind):
    mp = os.path.join(outdir, "manifest.json")
    if not os.path.exists(mp):
        return False
    try:
        m = json.load(open(mp, encoding="utf-8"))
    except Exception:
        return False
    for e in m.get("artifacts", []):
        if e.get("country") == country and e.get("kind") == kind:
            return bool(e.get("complete")) or int(e.get("rows", 0)) >= 900
    return False


def write_artifact(outdir, country, kind, rows, source_url, complete, unique_total):
    os.makedirs(outdir, exist_ok=True)
    base = "%s-%s" % (country, kind)
    tsv_p = os.path.join(outdir, base + ".tsv")
    txt_p = os.path.join(outdir, base + ".txt")
    seen = set()
    dups = 0
    with open(tsv_p, "w", encoding="utf-8", newline="\n") as f:
        f.write("rank\tgender\tgender_pct\tname\tslug\tusername\tincidence\tfrequency_1x\tsource_url\n")
        for r in rows:
            uname = ascii_username(r["name"]) or ascii_username(r["slug"].replace("-", ""))
            if not uname or uname in seen:
                dups += 1
                continue
            seen.add(uname)
            f.write(
                "%d\t%s\t%d\t%s\t%s\t%s\t%d\t%s\t%s\n"
                % (
                    r["rank"], r["gender"], r["gender_pct"], r["name"],
                    r["slug"], uname, r["incidence"], r["frequency_1x"], source_url,
                )
            )
    with open(txt_p, "w", encoding="utf-8", newline="\n") as f:
        # mesma ordem de rank do TSV (popularidade preservada)
        with open(tsv_p, encoding="utf-8") as t:
            next(t)
            for line in t:
                f.write(line.rstrip("\n").split("\t")[5] + "\n")
    sha = hashlib.sha256(open(tsv_p, "rb").read()).hexdigest()
    return {
        "country": country,
        "kind": kind,
        "source_url": source_url,
        "tsv": os.path.basename(tsv_p),
        "txt": os.path.basename(txt_p),
        "rows": len(seen),
        "duplicates_skipped": dups,
        "complete": complete,
        "unique_total_site": unique_total,
        "sha256_tsv": sha,
        "fetched_at": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "columns": ["rank", "gender", "gender_pct", "name", "slug", "username",
                    "incidence", "frequency_1x", "source_url"],
    }


def save_manifest(outdir, entries):
    mp = os.path.join(outdir, "manifest.json")
    prev = {}
    if os.path.exists(mp):
        try:
            prev = json.load(open(mp, encoding="utf-8"))
        except Exception:
            prev = {}
    by = {(e["country"], e["kind"]): e for e in prev.get("artifacts", [])}
    for e in entries:
        by[(e["country"], e["kind"])] = e
    doc = {
        "version": prev.get("version", 1),
        "updated_at": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "source": "forebears.io",
        "normalization": "NFKD->ASCII lowercase, keep [a-z0-9]; ordem de popularidade (rank) preservada",
        "artifacts": sorted(by.values(), key=lambda e: (e["country"], e["kind"])),
    }
    json.dump(doc, open(mp, "w", encoding="utf-8"), ensure_ascii=False, indent=2)
    return mp


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--countries", default="brazil,japan,united-states,spain")
    ap.add_argument("--kinds", default="forenames,surnames")
    ap.add_argument("--out", default="dataset/names/forebears")
    ap.add_argument("--delay", type=float, default=3.0)
    ap.add_argument("--max-pages", type=int, default=1)
    ap.add_argument("--from-html", default="")
    ap.add_argument("--from-html-dir", default="",
                    help="pasta com <pais>-<kind>.html salvos no navegador")
    ap.add_argument("--from-markdown", default="")
    ap.add_argument("--from-reader-file", default="",
                    help="markdown em tabela do reader (r.jina.ai)")
    ap.add_argument("--country", default="brazil")
    ap.add_argument("--kind", default="forenames")
    ap.add_argument("--via", default="reader", choices=["reader", "direct"],
                    help="reader = via r.jina.ai (default, mais robusto); direct = site direto")
    ap.add_argument("--overwrite", action="store_true",
                    help="refaz mesmo artefato ja completo no manifest")
    a = ap.parse_args()

    entries = []
    if a.from_markdown:
        text = open(a.from_markdown, encoding="utf-8", errors="replace").read()
        rows = parse_markdown_rows(text, a.kind)
        url = "%s/%s/%s" % (BASE, a.country, a.kind)
        entries.append(write_artifact(a.out, a.country, a.kind, rows, url, False, None))
        print("seed markdown: %d linhas -> %s" % (len(rows), a.out))
    elif a.from_reader_file:
        text = open(a.from_reader_file, encoding="utf-8", errors="replace").read()
        rows, uniq = parse_reader_table(text, a.kind)
        url = "%s/%s/%s" % (BASE, a.country, a.kind)
        entries.append(write_artifact(a.out, a.country, a.kind, rows, url, False, uniq))
        print("seed reader: %d linhas -> %s" % (len(rows), a.out))
    elif a.from_html:
        html = open(a.from_html, encoding="utf-8", errors="replace").read()
        rows, uniq, skip = parse_html_rows(html, a.kind)
        url = "%s/%s/%s" % (BASE, a.country, a.kind)
        entries.append(write_artifact(a.out, a.country, a.kind, rows, url, False, uniq))
        print("seed html: %d linhas (%d puladas) -> %s" % (len(rows), skip, a.out))
    elif a.from_html_dir:
        for fn in sorted(os.listdir(a.from_html_dir)):
            m = re.fullmatch(r"([a-z-]+)-(forenames|surnames)\.html?", fn)
            if not m:
                continue
            c, k = m.group(1), m.group(2)
            html = open(os.path.join(a.from_html_dir, fn), encoding="utf-8", errors="replace").read()
            rows, uniq, skip = parse_html_rows(html, k)
            url = "%s/%s/%s" % (BASE, c, k)
            entries.append(write_artifact(a.out, c, k, rows, url, False, uniq))
            print("seed html %s/%s: %d linhas (%d puladas)" % (c, k, len(rows), skip))
    else:
        countries = [c.strip() for c in a.countries.split(",") if c.strip()]
        kinds = [k.strip() for k in a.kinds.split(",") if k.strip()]
        for c in countries:
            for k in kinds:
                if manifest_done(a.out, c, k) and not a.overwrite:
                    print("SKIP %s/%s (ja completo no manifest; use --overwrite p/ refazer)" % (c, k))
                    continue
                url = "%s/%s/%s" % (BASE, c, k)
                get_url = ("%s/%s/%s" % (READER, c, k)) if a.via == "reader" else url
                print("GET %s" % get_url)
                try:
                    if a.via == "reader":
                        text = fetch(get_url)
                        rows, uniq = parse_reader_table(text, k)
                        skip = 0
                    else:
                        html = fetch(get_url)
                        rows, uniq, skip = parse_html_rows(html, k)
                        pages = [get_url]
                        for pm in NEXTPAGE_RE.finditer(html):
                            nxt = pm.group(1)
                            if nxt not in pages and len(pages) < a.max_pages:
                                pages.append(nxt if nxt.startswith("http") else BASE + nxt)
                        for pu in pages[1:]:
                            time.sleep(a.delay + random.uniform(0, 2))
                            print("GET %s" % pu)
                            try:
                                h2 = fetch(pu)
                            except Exception as e:  # noqa: BLE001
                                print("  ERRO: %s" % e)
                                break
                            r2, _, s2 = parse_html_rows(h2, k)
                            skip += s2
                            rows.extend(r2)
                except Exception as e:  # noqa: BLE001
                    print("  ERRO: %s (tente --via %s, manual, ou mais tarde)" %
                          (e, "direct" if a.via == "reader" else "reader"))
                    continue
                rows.sort(key=lambda r: r["rank"])
                complete = uniq is not None and len(rows) >= uniq
                entries.append(write_artifact(a.out, c, k, rows, url, complete, uniq))
                print("  ok: %d linhas (%d puladas, complete=%s)" % (len(rows), skip, complete))
                time.sleep(a.delay + random.uniform(0, 2))
    if entries:
        mp = save_manifest(a.out, entries)
        print("manifest: %s" % mp)


if __name__ == "__main__":
    sys.exit(main())
