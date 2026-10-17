#!/usr/bin/env python3
"""L96A1 Trends importer (Fase 4): popularidade real por palavra.

Gera `dataset/trends.tsv` (`term\\tscore0-100[\\tupdated]`, `#` comenta),
lido por `l96a1_dataset::load_popularity` como boost de cobiça
(score >= 50 = cobiçado).

Duas fontes (a 1a que funcionar vence):
  1. pytrends (pip install pytrends): interesse real do Google Trends,
     em lotes de 5 termos com delay (a API não-oficial rate-limita fácil).
  2. CSV manual: `term,interest` (exporte do https://trends.google.com.br/
     ou da IA de pesquisa do Google — ex: "Celebridade é popular?" -> 85).

Uso:
  py tools/trends_import.py --terms dataset/names/forebears/brazil-forenames.txt --top 200
  py tools/trends_import.py --csv minha_pesquisa.csv --out dataset/trends.tsv
  py tools/trends_import.py --terms palavras.txt --csv base.csv --top 100

Somente stdlib, exceto pytrends (opcional).
"""

import argparse
import datetime
import sys
import time


def read_terms(path, top):
    out = []
    for line in open(path, encoding="utf-8", errors="replace"):
        t = line.strip().lower()
        if t and not t.startswith("#") and t not in out:
            out.append(t)
        if len(out) >= top:
            break
    return out


def read_csv(path):
    scores = {}
    for line in open(path, encoding="utf-8", errors="replace"):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = [p.strip() for p in line.replace(";", ",").split(",")]
        if len(parts) >= 2:
            try:
                scores[parts[0].lower()] = max(0, min(100, int(float(parts[1]))))
            except ValueError:
                pass
    return scores


def via_pytrends(terms, delay):
    from pytrends.request import TrendReq
    pt = TrendReq(hl="pt-BR", tz=-180)
    scores = {}
    for i in range(0, len(terms), 5):
        batch = terms[i:i + 5]
        try:
            pt.build_payload(batch, timeframe="today 12-m", geo="BR")
            df = pt.interest_over_time()
            if df is not None and not df.empty:
                mx = df[batch].max(numeric_only=True)
                for t in batch:
                    scores[t] = max(0, min(100, int(mx.get(t, 0))))
            print("  trends ok: %s" % ",".join(batch))
        except Exception as e:  # noqa: BLE001 - rate-limit/flake, continua
            print("  trends ERRO %s: %s (use --csv p/ completar)" % (",".join(batch), e))
        time.sleep(delay)
    return scores


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--terms", default="")
    ap.add_argument("--csv", default="")
    ap.add_argument("--top", type=int, default=200)
    ap.add_argument("--delay", type=float, default=5.0)
    ap.add_argument("--out", default="dataset/trends.tsv")
    a = ap.parse_args()

    scores = {}
    if a.csv:
        scores.update(read_csv(a.csv))
        print("csv: %d termos" % len(scores))
    if a.terms:
        terms = [t for t in read_terms(a.terms, a.top) if t not in scores]
        if terms:
            try:
                got = via_pytrends(terms, a.delay)
                scores.update(got)
            except ImportError:
                print("pytrends ausente (pip install pytrends); use --csv. "
                      "%d termos pendentes." % len(terms))
    if not scores:
        print("nada para gravar (passe --csv e/ou --terms).")
        return 1
    today = datetime.date.today().isoformat()
    with open(a.out, "w", encoding="utf-8", newline="\n") as f:
        f.write("# term\tscore0-100\tupdated (gerado por tools/trends_import.py)\n")
        for t in sorted(scores):
            f.write("%s\t%d\t%s\n" % (t, scores[t], today))
    print("trends.tsv: %d termos -> %s" % (len(scores), a.out))
    return 0


if __name__ == "__main__":
    sys.exit(main())
