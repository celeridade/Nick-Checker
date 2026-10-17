# Research — Steam (`steam.rs`)

## Logged-out (via proxy)
| estado | `steamcommunity.com/id/<nick>` |
|---|---|
| taken (gaben) | 200 (~57 KB) + `g_rgProfileData =`, sem not-found |
| disponível @gibberish | 200 (~27 KB) + "could not be found", SEM profile-data |
- Lógica atual CONFIRMADA (profile-data=taken, could-not-be-found=avail).
- Nota: `/id/` é vanity URL; conta sem vanity = outro formato (não confundir
  na validator — checamos o formato vanity mesmo).
