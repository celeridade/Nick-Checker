# Nick Checker (L96A1)

Ferramenta para consultar a presença de um nome de usuário em várias plataformas. O workspace combina um motor de verificação em Rust, uma API HTTP, um painel Next.js, PostgreSQL e integrações opcionais com Discord.

Use o sistema somente em serviços para os quais você tem autorização. Respeite os termos de uso, `robots.txt`, limites de requisição e legislação aplicável. O projeto não deve ser usado para contornar CAPTCHA ou bloqueios.

## Como funciona

O painel envia a consulta para `l96a1-api`. A API resolve os checkers do catálogo, faz as consultas concorrentes e transmite os resultados por SSE (`connected`, `start`, `result`, `warn`, `checked`, `done`). Histórico, jobs, watchlist e configurações são persistidos em PostgreSQL e arquivos de estado.

| Componente | Diretório | Função |
| --- | --- | --- |
| API | `crates/l96a1-api` | HTTP, SSE, jobs, histórico, proxies e sniper |
| Motor/checkers | `crates/l96a1-core`, `crates/l96a1-checkers` | Catálogo e lógica dos serviços |
| Proxies | `crates/l96a1-proxy` | Rotação, health-check, cooldown e prune |
| Bot | `crates/l96a1-bot` | Setup e comandos Discord opcionais |
| Web | `website` | Painel Next.js e BFF |
| Dados | `dataset`, `sites.json`, `wmn.json` | Listas e metadados |

## Requisitos

Para Docker: Docker Desktop com Compose e portas livres `3000`, `8080` e `5432`. Para execução local: Rust 1.99, Node.js 26/npm, PostgreSQL 18 e OpenSSL para gerar o segredo de sessão.

## Configuração

```powershell
Copy-Item .env.example .env
openssl rand -base64 32
```

Coloque o resultado em `AUTH_SECRET`. Nunca versione `.env`, tokens, cookies ou arquivos de estado.

Variáveis principais:

| Variável | Padrão | Uso |
| --- | --- | --- |
| `DATABASE_URL` | PostgreSQL local | Banco do painel |
| `AUTH_SECRET` | vazio | Obrigatório para login |
| `API_PORT` / `WEB_PORT` | `8080` / `3000` | Portas publicadas |
| `L96A1_API_URL` | `http://127.0.0.1:8080` | API vista pelo Next.js |
| `TIMEOUT_SECS` | `10` | Timeout por consulta |
| `MAX_CONCURRENT` | `100` | Concorrência máxima |
| `THROTTLE_BATCH_SIZE` / `THROTTLE_DELAY_SECS` | `0` / `30` | Controle de lotes |
| `HISTORY_FILE` / `CHECKED_FILE` | `history.json` / `checked.json` | Estado persistente |
| `SNIPE_INTERVAL_SECS` | `300` | Intervalo do sniper |
| `BOT_TOKEN` / `BOT_GUILD_ID` | vazio | Discord opcional |

Tokens de API, sessões TikTok e webhooks são opcionais. Sem credenciais, alguns checkers funcionam somente no fluxo público.

## Proxies

Crie `proxies.txt` na raiz, com uma entrada por linha. Linhas vazias e iniciadas por `#` são ignoradas:

```text
# http, socks4 e socks5 são aceitos
http://usuario:senha@203.0.113.10:8080
socks5://203.0.113.11:1080
203.0.113.12:3128
```

Uma entrada sem esquema recebe `http://`. O pool usa round-robin entre proxies saudáveis, afinidade para plataformas que precisam de sessão e cooldown por par `(proxy, plataforma)` quando há rate-limit. Falhas repetidas fazem o proxy sair do pool; estar listado não garante que ele esteja acessível.

No Compose, `proxies.txt` é montado como somente leitura. Edite o arquivo no host para alterar o pool e use os endpoints/painel para testar ou podar proxies. Avalie a privacidade e a autorização dos proxies antes de usá-los.

## Executar com Docker (recomendado)

```powershell
Copy-Item .env.example .env
docker compose up --build
```

