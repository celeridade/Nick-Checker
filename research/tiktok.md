# Research — TikTok (`tiktok.rs`)

Sessão via CDP no Brave-NOIA-Debug (logado). Aba research: `tiktok.com`.
Nada de cookies/tokens extraído; só estrutura DOM + JSON embutido.
Avatar URLs com `x-expires`/`x-signature` NÃO vão para fixtures (sanitizar).

## Estado: conta PRÓPRIA logada (@toxidez)
- URL: `https://www.tiktok.com/@toxidez` (redirect de `/` p/ perfil próprio)
- Visível: "Edit profile", 0 Following / 0 Followers / 0 Likes, "No bio yet."
- `__UNIVERSAL_DATA_FOR_REHYDRATION__` presente.

## Estado: PÚBLICO (@tiktok, oficial)
- URL: `https://www.tiktok.com/@tiktok` · title genérico "TikTok - Make Your Day"
- Visível: nickname + uniqueId, "96M Followers", botões Follow/Message, sem captcha.
- `__UNIVERSAL_DATA_FOR_REHYDRATION__` →
  `__DEFAULT_SCOPE__["webapp.user-detail"].userInfo.user`:
  - `uniqueId:"tiktok"`, `nickname:"TikTok"`, `verified:true`,
    **`privateAccount:false`**, `secret:false`
  - (contadores ficam em `stats`/`statsV2` — extrair na próxima passada)
- Marcador logged-out candidato (a validar): oembed 200 + `author_url`
  (atual) + `privateAccount:false` no HTML da página.

## Estado: PRIVADO (@guerreira, fornecido pelo operador)
- Title: "(6)This account is private. Follow this account to see their videos…"
- Visível: "This account is private" + "Follow this account to see their
  contents and likes", sem vídeos. Counters 63 Following / 0 / 0.
- Universal-data: `privateAccount:true`, `secret:true`, `verified:false`.

## Logged-out = o que o checker vê (via proxy = igual ao checker)
| estado | oembed | page |
|---|---|---|
| público @tiktok | 200 + `author_url` (684 B) | 200 + `userInfo` + `privateAccount:false` |
| privado @guerreira | **400 `Something went wrong`** | 200 (~374 KB) + `userInfo` + **`"privateAccount":true`** + "This account is private" |
| disponível @zzqxkvw123nonexist | **400 idêntico ao privado** | 200 (~372 KB) + sem `userInfo` + texto not-found |
| IP direto sem proxy | 429 `ratelimit triggered` (19 B) | — |
- ACHADO CRÍTICO: oembed 400 = privado OU disponível (indistinguível).
  O código atual (`tiktok.rs:35-37`) marca 400 como **avail_authoritative** →
  **falso positivo em toda conta privada**. Correção: 400 = ambíguo →
  2º sinal = page (`"privateAccount":true` → `Private`;
  sem `userInfo` + not-found → `Available`; resto `Unknown`).
- Cuidado: substring `privateAccount` solta aparece até em page disponível
  (bundle JS) — marcador válido é a forma dado `"privateAccount":true`.
  Validar com fixtures nos dois estados.

## Estado: BANIDO/SUSPENSO — sem distinção (declaração do operador)
- TikTok não expõe estado banido distinto; conta banida ≈ not-found.
- Sem ramo `Banned` p/ TikTok (variante segue útil p/ outras plataformas).

## Página de TROCA de username (modal Edit profile em @toxidez)
- Texto oficial: "Usernames can only contain letters, numbers, underscores,
  and periods. Changing your username will also change your profile link."
- Nickname: "can only be changed once every 7 days" (username: sem cooldown
  exibido no modal — não afirmar 30d sem fonte).
- Inputs sem `maxlength`/`pattern`/`aria-describedby` (validação server-side/JS);
  sem elementos de hint/erro no DOM parado.
- Campo username = `input[type=text]` com valor atual; NÃO digitei (conta real).
- Charset+limites oficiais p/ `validator()` (bundle `user.<hash>.js`,
  2026-10-09): regex `eR=/^[0-9a-z._A-Z]{2,24}$/`, `<2` min2char, `>24` max.
  Checker usa `UsernameValidator::new(2, 24, r"^[a-z0-9_.]{2,24}$")`.
