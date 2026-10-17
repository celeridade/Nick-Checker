# Research — guns.lol (`guns_lol.rs`)

## Logged-out (via proxy)
- `/<nick>` → **307 p/ si mesmo** nos dois estados (stub de 41/55 B, só eco
  da URL). reqwest SEM cookie-jar entra em loop de self-redirect → erro.
- COM cookie-jar: taken segue 2 redirects → 200 (~40 KB) com perfil
  (avatar/links), SEM not-found; avail → 200 (~22 KB) COM not-found
  (`not found` / `everything you want` / `bulunamad` — lista atual ok).
- Erro de forma atual: sem jar, tudo = `Error`; com jar mas sem marcadores
  taken, ocupado = `RateLimited`.

## Gap do checker atual (`guns_lol.rs:26-41`)
(1) habilitar `cookie_store` no client (cross-cutting: avaliar impacto
global no P0 — útil p/ vários sites, isolar se arriscado);
(2) taken = foto/links de perfil + ausência de not-found (2+ sinais);
(3) manter avail-list atual; regra premium-alias mantida (barreira, ok).
