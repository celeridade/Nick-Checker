# Research — Medium (`medium.rs`)

## Via Brave (IP residencial): marcadores CONFIRMADOS
| estado | `medium.com/@<nick>` |
|---|---|
| taken @medium | `"__typename":"User"` presente, sem miss |
| disponível @gibberish | SEM `__typename:User` + `PAGE NOT FOUND` / "Out of nothing, something." |
- Lógica atual dos marcadores está CERTA em princípio.

## Transporte (problema real)
Via nossa rede (direto e proxies): **403 nos dois estados**, corpo sem
nenhum marcador → hoje tudo cai em `RateLimited` (seguro, mas cego).
403 NÃO contém miss-markers (verificado) → sem risco de falso Avail.
- Fix = rota (residencial/sticky), mantendo os marcadores; 403 → `Unknown`
  explícito (nunca avail).
