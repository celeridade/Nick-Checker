# Forebears — research (Fase 1)

Fonte: https://forebears.io/brazil/forenames, https://forebears.io/brazil/surnames
(espelho por pais: `/japan`, `/united-states`, `/spain`, ...).

## O que a pagina mostra (igual ao TSV)

| Coluna site | Campo TSV | Notas |
|---|---|---|
| Rank | `rank` | posicao por incidencia no pais |
| Gender (barra) | `gender` + `gender_pct` | `m`/`f` vem de `class="m"/"f"` no HTML; `u` = ambiguo (o markdown perde o rotulo, so o % sobrevive) |
| Forename/Surname | `name` + `slug` | slug = parte do href `forenames/<slug>` |
| Incidence | `incidence` | absoluto, com milhares (`14,524,659`) |
| Frequency | `frequency_1x` | `1:15` (guardamos so o `15`) |

Ex.: Brasil tem ~2.224.359 forenames unicos, ~92 pessoas por nome.
Top BR: Maria 14,5M (1:15), Jose 8M (1:27), Joao 3,8M (1:56).

## Parsing (validado contra scrapers open-source)

- Tabela em `<thead>...Frequency</th></tr></thead><tbody>` + linhas `<tr><td>`.
- Linha tipica: `<tr><td>RANK</td>... class="m|f" ... width:NNpx ... href=".../forenames/<slug>">NOME</a></td><td>INCIDENCE</td><td><span ...>...`.
- Genero 100% puro vem como `>100%<` + classe; misto vem como duas larguras px.
- Referencias: `jrzvnn/pop-names-scraper` (PH, colunas rank/forename/incidence/frequency/gender),
  `kahramankostas/Name-statistics-by-country-in-the-world` (CSV por pais via urllib + regex, UA Mozilla/5.0).

## Anti-bot / rate-limit (observado)

- Sem `User-Agent: Mozilla/5.0`, requests falham (transporte).
- Apos 1 fetch OK, fetches seguintes do mesmo IP falharam em sequencia
  (browser, webfetch, curl e Invoke-WebRequest) — rate-limit por IP.
- Rota que funcionou: **reader** (`https://r.jina.ai/https://forebears.io/...`),
  que devolve markdown em tabela (forenames COM coluna Gender, surnames SEM)
  ou, as vezes, texto TSV puro sem links — o importador parseia os 3 formatos
  (`parse_html_rows`, `parse_reader_table` + fallback TSV, `parse_markdown_rows`).
- O reader tambem rate-limita (403 apos N hits): importador tem UA rotativo,
  retry 5x com backoff + jitter, `--delay` com jitter e **resume** (pula
  artefato ja completo no manifest). Com isso vieram os 8 top-1k
  (BR/JP/US/ES x forenames/surnames) em tentativas alternadas.
- Fallback manual: salve a pagina no navegador como HTML
  (`<pais>-<kind>.html`) e use `--from-html-dir`.

## Paginacao (a confirmar na sua maquina)

- A pagina BR/forenames retornou ~1000 linhas; total unico do site (2,2M)
  sugere paginacao (`?page=N`) ou "show more". O importador segue links
  `?page=` ate `--max-pages` (default 1 = pagina principal).
- `manifest.complete=false` enquanto `rows < unique_total_site`.

## API paga (nao usar p/ bulk)

- Onograph API (`ono.4b.rs/v1/jur?key=...&name=..&type=forename|surname`):
  por-nome, com API key de 32 chars e custo por credito (+1 credito p/ gender).
  Serve p/ enriquecer 1 nome, nao p/ baixar paises inteiros.

## Normalizacao (igual dataset)

NFKD -> ASCII lowercase, mantem `[a-z0-9]` ("Joao" -> "joao",
"De Souza" -> "desouza"). Colisoes de acento (ex.: Jose/José) contam como
duplicado e ficam so no primeiro rank — `validate` acusa.
