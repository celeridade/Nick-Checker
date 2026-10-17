# Research — Pinterest (`pinterest.rs`)

## Logged-out (via proxy = igual ao checker)
| estado | `pinterest.com/<nick>/` |
|---|---|
| taken @pinterest | 200 (~1,28 MB) + `"@type": "Person"` (JSON-LD) + self-ref |
| disponível @gibberish | 200 (~1,05 MB) + SEM Person + JSON `v3_get_user_handler`: **`"code":30,"message":"User not found."`** |
- Par autoritativo limpo: Person LD = taken; `User not found.` (code 30) = avail.
- Title vazio nos dois (`<title></title>` SSR) — não usar title como sinal.

## Gap do checker atual (`pinterest.rs:27-42`)
Taken ok (Person). Avail SEM marcador → 200 sem evidência = `ambiguous-200`
→ **RateLimited** em vez de Available (falso "rate": perde nick livre!).
Adicionar `User not found.`/`"code":30` como avail autoritativo.
Banido/privado: sem exemplos; Pinterest shadowbans não mudam a page
pública — sem ramo por ora (`Unknown` se ambíguo).
