# Research — SoundCloud (`soundcloud.rs`)

## Logged-out (via proxy)
| estado | `soundcloud.com/<nick>` |
|---|---|
| taken @billieeilish | 200 (~115 KB) + `og:title`="Billie Eilish" + `"username"` + refs `/billieeilish` |
| disponível @gibberish | **404** (~66 KB) |
- Par limpo: 404=avail autoritativo; 200 + og:title + username-key +
  self-ref = taken (2+ sinais).
- 403 → barreira (código já trata; manter).

## Gap do checker atual (`soundcloud.rs:35-41`)
Marcadores taken `soundcloud://users:<nick>` e `"username":"<nick>"`
MORTOS (ausentes na page real) → ocupado = `RateLimited`.
Trocar pelos acima.
