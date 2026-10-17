# Research — Threads (`threads.rs`)

## Logged-out (via proxy = igual ao checker)
- API `threads.net/api/v1/users/web_profile_info/?username=` (+ headers
  X-IG-App-ID/Accept/XRW): **429 nos dois estados** (taken @zuck e gibberish).
  Mesmo muro do IG — checker atual (só-API) está QUEBRADO via estes
  proxies: tudo vira RateLimited/Unknown.
- `threads.net/@<nick>` → **301 p/ `threads.com/@<nick>`** nos dois estados
  (seguir o redirect; checar destino final).
- `threads.com/@zuck` vs `/@gibberish`: ambos 200 (~276 KB, 5 B de diferença).
  Shell opaco logged-out; HÁ JSON de user no bundle (`follower_count`/
  `textPost`/`username` matched) — comparação por estado PENDENTE
  (extrair e contrastar com fixtures na implementação).

## Gap do checker atual (`threads.rs:27-50`)
API-only + 429 = morto. Reescrever: tentar API (com retry/backoff, pode
funcionar em IP residencial), fallback page `threads.com` com marcadores
a definir por contraste de fixtures; sem sinal → `Unknown` (nunca chute).
Privado/banido: sem exemplos ainda — se operador tiver `@` privado no
Threads, capturar; senão, `Unknown` seguro.
