# Research — GitLab (`gitlab.rs`)

## API `gitlab.com/api/v4/users?username=<nick>` (via proxy)
| estado | resposta |
|---|---|
| taken `root` | 200 `[{id, username, state:"active", locked:false, ...}]` |
| disponível @gibberish | 200 `[]` |
- Lógica atual CONFIRMADA correta (`[]`=avail, `id`=taken).
- Gancho `Banned`: o array traz **`state`** (`active` visto; banido deve vir
  `banned`/`blocked` + `locked:true`) — sem exemplar confirmado; na
  implementação, mapear `state != active` → `Banned` (validar com fixture
  real ou conta de teste).
