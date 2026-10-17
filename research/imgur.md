# Research — Imgur (`imgur.rs`)

## Transporte (bloqueio atual)
- API `account/v1/accounts/<nick>?client_id=546c25a59c58ad7`:
  via proxy = timeout/000; direto = **429 + "Imgur is temporarily over
  capacity"** (403). API instável daqui em todas as rotas.
- Marcador `"id"` (taken) e 404/400 (avail) NÃO verificados — sem fixture,
  sem mudança (regra ouro).
- Na implementação: retry/backoff + fixtures dos 3 estados; fallback
  `imgur.com/user/<nick>` page se a API seguir instável; `client_id`
  pode estar queimado (avaliar alternar).
