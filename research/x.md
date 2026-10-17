# Research — X (`x.rs`)

Sessão via CDP no Brave-NOIA-Debug (logado). Nada de cookies/tokens extraído.

## Página de TROCA (`x.com/settings/screen_name`)
- Campo `input[name=typedScreenName]`, valor atual, `autocomplete=off`,
  SEM `maxlength`. Sem regra estática no DOM.
- Sugestões geradas exibidas (ex.: `<base>zbgv`, `<base>kq`, `<base>gzz`).
- Nota: "The handle you want might be on the Handle Marketplace / Browse handles"
  (handles de inativas podem estar à venda — nunca assumir liberável).
- Textos de erro de taken/inválido só após interação (não digitado em conta real).

## Estados logados (DOM)
| estado | visível |
|---|---|
| público @elonmusk | 109.2K posts, Follow, `og:title` presente |
| protegido @driko171 (próprio) | owner-view: 0 posts, "Set up profile" (sem muro p/ dono; muro "These posts are protected" é p/ terceiros) |
| suspenso (ex.: `fuck`) | title "Profile / X" + **"Account suspended / X suspends accounts which violate the X Rules"** |

## Logged-out = o que o checker vê (via proxy = igual ao checker)
| estado | page `x.com/<nick>` |
|---|---|
| público | 200 (~180 KB) + self-ref + sem protected |
| protegido | 200 (~47 KB) + self-ref + **`is_protected` / "These posts are protected"** |
| suspenso | 200 (~30 KB) + SEM self-ref + **`UserUnavailable/unavailable_reason:"Suspended"` + "Account suspended"** → `Banned` autoritativo |
| disponível | **404** (~27 KB) |
- Decisão limpa só com a page direta. Cuidado: `protected`/`screen_name`
  soltos aparecem no bundle — usar formas dado/contexto (validar em fixtures).

## Gap do checker atual (`x.rs:27-39`)
Só mirror vxtwitter (`api.vxtwitter.com`): 200+`screen_name`→taken,
404→avail. Problemas: (1) sem ramo protegido/suspenso; (2) mirror é
terceiro com Cloudflare — via proxy deu **403 challenge** (não confiável
p/ datacenter). Proposta: page direta como sinal 1 (self-ref + protected +
suspended→`Banned`), mirror como sinal 2 opcional.
