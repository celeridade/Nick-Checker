# Research — Myspace (`myspace.rs`)

## Logged-out (via proxy)
| estado | `myspace.com/<nick>` |
|---|---|
| taken @tom | 200 (~101 KB) + title **"Tom Anderson (tom) on Myspace"** |
| disponível @gibberish | **404** (~87 KB) |
- Taken limpo: title self-ref `(<nick>) on Myspace` (+ nome).
  Marcadores atuais (`profileHeading`/`profile-user`) MORTOS (ausentes).
- Par: 404=avail autoritativo; 200 + title-self-ref = taken.

## Gap do checker atual (`myspace.rs:27-36`)
Trocar taken-marker por title self-ref (+ 2º sinal a definir em fixture,
ex. foto/bio); estrutura 404 mantida.
