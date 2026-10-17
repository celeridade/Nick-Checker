# Research — Snapchat (`snapchat.rs`)

Só snapshots logged-out via proxy (sem login necessário p/ este vetor).

## Logged-out (via proxy = igual ao checker)
- `/add/<nick>` → **308 p/ `/@<nick>` nos dois estados** (reqwest segue;
  checar URL/status FINAL, não o 308).
- `/@teamsnapchat` (taken): **200 (~120 KB)** + `displayName`/nome, SEM
  `snapchat.com/add/`, sem not-found.
- `/@zzqxkvw123nonexist` (avail): **404 (~6 KB)**.
- Banido/privado: Snapchat perfis são semi-públicos por padrão; estado
  banido não mapeado (sem exemplo) → ambíguo vira `Unknown`.

## Gap do checker atual (`snapchat.rs:27-36`)
404→avail ok. Mas marcador taken `snapchat.com/add/` está MORTO (ausente
na page real) → taken cai em `ambiguous-200` → **RateLimited** em vez de
Taken. Reescrever: 404=avail autoritativo; `displayName`/JSON de
perfil=taken (autoritativo ou 2 fortes); resto `Unknown`.