- Textos de erro do modal: `editprofile_editpage_error_notavailable`
  (taken/inválido), `error_min2char`, `error_username_max`, toast
  `error_tryagain` (falha de rede no check).

## Gap do checker atual (`tiktok.rs:27-43`)
Só oembed (`author_url`/400/404). Privado/banido/deletado colapsam.
Proposta: oembed + `page` com `privateAccount`/`verified`/status vazio
→ `Taken` / `Private` / `Banned` / `Available` explícitos, resto `Unknown`.

## Método autenticado ATIVO (2026-10-09, sem browser)
`GET /api/uniqueid/check/?unique_id={nick}&...&X-Bogus=..&X-Gnarly=..`
+ `Cookie` da sessão + header `tt-csrf-token` => veredito real standalone:
- livre: `{is_valid:true, status_code:0}` (+ `recommended_unique_ids`);
- ocupado E privado: `{is_valid:false, status_code:3249}` (ambos não
  claimables — o oráculo não distingue privado de taken);
- sem sessão (mesmo assinado): vazio; sem assinatura: `{}`/3002301.
Receita provada live (direto E via proxy datacenter, UA Safari/Mac,
params app-shape `user_is_login=true`, device/odin frescos): ingredientes
decisivos foram a variante ATUAL do X-Bogus (magic `1245783967`, direto,
não o interleave de gists antigos) + X-Gnarly 5.1.1 como query params.
Pool em `tiktok_session.rs` (round-robin, cooldown, cap 3/nick);
veredito vira autoritativo e o fallback unauthed só roda sem veredito.
Bulk com 1 sessão: 8/8 decididos, 0 rate, 0 unknown.
Achado no modal Edit profile (conta @toxidez logada, nick `quilometragem`
com check verde `#0BE09B` / erro `PInputError` "This username isn't available"):
- Código cliente (`user.<hash>.js`): `checkUniqueId(e) => fetch.get("/api/uniqueid/check/",
  {query:{unique_id:e}, baseUrlType:FixedWww, headers:{...base, csrfToken}})`,
  debounce 1000ms, nick em lowerCase.
- Semântica da resposta: `is_valid && status_code==0` => `uniqueIdValid=1`
  (limpa erro = CHECK VERDE = claimable); senão `uniqueIdValid=2`
  (mensagem notavailable); erro de rede => `{status_code:6}` (toast tryagain).
- Validação local antes da rede: `<2` min2char, `>24` max,
  regex `eR=/^[0-9a-z._A-Z]{2,24}$/` => **charset+limites oficiais: 2-24**
  (pendência antiga de limites RESOLVIDA; checker usa 1-30 — apertar).
- Requisitos (resolvido em "Método autenticado ATIVO" acima): sem
  assinatura => `{}`/vazio; sem sessão (mesmo assinado) => vazio;
  receita completa (params app-shape + X-Bogus atual + X-Gnarly 5.1.1 +
  sessão + csrf) => veredito. Endpoint é session-bound => pool de
  sessões estilo Discord com fallback oembed+page.
- Mapa de status do `webapp.user-detail` (confirmado no bundle:
  `0===statusCode||10222===statusCode)&&userInfo`): **0**=pública,
  **10222**=privada (com userInfo), **10221**=livre/inexistente (sem userInfo).

## Nick livre vs shell: `webapp.user-detail` decide (2026-10-09)
Comparação lado a lado (page via proxy full x shell via proxy flagado):
- LIVRE (3 amostras: `zzqxkvw123nonexist999`, `qxzzfree01`,
  `uqnviqwyxzkj123`): `webapp.user-detail":{"statusCode":10221,...}`
  SEM `"userInfo":{` em lugar nenhum => **Available** (`detail-no-user`).
- SHELL (taken e livre geram o mesmo shell 276KB): universal-data SEM
  o bloco `webapp.user-detail`, sem `userInfo`, sem `statusCode` =>
  **Unknown** honesto.
- TAKEN: bloco com `"userInfo":{"user":{... "uniqueId":"{nick}"`.
- PRIVADO (`@guerreira`, live): bloco com `userInfo` + self-ref.
- Textos ("couldn't find this account", "this account is private",
  "suspended", "No videos yet") SÃO INÚTEIS como marcador: estão no
  bundle i18n e aparecem até na page TAKEN. Nunca usar sozinhos.
