# Research — Discord (`discord.rs`)

## Via register (`discord.com/register`, campo `input[name=username]`)
Mensagens ao vivo (UI):
- Disponível (gibberish do operador): **"Username is available. Nice!"**
- Ocupado (`test`): **"Username is unavailable. Try adding numbers,
  letters, underscores _ , or periods."**
- Palavra bloqueada (`discord`): **'Username cannot contain "discord"'**
  (client-side, sem rede).
- Campo limpo após os testes (nada submetido).

## Wire (endpoint ideal p/ checker, sem token!)
- `POST https://discord.com/api/v9/unique-username/username-attempt-unauthed`
  `{"username":"<nick>"}` → 200 `{"taken":true}` / `{"taken":false}`.
- Par autoritativo limpo. Sem autenticação (melhor que o pomelo atual!).
- Rate limit agressivo por rota/IP: via proxy queimado deu 429
  `retry_after: 1176s`. Implementação exige rotação + backoff + 429
  SEMPRE como barreira/Unknown (X-RateLimit headers a checar).
- Banned/deleted: Discord recicla? pomelo migration bloqueia alguns;
  sem exemplar — `Unknown` se ambíguo; `Banned` só com evidência futura.

## Método 3: modal logado (Settings → My Account → Username Edit)
- Modal: "Change your username / Enter a new username and your existing
  password." + campo + "Current Password" + Cancel/Done. NADA salvo
  (modal cancelado após os testes; sem tocar em senha).
- Mensagens ao vivo (conta real do operador):
  - livre (gibberish): "Username is available. Nice!"
  - ocupado (`test`): "Username is unavailable. Try adding numbers,
    letters, underscores _ , or periods."
  - charset: "Please only use numbers, letters, underscores _ , or periods."
- Wire do modal: sem fetch rastreável (vai pelo gateway WS ou bundle);
  equivale ao pomelo-attempt authed. Poucos checks espaçados = sem rate
  na sessão (não martelar conta real!).

## Estratégia 4 métodos (decisão do operador: manter token!)
1. **pomelo authed** `@me/pomelo-attempt` (token, maior confiança e cota).
2. **unauthed** `username-attempt-unauthed` POST `{"taken":bool}` (sem token).
3. **suggestions unauthed** `username-suggestions-unauthed?global_name=` GET:
   eco exato = livre, `nick+4 dígitos` = ocupado (2º sinal unauthed, barato).
4. **modal-equivalente** (mesmo pomelo; reserva).
- Orquestração: pool de tokens com rotação em 429 (padrão suenerve);
  401 pula p/ próximo token; pomelo Unknown → unauthed → suggestions;
  se nada conclui mas houve rate, devolve o rate (backoff) em vez de Unknown.
- Rate-limit oficial (discord-api-docs#rate-limits): honra `Retry-After`
  (header, vence o corpo) + `retry_after` JSON; marca `X-RateLimit-Scope`
  (`user`|`global`|`shared`) na evidência. Sem token, o limite global
  (50/s) cai no IP — por isso proxies + throttle por plataforma.
- AVISO (g1ver): pomelo-attempt só existe enquanto a conta NÃO trava um
  username novo — nunca complete troca de nick nas contas de check.

## Gap do checker atual (`discord.rs`)
Fechado: (1) unauthed como sinal sem token; (2) orquestrador multi-método
com pool e cotas; (3) fixtures dos 3 estados p/ cada método
(`decide_unauthed`, `decide_suggest`, `decide_pomelo` + `rate_info`).
