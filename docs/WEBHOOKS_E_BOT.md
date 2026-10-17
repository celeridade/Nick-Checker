# Webhooks + Bot (sniper vs normal) + Prefixo `!` vs `/slash` + Purge

Este guia resolve, em ordem: (1) `!purge 100` mudo, (2) onde vai cada webhook,
(3) rate-limit do purge + `purge all`.

## 1. `!x` não responde mas `/x` sim — causa e fix

**Causa raiz (confirmada no código):** o `StandardFramework` da serenity usa o
**nome da função** como nome do comando. Nossas fns chamavam `purge_cmd`,
`ban_cmd`… então o bot registrava `!purge_cmd` — e `!purge` caía no vazio.
Slash funcionava porque é registrado à parte via `CreateCommand`.

**Fix definitivo (poise 0.7):** `crates/l96a1-bot/src/run.rs` foi reescrito sem
`StandardFramework` — cada comando é `#[poise::command(prefix_command, slash_command)]`,
então `!purge` e `/purge` executam o MESMO código (a divergência não pode voltar).
Prefixo configurável (`BOT_PREFIX`), case-insensitive, `ignore_bots`. Cooldowns
poise (`user_cooldown`) no `/check` e `/status`; checks `check_admin`/`check_root`
com resposta `⛔ só admin.` (nunca silêncio).

**Segunda causa clássica (portal):** prefixo exige **Message Content Intent**.

1. https://discord.com/developers/applications → seu app → **Bot**
2. Em **Privileged Gateway Intents**, ative:
   - `SERVER MEMBERS INTENT`
   - `MESSAGE CONTENT INTENT`
3. Salve, **reinicie o bot** (`BOT_TOKEN` + gateway sidecar).
4. Reconvide com scopes `bot` + `applications.commands` e perms:
   `Manage Channels`, `Manage Roles`, `Manage Webhooks`, `Ban Members`,
   `Kick Members`, `Moderate Members (timeout)`, `Manage Messages`,
   `Read Message History`, `Send Messages`.
5. Teste: `!ping` → `⌖ HUD online.` · `!help` → lista · `/ping` → idem.
   Se `!` segue mudo e `/` ok, é 99% o Intent do passo 2.

## 2. Webhooks: sniper vs bot normal (são 2 coisas)

| Uso | Onde configura | O que faz |
|---|---|---|
| **Sniper 24h** (`WEBHOOK_URL`) | `.env` → `WEBHOOK_URL=https://discord.com/api/webhooks/<id>/<token>` · `WEBHOOK_MODE=batch` | O loop da API (`run_snipe_cycle_and_notify`) posta `🎯 liberados` em blocos de 5. Sem gateway envolvido. Teste: dashboard Sniper → **Testar webhook** (`POST /v1/webhooks/test`). |
| **Bot setup** (`#hits` + 1 por plataforma) | Dashboard **Bot** → digite `guild_id` → **Setup**. Ou `POST /v1/bot/setup {"guild_id":…}` | `setup_guild` cria categorias/canais + `l96a1` (por plataforma) e `l96a1-hits` (no `#hits`). Retorna `hits_url` (vira o `WEBHOOK_URL` principal) + `channels{plat:url}`. |
| **Bot normal (gateway)** | `BOT_TOKEN` + `BOT_GUILD_ID` + `BOT_PREFIX=!` (sidecar 24h) | Comandos `!`/`/`. **Não usa webhook** p/ responder — só o sniper usa. |

### Como pegar a URL do webhook (manual, 30s)

1. No Discord: canal `#hits` (ou qualquer canal) → **Editar canal → Integrações → Webhooks → Novo webhook**.
2. Troque nome/avatar se quiser → **Copiar URL do webhook**.
3. Cole no `.env` como `WEBHOOK_URL=…` e reinicie a API. Ou rode o **Setup via web** que cria tudo sozinho.
4. Valide: `curl -X POST $L96A1_API_URL/v1/webhooks/test -H 'Content-Type: application/json' -d '{}'` → `{"sent":true}`.

> `WEBHOOK_MODE=batch` agrupa até 5 liberados por mensagem (igual `l96a1.toml [webhook]`).

## 3. Purge: rate-limit + `all` (Red + discord.py)

Limites do Discord (não tem como fugir, só respeitar):
- **Bulk delete: máx 100/call, só msgs <14 dias.** Se qualquer msg do lote tem 14d+, o bulk inteiro falha → caímos p/ delete 1-a-1.
- **Rate-limit:** ~5 bulks/5s por canal. Por isso **1s entre lotes** + micro-pausas no 1-a-1.

Comandos (prefixo **e** slash):

```
!purge 100            → últimas 100 (bulk quando dá, 1-a-1 quando precisa)
!purge 100 @user      → só desse autor (Red `cleanup user`)
!purge 100 bots       → só bots (Red `cleanup bots`)
!purge all            → VARRE O CANAL ATÉ ESVAZIAR (sem cap; 1s/lote)
/purge n:200          → slash até 1000
/purge all:true       → slash all (com defer, pode levar minutos)
/purge all:true user:@x → all filtrado
```

Só admin (`Administrator` ou dono). `all` pagina com `before` até `batch.is_empty()` — antes o código cortava em 2000 e relia sempre os mesmos 100 (parecia "só 100").

## 4. Comandos novos (portados dos estudos em `estudos/`)

De `Red-DiscordBot/cogs/{cleanup,mod}` + `discord.py/purge` + `CorwinDev` + `serenity/e05`:

- `!slowmode 5` · `/slowmode segundos:5` (`0`=off) — `Manage Channels`.
- `!lock` / `!unlock` · `/lock` `/unlock` — `@everyone` sem/com `Send Messages`.
- `!userinfo [@user]` · `/userinfo` — id, criação, entrada, cargos.
- `!purge … bots` — filtro de bots.

## 5. Referências clonadas em `estudos/` (depth-1)

- `Red-DiscordBot` — cogs `admin/cleanup/mod/mutes/permissions`: modos de cleanup, confirmação, cooldowns, help paginado.
- `discord.py` — `TextChannel.purge(limit, check, bulk)`: bulk<14d senão single; `commands.Bot(command_prefix, intents.message_content)`, hybrid slash+prefix.
- `discord.js` — `SlashCommandBuilder` + `PermissionFlagsBits` + deferred replies p/ ops longas.
- `CorwinDev/Discord-Bot` — `config.json`, handlers por evento, anti-raid/tickets (inspiração p/ `lock/slowmode`).
- `serenity` — `e05_command_framework`: `#[command("nome")]`, `#[group]`, buckets/cooldown, `e14_slash_commands`, `e18_webhook`.

Stack: **100% Rust** (serenity 0.12 + poise 0.7 híbrido). Não compensa adicionar Python/JS ao runtime — o port foi de **padrões**, não de linguagem. A migração `StandardFramework` (depreciado, removido) → `poise` (1 definição = `!x` + `/x`) foi concluída em `crates/l96a1-bot/src/run.rs`.
