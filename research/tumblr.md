# Research — Tumblr (`tumblr.rs`)

## Transporte (achado operacional)
- Da nossa rede (direto E via proxies do pool): `tumblr.com` = **403
  (6.869 B, block de perímetro) nos dois estados**. Checker atual cego aqui.
- Via Brave do operador (IP residencial, logado): carrega normal.
- Implicação: Tumblr precisa rota residencial/sticky ou fonte alternativa;
  sem isso, tudo = `Unknown` honesto (nunca `RateLimited` fantasma).

## Estados via Brave (logado)
| estado | visível |
|---|---|
| taken @staff | title "@staff on Tumblr", `@staff / staff.tumblr.com`, Follow, posts |
| disponível @gibberish | title "Tumblr", landing ("Join over 100 million…", trending), sem blog |
- Marcadores p/ fixtures: self-ref `@nick / nick.tumblr.com` + posts = taken;
  landing genérica + title "Tumblr" = avail (confirmar redirect-vs-render
  na implementação).

## Gap do checker atual (`tumblr.rs:27-44`)
Supõe 404/302 que não se verificam daqui; `"blog":`/`login_required` não
validados logged-out. Reescrever após capturar fixtures (page via rota que
funcione + contraste taken/avail); até lá, sem chute.
