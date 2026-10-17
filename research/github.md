# Research — GitHub (`github.rs`)

## API `api.github.com/users/<nick>` (via proxy)
| estado | resposta |
|---|---|
| taken @torvalds | 200 + `"login":"torvalds"`, `"id":1024025` |
| disponível @gibberish | 404 |
- Lógica atual CONFIRMADA correta (404=avail, 200+id=taken, 403/429=barreira).
- Caveat conhecido (sem exemplar): conta **suspensa** no GitHub retorna 404
  na API também → falso Available potencial. Follow-up: 2º sinal = page
  `github.com/<nick>` ("This account is suspended"?) quando API der 404
  em nick de alto valor — avaliar custo na implementação.
- Validator atual `(1,39, ^[a-z0-9](?:[a-z0-9-]{0,37}[a-z0-9])?$)` ok
  (regras reais GH; sem hífen nas pontas, max 39).
