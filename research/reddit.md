# Research — Reddit (`reddit.rs`)

## Transporte (bloqueio atual)
- `www.reddit.com/user/<nick>/about.json` via proxy: **403 nos dois
  estados** (190 KB, block page), independente de UA.
- `old.reddit.com/.../about.json`: taken devolve HTML (não JSON), avail 403.
- Conclusão: rota atual cega; marcadores (`"id"`, 404) NÃO verificados
  logged-out daqui.

## Caminhos p/ implementação (sem chute)
1. Rota residencial/sticky + revalidar `about.json` com fixtures.
2. **OAuth app-only** (sem login de usuário): registrar app, `client_credentials`
   → `oauth.reddit.com/user/<nick>/about.json` — canônico e estável.
3. Suspenso: `about.json` de suspenso costuma 404 → mesmo caveat do GitHub
   (falso Avail); page tem "account suspended"? — validar com exemplar.
- Até ter rota+fixtures: manter lógica, sem mudança de classificação.
