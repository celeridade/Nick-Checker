# Research — Instagram (`instagram.rs`)

Sessão via CDP no Brave-NOIA-Debug (logado). Nada de cookies/tokens extraído.

## Página de TROCA (Accounts Center → Edit profile)
- Username exibido como texto ("Premeditar" p/ editar, UI em PT).
- Inputs sem `maxlength`/`pattern`; só label "Bio" como `<label>`.
- Clique via CDP em "Premeditar": sem dialog, foco continua no BODY
  (editor não abriu por evento sintético ou requer gesto real).
- PENDENTE: operador clica "Premeditar" e descreve (campo? regras? erros?).

## Logged-out = o que o checker vê (via proxy, headers fiéis incl. X-IG-App-ID)
| estado | page `/{user}/` |
|---|---|
| público @instagram | 200 (~635 KB) + SEM `edge_followed_by`, SEM `"username":"`, SEM `is_private`, SEM sorry-text |
- ACHADO: marcadores taken atuais NÃO aparecem logged-out → `decide_http`
  cai em `ambiguous-200` → `aggregate` retorna **RateLimited** p/ perfil
  ocupado! (falso "rate", seguro mas errado e ruidoso).
- `i.instagram.com/api/v1/users/web_profile_info/?username=`: 429 nos dois
  estados (proxy queimado p/ endpoint ou exige login). Candidato, não conclusão:
  revalidar com proxy fresco + retry na implementação.

## Página de TROCA (Accounts Center `/profiles/<id>/username/`)
- Única regra exibida: "Changing your username will also change your
  Instagram profile address." Sem charset/limite/cooldown no DOM estático.
- Input sem `maxlength`. Sem digitar na conta real (textos de erro de
  taken/inválido pendentes via signup logged-out).

## Estado: PRIVADO (@_kayszx, fornecido pelo operador)
- Visível logado: "This profile is private / Follow to see their photos
  and videos." (80 followers).
- Logged-out page: 200 (~635 KB) + SEM `edge_followed_by`, SEM
  `"is_private":true`, SEM sorry-text — **idêntica à pública**.

## Logged-out: shells sem sinal (via proxy, headers fiéis)
- pública, privada, disponível e pejorativa (`fuck`): todas 200, todas
  `polarisRouteConfig.pageID = httpErrorPage` (login wall p/ deslogado).
- Disponível tem `"url":"\/<nick>\/"` no route-data (eco da URL, não perfil).
- Hipótese do banido REFUTADA p/ pages: status e corpo não distinguem
  banido de inexistente — pejorativo é indistinguível por este vetor.
- Caminho restante: API (`web_profile_info` — 429, revalidar) ou endpoint
  de check do signup (sondar na implementação, logged-out e seguro).

## Gap do checker atual (`instagram.rs:35-41`)
Marcadores taken mortos logged-out → tudo cai em `ambiguous-200` →
agregador devolve **RateLimited** até p/ perfil ocupado. Reescrever com
API/signup-check; `Banned` só se a API distinguir suspensão.

---

# Sessão 2026-10-10 — bulk sem rate-limit (Brave-NOIA-Debug + curl_cffi)

Método: CDP no perfil debug (`E329B8E…`, logado `@premeditar`, nada extraído)
+ `curl_cffi 0.16.3 impersonate=chrome` (HTTP/2 + TLS de browser) via
`proxies.txt` (100). `requests` puro (HTTP/1.1) foi controle negativo.

## ACHADO CENTRAL: page logged-out DISTINGUE com TLS correto
A "casca idêntica ~635KB" anterior era resposta p/ fingerprint bloqueado
(HTTP/1.1 / rustls). Com HTTP/2 + TLS chrome, `GET www.instagram.com/{u}/`
logged-out diferencia (5 nicks, mesmo proxy):

