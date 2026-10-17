#!/usr/bin/env python3
"""Auditoria fase 2: para cada suspect_taken, testa gibberish novo + sample
known via /v1/{key}/{nick}. Classifica: broken (taken nos 2), collision
(taken só no sample), inverted (available nos 2), walled/error.
Uso: python tools/audit_phase2.py results/audit-<ts>.json
"""
import json
import re
import sys
import time
import urllib.parse
import urllib.request

API = "http://127.0.0.1:8080"
GIB2 = "plmxvq904173"


def get_json(path):
    with urllib.request.urlopen(API + path, timeout=40) as r:
        return json.loads(r.read().decode())


def single(key, nick):
    try:
        return get_json(f"/v1/{urllib.parse.quote(key, safe='')}/{urllib.parse.quote(nick)}")
    except Exception as e:
        return {"_error": str(e)[:120]}


def load_samples():
    samples = {}
    txt = open("crates/l96a1-checkers/tests/generated_sites.rs", encoding="utf-8").read()
    for m in re.finditer(r'\("([^"]+)", "([^"]+)"\),', txt):
        samples[m.group(1)] = m.group(2)
    return samples


def main():
    audit = json.load(open(sys.argv[1], encoding="utf-8"))
    sites = audit["sites"]
    samples = load_samples()
    suspects = [k for k, v in sites.items() if v["status"] == "suspect_taken"]
    print(f"suspects: {len(suspects)}", flush=True)
    out = {}
    for i, key in enumerate(suspects):
        sample = samples.get(key)
        r_gib = single(key, GIB2)
        r_smp = single(key, sample) if sample else {"_nosample": True}
        g = r_gib.get("result", "?")
        s = r_smp.get("result", "?")
        if g == "taken" and s == "taken":
            verdict = "broken? (taken nos 2)"
        elif g == "available" and s == "taken":
            verdict = "ok (colisao no gib1)"
        elif g == "available" and s == "available":
            verdict = "inverted? (available ate no known)"
        else:
            verdict = f"walled/error (gib={g} sample={s})"
        out[key] = {"gib2": g, "sample": sample, "sample_result": s,
                    "verdict": verdict,
                    "ev_gib": str(r_gib.get("tag", r_gib.get("evidence", "")))[:100]}
        print(f"[{i+1}/{len(suspects)}] {key}: {verdict}", flush=True)
        time.sleep(0.4)
    json.dump(out, open("results/audit-phase2.json", "w", encoding="utf-8"),
              ensure_ascii=False, indent=1)
    from collections import Counter
    print(Counter(v["verdict"].split(" (")[0] for v in out.values()))
    print("salvo: results/audit-phase2.json")


if __name__ == "__main__":
    sys.exit(main())
