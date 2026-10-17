# BRAVE-NOIA-Debug — perfil isolado p/ research + HAR

Comando oficial (PowerShell):

```powershell
& "${env:ProgramFiles}\BraveSoftware\Brave-Browser\Application\brave.exe" --remote-debugging-port=9222 --user-data-dir="$env:LOCALAPPDATA\Brave-NOIA-Debug" --new-window about:blank
```

- `--remote-debugging-port=9222` alimenta `tools/cdp.ps1` (`ws://127.0.0.1:9222/devtools/page/<TargetId>`) e `http://127.0.0.1:9222/json` (lista `TargetId`).
- `--user-data-dir=...Brave-NOIA-Debug` = perfil isolado. Login manual 1x. **Nunca extrair cookie/token.** Não toca no perfil principal.
- `--new-window about:blank` = janela limpa, sem restaurar sessão velha no HAR.

## Workflow HAR (somente leitura)

1. Subir o Brave com o comando acima.
2. Abrir `https://fingerprint.to/dashboard/search?q=adoecido`, DevTools → Network → `Preserve log` + `Disable cache`, resolver o Turnstile manualmente, rodar a busca.
3. No request `api/search?query=...` → `Save all as HAR with content` → salvar em `research/fingerprint/fingerprint.to.har`.
4. Extrair offline (zero rede por padrão):
   `python research\fingerprint\download-har.py fingerprint.to.har site-dump`
   Gera `_har-meta/{urls.txt,summary.json,sse-results.json,sse-events.json,apis/*.txt,site-map.md}`.
5. JS live sem HAR: `powershell -File tools\cdp.ps1 -TargetId <id> -Expression 'document.title'`
   Ex.: `document.documentElement.outerHTML.slice(0,4000)` para achar `og:image`, `avatar_url`, `banner`, `__NEXT_DATA__`, `SIGI_STATE`.

## O que anotar por plataforma (nativas primeiro)

Para cada uma das 29 nativas, atualizar `research/<plat>.md` com:
- endpoint de check + decisão (taken/avail/rate);
- endpoint de enrich (mesmo body ou 2º fetch público);
- seletores exatos: `og:image`, `avatar_url`, `profile_pic_url_hd`, `banner_image`, `__NEXT_DATA__`, etc.;
- exemplo real `adoecido` de `sse-results.json` (pfpUrl/bannerUrl/userId/bio).

Fonte de imagem NÃO é `forebears.md` (nomes por país) — é `research/fingerprint/site-dump/_har-meta/sse-results.json`.