| nick | estado real | status | len | `<title>` | `og:image` | `username_n` |
|---|---|---|---|---|---|---|
| `@instagram` | taken público | 200 | ~847k | `Instagram (&#064;instagram) • …` | sim (URL) | 29 |
| `@_kayszx` | taken privado | 200 | ~749k | `… (&#064;_kayszx) • …` | sim (vazio) | 11 |
| `adoecido` | **disponível** | 200 | ~629k | `Instagram` | não | 2 |
| `xkqzptwjmv12345zz` | disponível | 200 | ~629k | `Instagram` | não | 2 |
| `fuck` | banido/reservado | 200 | ~629k | `Instagram` | não | 2 |

Regra: title contém `(&#064;<nick>)` OU `og:image` presente OU
`username_n>=8` → **taken**; title exato `Instagram` + sem `og:image` +
len<670k → **avail**. `fuck` é indistinguível de disponível por este vetor
(exige signup-check residencial p/ separar banido de livre).
`adoecido` está LIVRE no IG (topsearch confirma: sem exact-match).

## Rajadas (budget real)
- Logged-out page, mesmo proxy+sessão, 20 reqs sequenciais: **19/20 sem
  nenhum 429** (1 timeout de stall, não throttle). Lens estáveis nas 4 voltas.
- Logged-in page (CDP fetch), 25 reqs: **25/25 200, zero 429**.
- Logged-in `topsearch`, 12 reqs: **12/12 200, zero 429**.
Conclusão: **page + topsearch têm budget alto p/ bulk**; não há endpoint
"sem limite", mas estes aguentam sniper com sticky proxy + throttle leve.

## `topsearch` logged-in (secundário ideal, 2-8KB)
`GET /api/v1/web/search/topsearch/?query=` + `X-IG-App-ID`:
`instagram/premeditar/_kayszx` → exact-match true (taken, privado incluso);
`adoecido/fuck` → exact false; random → `users:[]`. Regra:
exact (case-insensitive) = taken; `n==0` = provável avail; `n>0` sem exact =
avail provável OU banido (cruzar com page). Logged-out dá 429 (exige login).

## Mortos / condicionais p/ bulk datacenter
- `i.instagram.com/.../web_profile_info/`: logged-out = 401
  `require_login:true` (até direct sem proxy, com HTTP/2); logged-in via
  fetch www→i. também 401 (cookies www não vão p/ domínio i.); via
  `requests` HTTP/1.1 = 429 vazio. Exige sessão no domínio i. ou mobile auth.
  Não usar como primário.
- `web_create_ajax/attempt/` logged-out (curl_cffi, csrf+mid frescos):
  **429 `{"spam":true,"status":"fail"}`** p/ taken E avail — ASN datacenter
  do pool flagado. Só com **residencial**. Logged-in retorna HTML (contexto
  errado). `username_suggestions` igual.
- `/{u}/embed/` logged-out distingue (249k taken vs 207k avail) — fallback
  barato válido, mesma exigência de TLS.
- Mirrors: greatfon/dumpor (`dumpor.io`) mesma página p/ taken/avail no HTML
  estático + `Verifying…` (challenge) no browser; picuki/imgsed 403
  Cloudflare; gramhir DNS morto. **Inviáveis p/ bulk**.
- Threads `threads.net/@u` via fetch = CORS fail; oEmbed só serve p/ posts
  (`invalid param 'url'` p/ perfil). Graph `Business Discovery` só cobre
  Business/Creator.

## Recomendação p/ `instagram.rs` (bulk, sem login)
1. Primário logged-out: `GET https://www.instagram.com/{u}/` com headers
   browser (`Accept-Language`, `Referer`) + sticky proxy, decisão por
   title/og/`username_n` (não por len sozinho).
2. Fallback: `web_profile_info` (quando houver sessão i./mobile) p/ refinar
   Private; `embed` como 2º sinal barato.
3. Follow-ups: (a) TLS impersonation no Rust (`rquest` impersonate chrome —
   reqwest/rustls hoje recebe a casca 629k); (b) pool opcional
   `INSTAGRAM_SESSIONID` p/ `topsearch` exact-match; (c) residencial p/
   `attempt` (único que separa banido de livre + 14-day `username_held_by_others`).
