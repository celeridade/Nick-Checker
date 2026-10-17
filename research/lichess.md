# Research — Lichess (`lichess.rs`)

## API `lichess.org/api/user/<nick>`
| rota | taken | disponível |
|---|---|---|
| via proxy | 000 (timeout/egress bloqueado) | 000 |
| direto | 200 (1,7 KB) | **404** (21 B) |
- Lógica CONFIRMADA (200+id=taken, 404=avail); rota tem que ser DIRETA
  (batendo com `force_direct` atual — manter, sem proxy).
- Follow-up `Banned`: Lichess marca engine/cheaters publicamente — ver se
  a API expõe flag (`disabled`/`engine`?) em conta fechada/banida;
  precisa exemplar (sem bunny agora).
