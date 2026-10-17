# Research — YouTube (`youtube.rs`)

## Logged-out (via proxy)
| estado | `youtube.com/@<nick>` |
|---|---|
| taken @mkbhd | 200 (~2,5 MB) + `"channelId"`, sem "does not exist" |
| disponível @gibberish | **404** (755 B) |
- Lógica atual CONFIRMADA (404=avail, channelId=taken).
- Caveat: canal **terminado** pelo YouTube costuma virar 404/"does not
  exist" → falso Available potencial. Follow-up: 2º sinal p/ 404
  (ex.: oEmbed `youtube.com/oembed?url=...` — testar na implementação;
  terminado x livre podem diferir lá).

---

# Sessão 2026-10-10 — falsos positivos 4L repeaters + ground truth de claim

Nicks: `aamm`, `abba`, `akka` (+ `amma`, `@mkbhd`, gibberish controle).
Lista: `dataset/patterns/repetition/letters/4C/4c-two-pairs-letters-repetition/a.txt`.

## Causa raiz: colisão de namespace legado, não bug de threshold
| nick | `/@nick` | `/c/nick/about` | `/user/nick/about` | claim (Studio) |
|---|---|---|---|---|
| `aamm` | 404 755B | 404 | **200 "Andrew Amm"** | **UNAVAILABLE** (`@aamm-f3n7d` sugerido) |
| `abba` | 404 755B | 404 | **200 "ABBA"** (banda) | **UNAVAILABLE** |
| `akka` | 404 755B | 404 | **200 "Hiideto"** | **UNAVAILABLE** (`@akka-z7v` sugerido) |
| `amma` | — (redirect p/ `/amma`) | — | — | — (canal real "Amma", canon `/@Amma` — match case-insensitive!) |
| gibberish | 404 755B | 404 | 404 | **OK** (`CHANNEL_HANDLE_VALIDATION_RESULT_OK`) |

Leitura: `@` livre + `/user/` ocupado = **claim bloqueado**. O `taken` que
o usuário via vinha do checker legado `wm:youtube-user` — sinal CORRETO p/
claimability, nome confuso no dashboard. E o checker de `@` dizendo "avail"
p/ esses nicks seria o erro real p/ sniper (livre no HTML, indisponível no claim).

## Ground truth de claim (CDP Studio, só leitura — nada salvo)
- Endpoint: `POST /youtubei/v1/channel_edit/validate_channel_handle?alt=json`.
- Taxonomia: `CHANNEL_HANDLE_VALIDATION_RESULT_OK` (livre, mostra a URL
  `youtube.com/@nick`) vs `..._UNAVAILABLE` (+ `availableVariations`, ex.
  `@akka-z7v`). Resposta ~4,2KB.
- Regras visíveis: handle único `@`, **máx 2 trocas / 14 dias, hold do
  handle anterior por 14 dias**. 4 checks espaçados sem throttle — mas é
  endpoint logado sensível: NÃO usar em bulk, só validação manual de alto valor.
- Campo restaurado p/ valor original (`xqda`), nenhum Publish clicado.

## Budget público (bulk)
- 12x sequencial + 66x paralelo diretos em `aamm/abba/akka`: **tudo 404
  limpo, zero 429**. Sem throttle nessa escala.
- Rajada não reproduziu FP: `decide()` atual nunca marca taken num 404
  (cai em `Unknown`). FP da classe "channelId alheio" é latente: página 200
  de parede/busca/consentimento com `channelId` estranho marcaria taken —
  corrigido exigindo `canonicalBaseUrl` do handle exato (ver `youtube.rs`).
- Nota: campo real de handle é `canonicalBaseUrl":"/@nick` (não existe
  `"channelHandle"` no HTML); `channelUrl`/`vanityChannelUrl` aparecem 1x.

## Regra de decisão (pós-patch)
Taken SOMENTE com `200` + `canonicalBaseUrl:"/@<nick>"` exato
(case-insensitive) + `channelId` presente. `200` sem canon exato → `Unknown`
(nunca taken). `404` + título/"does not exist" → `Available`. `@` livre +
legado ocupado = claim bloqueado — reportar namespaces separados; agregado
honesto p/ sniper é "não reivindicável", não "livre".
