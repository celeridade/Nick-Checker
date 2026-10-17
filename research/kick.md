# Research — Kick (`kick.rs`)

## API `kick.com/api/v2/channels/<nick>` (via proxy, sem CF!)
| estado | resposta |
|---|---|
| taken (xqc) | 200 JSON + `slug`+`id` (+ `is_banned:false`, `verified`, counts) |
| disponível @gibberish | **404** (65 B, checar texto "Channel not found" em fixture) |
- Lógica atual CONFIRMADA e boa (slug match + 404+payload).
- Gancho `Banned`: campo **`is_banned`** existe no JSON — banido exemplar
  pendente; quando aparecer, `is_banned:true` → `Banned` autoritativo.
- Sem WAF nesta rota (direto funciona); manter `force_direct`, sem proxy.
