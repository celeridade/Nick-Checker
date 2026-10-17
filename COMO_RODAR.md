# COMO RODAR — Site + API + Bot (L96A1)

Guia rápido para subir os 3 componentes: **Site (Next.js)**, **API (Rust/Axum)**, **Bot (Discord/poise)** + **Postgres**.

| Serviço | Porta local | URL | Código |
|---|---|---|---|
| Site (web) | 3000 | http://localhost:3000 | `website/` |
| API (motor Rust) | 8080 | http://localhost:8080 | `crates/l96a1-api/` |
| Postgres | 5432 | `localhost:5432` | `docker-compose.yml` → `db` |
| Bot (Discord) | — (gateway) | via API + Discord | `crates/l96a1-bot/` |

## 0. Pré-requisitos

- **Docker + Docker Compose** (modo recomendado) — *ou* modo local:
  - Node 26 + npm (`node -v`)
  - Rust 1.99 + cargo (`cargo -V`)
  - Postgres 18 local (se for rodar sem Docker)
- Portas livres: `3000`, `8080`, `5432`

## 1. Configurar o `.env` (obrigatório)

```powershell
# na raiz C:\Dev\L96A1
Copy-Item .env.example .env
# gere o segredo de auth:
openssl rand -base64 32
# cole o resultado em AUTH_SECRET no .env
```

Variáveis principais no `.env` (raiz — o compose lê daqui):

```ini
POSTGRES_USER=postgres
POSTGRES_PASSWORD=troque-aqui
POSTGRES_DB=app_db
POSTGRES_PORT=5432
DATABASE_URL=postgresql://postgres:troque-aqui@127.0.0.1:5432/app_db

API_PORT=8080
WEB_PORT=3000

RUST_LOG=info
TIMEOUT_SECS=10
MAX_CONCURRENT=100
THROTTLE_BATCH_SIZE=70
THROTTLE_DELAY_SECS=30

GITHUB_TOKEN=
DISCORD_TOKEN=
WEBHOOK_URL=
WEBHOOK_MODE=batch
BOT_TOKEN=seu-token-do-portal-developer
BOT_GUILD_ID=id-do-servidor
BOT_PREFIX=!

SNIPE_INTERVAL_SECS=300
SNIPE_PLATFORMS=all
WATCHLIST_FILE=watchlist.txt

L96A1_API_URL=http://127.0.0.1:8080
AUTH_SECRET=cole-o-openssl-aqui
```

> ⚠️ O `.env` e `l96a1.toml` atuais contêm **tokens reais** — nunca commitar (já estão no `.gitignore`). Não suba para GitHub.

`website/.env.local` (só para `npm run dev` sem Docker):

```ini
DATABASE_URL=postgresql://postgres:SUA_SENHA@127.0.0.1:5432/app_db
AUTH_SECRET=mesmo-valor-do-.env
L96A1_API_URL=http://127.0.0.1:8080
```

## 2. Opção A — Docker (recomendado: sobe db + api + web)

```powershell
docker compose up --build
# ou em background:
docker compose up --build -d
docker compose logs -f api web db
```

- Site: http://localhost:3000
- API health: http://localhost:8080/health
- Derrubar: `docker compose down` (dados ficam em `pgdata`; `down -v` **apaga o banco**)

O compose injeta automaticamente:

- `api`: `PORT=8080`, `PROXIES_FILE=/data/proxies.txt`, `DATASET_DIR=/data/dataset`, `HISTORY_FILE=/data/history.json`, etc. + volumes `./sites.json`, `./proxies.txt`, `./dataset`, `./history.json`, `./checked.json`, `./watchlist.txt`
- `web`: `DATABASE_URL=postgres://...@db:5432/app_db`, `L96A1_API_URL=http://api:8080`

> Postgres 16→18: se o volume `pgdata` antigo não subir, faça `docker compose down -v` (perde dados locais) ou `pg_upgrade`.

## 3. Opção B — Local sem Docker (dev)

Precisa de um Postgres rodando local (serviço `postgresql-x64-18` ou `docker run postgres:18`).

```powershell
# 1) banco + schema (a partir da raiz, com DATABASE_URL exportado ou via website/.env.local)
npx --prefix website drizzle-kit push

# 2) API Rust (terminal 1) — lê .env da raiz via dotenvy
cargo run -p l96a1-api
# curl http://127.0.0.1:8080/health

# 3) Site Next (terminal 2)
npm --prefix website install
npm --prefix website run dev
# abre http://localhost:3000
```

Build de produção do site:

```powershell
npm --prefix website run build
npm --prefix website run start  # usa .next/standalone/server.js (ver website/Dockerfile)
```

## 4. Site (Next.js) — detalhes

- Dev: `npm run dev --prefix website` (porta 3000). BFF faz proxy para `L96A1_API_URL`.
- Rotas: `/dashboard/search` (SSE único), `/dashboard/bulk`, `/dashboard/dataset`, `/dashboard/snipe`, `/dashboard/proxies`, `/dashboard/bot`, `/dashboard/doctor`, `/go/:platform/:handle` (redirect 302).
- Se o site não acha a API: confira `L96A1_API_URL` (`http://api:8080` no compose, `http://127.0.0.1:8080` local).
- Se login falha: `AUTH_SECRET` vazio — gere com `openssl rand -base64 32`.
- Schema: `website/src/db/schema.ts` (`users`, `searches`, `scans`, `watchlist`, `proxies`, `webhooks`, `bot_state`). Migra com `npx --prefix website drizzle-kit push` (`drizzle.config.ts` lê `DATABASE_URL`).

