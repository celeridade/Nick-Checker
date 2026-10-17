# Research — Bluesky (`bluesky.rs`)

API pública sem auth (`public.api.bsky.app`), via proxy.

## Logged-out = o que o checker vê
| estado | `getProfile?actor=` |
|---|---|
| taken `bsky.app` | 200 (~1 KB) + `"did"` |
| disponível `x.bsky.social` (forma completa!) | 400 + `InvalidRequest` + **"Profile not found"** |
| handle cru sem sufixo (`zzqxkvw123…`) | 400 + `InvalidRequest` + "Invalid AT identifier" (**não** é "not found" — cuidado) |
| `handle.invalid` | 200 taken (conta real GABALabs! nunca assumir nada por nome) |
- Deactivated/suspenso: erro canônico `AccountDeactivated` (400) — sem
  exemplar confirmado; marcar como candidato a `Banned`, validar em fixture.
- Cuidado: checar `"not found"` como substring casa com "Profile not found" ✓
  mas a forma do actor importa: sempre normalizar `nick` → `nick.bsky.social`
  quando sem ponto (o validator já aceita `[a-z0-9.-]`).

## Gap do checker atual (`bluesky.rs:27-39`)
Estrutura certa; faltam: (1) normalização do actor p/ checagem de
disponível (se checar o cru, 400 "Invalid AT identifier" cai em `Unknown`
em vez de `Available`); (2) ramo `AccountDeactivated/*suspend*` → `Banned`;
(3) fixtures dos 3+1 estados.
