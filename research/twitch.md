# Research — Twitch (`twitch.rs`, via twitchtracker)

## Logged-out (via proxy)
| estado | `twitchtracker.com/<nick>` |
|---|---|
| taken (ninja) | 200 (~50 KB) + `Overview</a>` |
| disponível @gibberish | **404** (~8 KB) |
- Lógica atual CONFIRMADA (404=avail, Overview=taken).
- Nota: depende de terceiro (twitchtracker). Banido na Twitch: tracker
  mostra status? — sem exemplar; se aparecer, mapear p/ `Banned`.
  Alternativa canônica futura: Helix API com app token (precisa
  client_id/secret cadastrados — mesmo padrão dos tokens atuais).
