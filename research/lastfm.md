# Research — Last.fm (`lastfm.rs`)

## Via signup: endpoint de check (RESOLVE o muro)
- `POST https://www.last.fm/join/partial/validate` form-encoded +
  `X-Requested-With: XMLHttpRequest` (+ `csrfmiddlewaretoken` da page `/join`).
- Taken (@RJ): 200 `userName:{valid:false,
  error_messages:["Sorry, this username isn't available."]}` → taken
  autoritativo.
- Disponível (gibberish): 200 `userName:{valid:true}` SEM chave
  `error_messages` → avail autoritativo. (UI mostra "Ok, that username
  can be yours!" nos dois casos — ler o JSON, não o texto!)
- Pendente p/ implementação: o endpoint aceita sem CSRF? funciona via
  proxy/datacenter ou só residencial? (page `/user/` é 404 p/ livre =
  "Page Not Found" — confirmado pelo operador — bom 2º sinal.)

## Transporte/WAF (contexto)
- Pages `/user/` via proxies: shell CSP idêntico nos dois estados → cego.
- Via Brave: firewall "Rate Limited" até no IP do operador em navegação
  direta. Endpoint `/join/partial/validate` funcionou no browser —
  validar via proxy na implementação.
- Marcador antigo `header-title-display-name`: descartar (nunca aparece
  nos shells medidos).

## Gap do checker atual (`lastfm.rs:27-36`)
Estrutura ok (404=avail, display-name=taken), mas sem prova de que o
marcador aparece logged-out. Na implementação: capturar fixture real
(ou via sessão logada no Brave) e só então manter; até lá, sem
mudança de classificação (regra ouro).
