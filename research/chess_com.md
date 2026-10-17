# Research — Chess.com (`chess_com.rs`)

## API `api.chess.com/pub/player/<nick>`
| rota | taken (hikaru) | disponível |
|---|---|---|
| via proxy | 403 CF (~5,6 KB) | 403 CF |
| direto | 200 + `"username"` + perfil rico | **404** (61 B) |
- Lógica CONFIRMADA (200+username=taken, 404=avail); rota DIRETA
  obrigatória (proxies tomam CF) — igual Lichess.
- Bônus sniper: payload traz `title` (GM), `followers`, `status`,
  `verified` — útil p/ scoring de OG (não só booleano).
- Regra edge-char atual (início/fim alfanumérico) mantida.
