# Research — Spotify (`spotify.rs`)

## Logged-out (via proxy): SEM SINAL
- `open.spotify.com/user/<nick>`: 200 nos dois estados, shells idênticos
  (~164 KB), title genérico "Spotify - Web Player", SEM `spotify:user:`,
  SEM JSON de usuário. Marcadores taken atuais MORTOS.
- oEmbed p/ perfil de usuário: 400 nos dois (não suportado; só tracks?).
- Hoje: ocupado = `ambiguous-200` → RateLimited (doença conhecida).

## Caminho real (pendente)
- **Web API com app token** (`client_credentials`, sem login de usuário):
  precisa `client_id`/`client_secret` cadastrados (mesmo padrão
  `DISCORD_TOKEN`/`GITHUB_TOKEN` do projeto — operador pode prover).
  Endpoint de lookup por username a definir na implementação
  (users/{id} usa ID base62; pesquisar equivalente p/ nome).
- Até lá: sem chute (`Unknown` quando sem sinal).
