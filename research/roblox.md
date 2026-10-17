# Research — Roblox (`roblox.rs`)

## API validate (via proxy, sem auth)
`GET auth.roblox.com/v1/usernames/validate?username=<nick>&birthday=...`
| estado | resposta |
|---|---|
| taken (`Roblox`) | 200 `{"code":1,"message":"Username is already in use"}` |
| disponível @gibberish | 200 `{"code":0,"message":"Username is valid"}` |
- Lógica atual CONFIRMADA (code 0=avail, demais=taken).
- Atenção implementação: outros `code` existem (ex. conteúdo impróprio) —
  todos significam indisponível → bucket Taken ok, mas evidência deve
  registrar o código (distinguir taken-real de censurado p/ sniper fino).