Em segundo plano:

```powershell
docker compose up --build -d
docker compose logs -f api web db
```

Abra <http://localhost:3000>. A API fica em <http://localhost:8080> e o health-check em <http://localhost:8080/health>. Para parar, `docker compose down`. `docker compose down -v` também apaga o volume local do PostgreSQL.

O Compose sobe `db`, `api` e `web`, injeta `L96A1_API_URL=http://api:8080` no painel e monta catálogo, dataset, proxies e estado em `/data`.

## Executar sem Docker

Com PostgreSQL ativo, configure `DATABASE_URL`, `AUTH_SECRET` e `L96A1_API_URL` (em `website/.env.local` ou no ambiente):

```powershell
npm --prefix website install
npx --prefix website drizzle-kit push
```

Terminal 1:

```powershell
cargo run -p l96a1-api
```

Terminal 2:

```powershell
npm --prefix website run dev
```

O painel abre em <http://localhost:3000>. Para produção:

```powershell
npm --prefix website run build
npm --prefix website run start
```

## API e painel

| Rota | Função |
| --- | --- |
| `GET /health` | Health-check |
| `GET /v1/platforms` | Lista plataformas |
| `GET /v1/search?q=nome&platforms=all` | Busca SSE |
| `GET /v1/bulk?users=a,b&platforms=all` | Busca SSE em lote |
| `POST /v1/scans` | Cria job |
| `GET /v1/scans/:id` | Consulta job |
| `POST /v1/scans/:id/cancel` | Cancela job |
| `GET /v1/proxies` | Estado do pool |
| `POST /v1/proxies/test` | Testa proxies |
| `POST /v1/proxies/prune` | Remove proxies degradados |
| `GET /v1/history` | Histórico |
| `GET /v1/snipe/status` | Status do sniper |
| `GET /v1/bot/status` | Status do Discord |

O painel tem busca, lote, dataset, sniper, proxies, bot e diagnóstico em `/dashboard/doctor`.

## Discord e sniper

São recursos opcionais. Crie o bot no [Discord Developer Portal](https://discord.com/developers/applications), configure `BOT_TOKEN` e `BOT_GUILD_ID`, habilite os intents necessários e faça o setup em `/dashboard/bot` ou pela API. O bot usa a mesma watchlist do sniper (`WATCHLIST_FILE`). Ajuste `SNIPE_INTERVAL_SECS` e `SNIPE_PLATFORMS` conforme os limites das plataformas.

## Acessibilidade, disponibilidade e segurança operacional

- Acesse o painel por navegador em `localhost` ou atrás de reverse proxy HTTPS configurado pelo operador.
- Em produção, restrinja CORS, autenticação, portas e exposição pública da API.
- `Unknown`, timeout ou erro não significa que o nome está disponível; o serviço pode estar bloqueado, indisponível ou exigir autenticação.
- Prefira concorrência e intervalos conservadores para evitar rate-limit.
- Não coloque credenciais ou cookies em logs, issues, imagens, `proxies.txt` ou no repositório.

## Troubleshooting

| Sintoma | Verificação |
| --- | --- |
| Web não encontra API | Compose usa `http://api:8080`; local usa `http://127.0.0.1:8080`. |
| Login falha | Confira `AUTH_SECRET` e use o mesmo valor no web e na API. |
| PostgreSQL falha | Confira senha/host em `DATABASE_URL` e o volume existente. |
| Nenhum proxy | Confira `PROXIES_FILE`, `proxies.txt` e o formato da URL. |
| Muitos timeouts | Reduza `MAX_CONCURRENT`, aumente `TIMEOUT_SECS` e teste o pool. |
| `drizzle-kit push` falha | Confirme que o PostgreSQL está acessível e a URL aponta para o banco correto. |
| Discord sem resultados | Confira token, intents, guild ID e permissões. |

## Testes

```powershell
cargo test --workspace
npm --prefix website run lint
npm --prefix website run typecheck
```

## Licença

Consulte a licença publicada no repositório e as regras de cada plataforma consultada antes de colocar o sistema em produção.
