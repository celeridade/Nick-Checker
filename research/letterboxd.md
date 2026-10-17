# Research — Letterboxd (`letterboxd.rs`)

## Via Brave (IP residencial)
| estado | `letterboxd.com/<nick>/` |
|---|---|
| taken @matt | title "?Matt Lynch's profile Letterboxd" + `profile-header` |
| disponível @gibberish | title "Letterboxd - Not Found", sem marcadores |
| `@letterboxd` | Not Found (NÃO existe — nunca assumir por nome!) |
- `person-poster` ausente até no taken (só em páginas de filme?) — validar
  em fixtures; `profile-header` + title `'s profile` como sinais taken.

## Via signup: endpoint de check (RESOLVE o muro)
- `GET https://letterboxd.com/s/checkusername?q=<nick>` (+ XRW).
  Wire decodificado do `bxd.initUsernameChecker` (fonte lida na página).
- `data.result`: `AVAILABLE` / `TAKEN` / `INVALID` ("Use a-z, 0-9 or _
  only" — charset confirmado, SEM ponto) / `ILLEGAL` ("Unavailable",
  bloqueadas → tratar como taken) / `TOO_SHORT` (len<2; "a" testado) /
  `TOO_LONG` / `MISSING` (parâmetro ausente) / `YOU` (próprio).
- UI: "Available" / "Taken" / "Too short" / "Too long!".
- `maxlength="15"` no input → validator max 15 (atual é 30!).
- Via proxy: CF challenge (muro) — validar rota na implementação
  (residencial/sticky); challenge → `Unknown`, nunca avail.
- "ab" (2 chars) → TAKEN; `reserved` → TAKEN; `handle.invalid` n/a aqui.

## Transporte (pages)
Via nossa rede: **403 nos dois estados** (mesmo muro de Medium/Tumblr).
Endpoint acima é o caminho; pages viram 2º sinal se a rota melhorar.

---

# Sessão 2026-10-10 — embed de registro + muro CF (CDP + proxy)

## Embed / taxonomia (via CDP, IP residencial — 8/8 200)
Homepage carrega `bxd.initUsernameChecker` (sem input de signup visível;
o wire é o mesmo `/s/checkusername?q=` + `X-Requested-With`):

| nick | resultado | nota |
|---|---|---|
| `matt` | `TAKEN` | |
| `letterboxd` | `ILLEGAL` | marca própria reservada (perfil não existe!) |
| `reserved` | `TAKEN` | |
| `xkqzptwjmv12` / `premeditar` | `AVAILABLE` | |
| `ab` | `TAKEN` | 2 chars ocupados são comuns |
| `a` | `TOO_SHORT` | |
| `@"@"invalid!` | `INVALID` | |
| resposta | `{"result":true,"csrf":"…","data":{"result":"…"}}` | `csrf` fresco por chamada |

Taxonomia do checker (`AVAILABLE/TAKEN/ILLEGAL→taken/INVALID/TOO_SHORT/
TOO_LONG/MISSING→Unknown`) CONFIRMADA sem mudanças. `YOU` só aparece
logado; logged-out nunca retorna.

## Muro Cloudflare (só datacenter — confirmado nas duas stacks)
- `requests` via `proxies.txt`: **403 `Just a moment...`**.
- `curl_cffi impersonate=chrome` via proxy: **403 igual** — o bloqueio é por
  reputação do ASN/IP, não só fingerprint TLS. Residencial é requisito.
- Rajada residencial (CDP, 15 nicks): **15/15 200, zero throttle** — endpoint
  leve (~200B), ótimo p/ bulk *se* a rota for residencial/sticky.

## Implicação p/ o checker
Lógica intacta; transporte é o gargalo: datacenter = `Unknown` honesto
(já é o comportamento). Follow-ups: (a) TLS browser-like no Rust (`rquest`)
+ proxy **residencial** sticky p/ este host; (b) nunca tratar 403/challenge
como avail (já garantido em teste).
