#!/usr/bin/env python3
"""Auditoria fase 1: 2 nicks gibberish x todas as plataformas via job.
Classifica cada key: healthy / suspect_taken / walled / dead / skipped.
Uso: python tools/audit_sites.py  (engine em localhost:8080)
Saida: results/audit-<ts>.json
"""
import json
import sys
import time
import urllib.request

API = "http://127.0.0.1:8080"
GIBBERISH = ["zxqkvn482917", "qwrtmpl739051"]


def post(path, body):
    data = json.dumps(body).encode()
    req = urllib.request.Request(API + path, data=data,
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=60) as r:
        return json.loads(r.read().decode())


def get(path):
    with urllib.request.urlopen(API + path, timeout=60) as r:
        return json.loads(r.read().decode())


def main():
    print("criando job: 2 gibberish x all ...", flush=True)
    job = post("/v1/scans", {"usernames": GIBBERISH, "platforms": "all",
                             "source": "audit-gibberish"})
    jid = job["jobId"]
    print("job:", jid, "total:", job.get("total"), flush=True)
    while True:
        time.sleep(10)
        j = get(f"/v1/scans/{jid}")
        print(f"  {j.get('status')} checked={j.get('checked')}/{j.get('total')} "
              f"hits={j.get('hits')} warns={len(j.get('warnings') or [])}", flush=True)
        if j.get("status") in ("done", "error", "cancelled"):
            break
    plats = get("/v1/platforms")
    keys = [p["key"] for p in plats.get("platforms", [])]
    name_to_key = {p["name"]: p["key"] for p in plats.get("platforms", [])}
    per = {k: {"status": "skipped", "detail": ""} for k in keys}

    def resolve(ref):
        # results/frees trazem NAME ("Discord"); warns trazem key
        if ref in per:
            return ref
        return name_to_key.get(ref, "")

    for r in j.get("results") or []:
        k = resolve(r.get("platform", ""))
        if not k:
            continue
        # hit (taken) num gibberish = suspeito (falso taken)
        per[k] = {"status": "suspect_taken", "detail": "taken em gibberish"}
    for f in j.get("available") or []:
        k = resolve(f.get("platform", f.get("key", "")))
        if not k:
            continue
        per[k] = {"status": "healthy", "detail": "available em gibberish"}
    for w in j.get("warnings") or []:
        k = w.get("key", "")
        if not k:
            continue
        reason = w.get("reason", "")
        ev = w.get("evidence", "")
        if reason == "rate":
            per[k] = {"status": "walled", "detail": ev[:160]}
        elif reason == "error":
            per[k] = {"status": "dead_error", "detail": ev[:160]}
        else:
            per[k] = {"status": "dead_unknown", "detail": ev[:160]}
    # nicks que o validator rejeitou somem das 3 listas -> skipped permanece
    out = {"meta": {"job": jid, "nicks": GIBBERISH,
                    "checked": j.get("checked"), "total": j.get("total")},
           "sites": per}
    ts = time.strftime("%Y%m%d-%H%M%S")
    path = f"results/audit-{ts}.json"
    json.dump(out, open(path, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
    from collections import Counter
    print(Counter(v["status"] for v in per.values()))
    print("salvo:", path)


if __name__ == "__main__":
    sys.exit(main())