## 5. API (Rust/Axum) — detalhes

Sobe com `cargo run -p l96a1-api` (`PORT` default 8080, `RUST_LOG=info`). Não precisa de `l96a1.toml` — tudo vem de env (ver `AppConfig::from_env` em `crates/l96a1-api/src/main.rs`).

Teste rápido:

```powershell
curl http://127.0.0.1:8080/health
curl "http://127.0.0.1:8080/v1/platforms"
curl "http://127.0.0.1:8080/v1/search?q=adoecido&platforms=all" -N  # SSE
curl "http://127.0.0.1:8080/v1/dataset/files"
```

Principais endpoints (`main.rs:1986+`):

| Método + rota | Função |
|---|---|
| `GET /health` | healthcheck (compose usa) |
| `GET /v1/platforms`, `/v1/catalogs` | lista 1205 checkers |
| `GET /v1/search?q=nick&platforms=all` | SSE single (eventos `connected/start/result/free/warn/checked/done`) |
| `GET /v1/bulk?users=a,b,c&platforms=all` | SSE multi-nick (máx 100) |
| `POST /v1/scans` `{usernames[], platforms}` | job bulk até 500 (resposta `{jobId}`) |
| `GET /v1/scans/:id`, `POST /v1/scans/:id/cancel` | polling / cancelar |
| `POST /v1/dataset/scan` `{files/filter, smart, limit, platforms}` | job do dataset |
| `GET /v1/proxies`, `POST /v1/proxies/test`, `POST /v1/proxies/prune` | pool + health-check + limpeza |
| `GET /v1/history`, `GET /v1/snipe/status`, `GET/POST /v1/snipe/watchlist`, `POST /v1/snipe/run-once` | histórico + sniper 24h |
| `GET /v1/bot/status`, `POST /v1/bot/setup` | status + setup do servidor |
| `GET /go/:platform/:handle`, `GET /v1/:platform/:handle` | redirect 302 / check JSON único |

Arquivos que a API resolve independente do CWD: `proxies.txt`, `discord_tokens.txt`, `watchlist.txt`, `history.json`, `checked.json`, `dataset/` (candidatos: env → `/data/*` (Docker) → raiz do repo → lado do binário).

## 6. Bot (Discord) — detalhes

O crate `l96a1-bot` é **lib** (sem binário próprio): o setup roda via API/web, o gateway roda como sidecar.

1. Crie o app em https://discord.com/developers/applications → Bot → **copie o token** → `BOT_TOKEN` no `.env`.
2. Ative os intents privilegiados **SERVER MEMBERS** + **MESSAGE CONTENT**.
3. Convide o bot com permissões de gerenciar canais/cargos/webhooks e anote o `BOT_GUILD_ID`.
4. Suba a API + site (Opção A ou B) e faça o setup do servidor de um dos jeitos:
   - UI: http://localhost:3000/dashboard/bot, ou
   - API direta:
     ```powershell
     curl http://127.0.0.1:8080/v1/bot/status
     curl -X POST http://127.0.0.1:8080/v1/bot/setup `
       -H "Content-Type: application/json" `
       -d '{"guild_id": 1179285881320898640}'
     ```
   Isso cria cargos (`owner`, `amigos`, `og` — ver `[bot]` no `l96a1.toml`), canais por plataforma, `#verify/#hits/#log` e webhooks.
5. Gateway 24h (comandos híbridos prefixo `!` + `/slash` via poise 0.7): `help ping verify check status snipe ban kick mute unmute purge slowmode lock unlock userinfo nuke` — mesma `watchlist.txt` do sniper (`crates/l96a1-bot/src/run.rs`).

## 7. Troubleshooting

| Sintoma | Causa provável / fix |
|---|---|
| `web` não conecta na API | `L96A1_API_URL` errado (compose=`http://api:8080`, local=`http://127.0.0.1:8080`) |
| Login do site falha | `AUTH_SECRET` vazio — gere com `openssl rand -base64 32` |
| `drizzle-kit push` falha auth | senha do `DATABASE_URL` ≠ senha real do Postgres local |
| `db` não sobe / healthcheck falha | volume `pgdata` da v16 — `docker compose down -v` e recrie |
| `proxies.txt` somente-leitura no prune | no compose o volume é `:ro` — edite o arquivo local, não via API |
| Porta ocupada | troque `API_PORT`/`WEB_PORT`/`POSTGRES_PORT` no `.env` |
| Discord checker fraco | sem `DISCORD_TOKEN`/`discord_tokens.txt` roda só unauthed (ver log `discord_tokens=N` no boot da API) |

Arquivos de referência: `docker-compose.yml`, `.env.example`, `website/Dockerfile`, `crates/l96a1-api/Dockerfile`, `crates/l96a1-api/src/main.rs`, `crates/l96a1-bot/src/{lib,run,setup}.rs`, `l96a1.toml` (origem dos valores do `.env`).