- Guarda: bloco sem user mas `statusCode:0` => Unknown (backend instável).
- CAVEAT banido: TikTok não expõe estado banido distinto; banida tende
  ao mesmo bloco 10221 => pode voltar Available e falhar só no claim.

## Pool de sessões + rota (2026-10-09)
- `tiktok_sessions.txt` (uma cookie-string por conta, espelho do
  `discord_tokens.txt`) + loader (`TIKTOK_SESSION[S]`/arquivo) + saúde em
  `/v1/doctor` (só `sess-xxxxxxxx`, sem segredo). Vazio = custo zero.
- Pool `tiktok_session.rs`: round-robin atômico, cooldown 60s em
  429/wall-3002301, aposentadoria em 401 (auto-cura 1h), cap 3 sessões/nick.
- Cascata: authed (`uniqueid/check`) decide em veredito REAL
  (`is_valid+code0`=Available / `is_valid:false`=Taken); resto cai no
  unauthed; rate authed só vale se o fallback for Unknown.
- X-Bogus na variante ATUAL (magic `1245783967`, direto sem interleave —
  gists antigos com magic `88,194,176,26` são rejeitados) + X-Gnarly 5.1.1
  (ChaCha20), ambos portados p/ Rust e testados (vetor XB byte-exato,
  round-trip simétrico do ChaCha, forma do token).
- Receita live: query app-shape (`unique_id` snake, `user_is_login=true`,
  device/odin frescos) + `&X-Bogus=` + `&X-Gnarly=` + `Cookie` + header
  `tt-csrf-token`, UA Safari/Mac. Sem sessão = vazio (pool é essencial).
  Provado direto E via proxy datacenter.
- `force_proxy`: `"tiktok"` REMOVIDO da lista do `HybridRouter`
  (era sticky de verdade mesmo com `sticky_proxy:false` no perfil —
  todo o bulk caía num IP só). Rota agora `proxy-req` com rotação.

## Anti-rate research (2026-10-08, via obscura + Brave-NOIA-Debug 9222 + 100 proxies)
- oembed e page = BUCKETS INDEPENDENTES: oembed 429 no IP direto
  (`ratelimit triggered`) enquanto `GET /@tiktok` segue 200 com
  `userInfo` no MESMO IP. Page nunca foi o problema.
- Teto oembed via proxy: 24/24 sequenciais no mesmo proxy, zero 429.
  Via proxy oembed responde 200+`author_url` (684B taken) / 400 idêntico
  (privado OU livre). IP direto já queimado persiste 429.
- Page via proxy: 8/10 proxies retornam full (~370-390KB + self-ref +
  `userInfo`); 2/10 datacenter recebem shell 276KB sem `userInfo`
  (bot/consent) => Unknown honesto, retry noutro proxy resolve.
- Causa do 1/94: `sticky_proxy: true` — 94 nicks x até 2 reqs todos no
  MESMO proxy (~150 reqs em rajada num IP). Fix: `sticky_proxy: false`
  (round-robin ~1-2 reqs/IP) + barreira nunca cai pra page no mesmo IP.
- Endpoints descartados: `node/share` (403 sem X-Bogus), `api/user/detail`
  (200 vazio sem assinatura `window.byted_acrawler.frontierSign`),
  `api/seo/user` (`status_msg: url doesn't match` sem params internos),
  `m.tiktok.com/oembed` (404), `/embed/@user` (503 `overload-protect` em
  rajada), mirrors tikwm/urlebird (403 Cloudflare em datacenter, planos
  pagos). Passport/signup-check não expõe verificação logged-out no JS
  do webapp-login-page (só `user.getUserUniqueId` de bridge nativa).
- `wm:tiktok` (gerado, oembed direto `needs_proxy:false`): além de queimar
  o IP direto, o probe genérico classifica 400 como Available =
  falso HIT em TODA conta privada. Recomendado EXCLUDE em
  `tools/gen_sites.py` (precedente: `mf:Twitter`, `mf:Spotify`) — nativa cobre.
