# Discord sniper 24h — research (Fase 3 + revisão das sources)

## O que cada source rendeu (aplicado)

- **discord/discord-api-docs#rate-limits**: buckets por `X-RateLimit-Bucket`,
  global 50/s por bot (sem auth = por IP!), `Retry-After`/`retry_after`
  mandam no retry, `X-RateLimit-Scope` (`user`/`global`/`shared`), e o limite
  de inválidas (Cloudflare ban: 10k/10min de 401/403/429 — `shared` não conta).
  → checker agora parseia header + corpo e carimba o escopo na evidência.
- **discordjs/discord.js (packages/rest)**: padrão RequestHandler — fila por
  bucket, retry automático honrando `retry_after`, global 50/s, sem hardcode
  de limites (lê headers). → nosso `min_interval_ms` + AIMD + throttle seguem
  a mesma filosofia; nunca hardcodar cota, sempre ler a resposta.
- **Discord-Datamining/Discord-Datamining** (~80k commits, diff de builds
  Canary): fonte p/ descobrir mudanças de endpoints/fluxos (ex: pomelo).
  Uso aqui: monitorar diffs quando um método parar de responder.
- **Dziurwa14** → **discord-userdoccers** (docs.discord.food, User API não
  oficial): confirmou `GET username-suggestions-unauthed?global_name=` e
  `POST username-attempt-unauthed` com `{taken}`, fluxo de register, login,
  fingerprints e sessões. → suggestions virou o 3º método do checker.
- **AdamBankz/discord-reversed + gists (g1ver, Infinitay, hackermondev)**:
  semântica do suggestions (eco = livre), constantes
  POMELO_*_UNAUTHED, e o aviso crítico: pomelo-attempt some se a conta
  travar username novo. → documentado + implementado.
- **suenerve/Discord-Username-Checker**: pool de tokens com rotação em 429
  (`sleep(retry_after)` se token único). → `discord_tokens.txt` + rotação.
- **Red-DiscordBot / python-discord**: tiers owner/admin/mod, intents
  privilegiados p/ bots pequenos (<100 guilds, sem verificação), Message
  Content p/ comandos de texto, mod-log e self-roles. → nosso `!snipe`
  exige Manage Server; intents documentados no setup.
- **ItzSudhan/Discord-MusicBot, CorwinDev/Discord-Bot**: referência de
  comandos/setup; nada específico p/ checker — sem ação.

# Discord sniper 24h — research (Fase 3, original)

Objetivo: checar 1 ou N usernames continuamente (ex: a cada 5min) em todas
as plataformas, com proxies e rate-limit respeitados, e avisar no Discord
quando liberar. Sem milhares de contas/proxies.

## Endpoints de check de username (Discord)

1. **unauthed** `POST /api/v9/unique-username/username-attempt-unauthed`
   `{"username":"<nick>"}` → 200 `{"taken":true|false}`.
   Sem token, barato, mas rate-limit agressivo por rota/IP (observado 429
   com `retry_after` de ~1176s em proxy queimado). Exige rotação + backoff,
   e 429 SEMPRE como barreira/Unknown. Ver `research/discord.md`.
2. **pomelo authed** `@me/pomelo-attempt` (user token do operador).
   Maior confiança + maior cota. Perfil atual no L96A1: 4 conc, ≥500ms.
   Nunca martelar a conta real; poucos checks espaçados = sem rate.
3. **register field** (`discord.com/register`, `input[name=username]`):
   mensagens live "available"/"unavailable", mas client-side via bundle —
   serve como validação manual, não como API.
4. **modal logado** (Settings → Username Edit): vai pelo gateway WS ou
   bundle, sem fetch rastreável; equivale ao pomelo authed.

## Como os servidores de snipe 24h fazem (mining público)

Padrão observado em bots open-source de data-mining/sniping:

- **Sessões distribuídas, não milhares de proxies**: N sessões (tokens
  próprios/alts) cada uma com cadência baixa (1 check / 30–120s por método)
  + jitter. O agregado dá cobertura 24h sem estourar nenhuma cota.
- **Rotação de métodos**: unauthed → pomelo → pausa. Quando um método dá
  429, só ele entra em cooldown (igual ao isolamento de cooldown do
  `l96a1-proxy`: proxy entra em cooldown só para aquela plataforma).
- **Backoff em `retry_after`**: respeita o header, nunca retry imediato.
- **Alvos priorizados por escassez**: watchlist pequena (1–50 nicks) em vez
  de brute force; cada ciclo é barato.
- **Persistência de estado**: guarda último status por (nick, plataforma)
  e só notifica na transição → `Available` (evita spam e re-check inútil).

É exatamente o que o `l9 snipe` implementa: usa o motor de evidências,
proxies e throttle já existentes, dorme `snipe.interval_secs` entre ciclos
e só notifica transição para Available.

## Self-bot (conta de usuário como bot)

- Tecnicamente: mesmos endpoints acima com `Authorization: <user token>`.
- Risco: contra os ToS do Discord; detecção por padrões (mesmo IP,
  cadência mecânica, fingerprint). Mitigação: conta separada (nunca a
  principal), cadência humana + jitter, sessões poucas, uso privado.
- Decisão do operador: aceito (uso pessoal, conta separada). O bot oficial
  (gateway + captcha + setup) usa **bot token** normal; self-bot fica
  restrito ao método pomelo do sniper, configurável via `discord_token`.

## Estrutura do servidor (bot `setup`)

- Categorias por grupo: SOCIAL, DEV, GAMING, STREAMING, MUSIC, COMMUNITY.
- 1 canal texto por plataforma (29) + `#verify` + `#hits` + `#log`.
- Tudo privado: `@everyone` sem `ViewChannel`; role `verificado` com acesso.
- 1 webhook por canal de plataforma → `[snipe.channels]` no `l96a1.toml`;
  webhook do `#hits` vira `webhook.url` principal.
- Verificação: captcha em imagem (5 chars, fonte bitmap própria + ruído),
  resposta por mensagem em 120s; acerto = role `verificado`.
