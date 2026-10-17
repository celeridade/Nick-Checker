# Tesseract data library

This directory is the single source for username generation and file-based checking.
Every candidate list is UTF-8, line-separated, trimmed, lowercased, transliterated to
username-safe ASCII (for example, `João` → `joao`), case-insensitively deduplicated,
and stored with a trailing newline. Documentation files are not loaded as candidates.

## Structure

- `patterns/`: deterministic number, letter, mixed-character, and repetition lists.

- `patterns/punctuation/`: 3- and 4-character candidates using `0-9 + a-z + _.-`,
  partitioned without duplicates into `underscore/`, `dot/`, `hyphen/`, and `mixed/`.
  The first three contain exactly one punctuation type; `mixed/` contains two or more types.
  The old `c/cc/ccc/cccc` files meant **characters**, not consonants. They are now
  named clearly and use the documented alphabet `0-9 + a-z`.
- `names/forebears/`: forenames + surnames por pais, importados do
  forebears.io (`/<pais>/forenames`, `/<pais>/surnames`). O TSV canonico
  guarda as colunas do site (rank, gender, gender_pct, name, slug,
  incidence, frequency_1x) mais `username` ascii e `source_url`; o `.txt`
  derivado lista os usernames em ordem de popularidade (rank). Ordem
  original de popularidade preservada. Ver `manifest.json` (versao, fonte,
  checksum, `complete`) e `tools/forebears_import.py`.
- `english/words/`: English words grouped by frequency, length, or ending; each logical list is partitioned by first character.
- `portuguese/words/`: Brazilian Portuguese words grouped by ending, plus a combined list; each logical list is partitioned by first character.
- `sites/br/`: clean hostnames ending in `.br` (including `.com.br`).
- `sites/com/`: clean hostnames ending in `.com`.
- `sites/other/`: clean hostnames using every other suffix.

Each site suffix folder contains FMHY recommendations by section, popular-brand categories,
a clothing list, and `all-sites.txt` for a deduplicated combined scan.


### Refined repetition layout

Pattern lists are partitioned so repeated candidates are not left in their base category:

- `patterns/characters/` = alphanumeric candidates with no repeated character.
- `patterns/letters/` = letters with no repeated letter.
- `patterns/numbers/` = digits with no repeated digit.
- `patterns/repetition/` = letter, digit, alphanumeric, and punctuation repetitions grouped by repeat count.
- `patterns/repetition/single-repeater/` = one-character-only repeats (`aaa`, `000`, `___`, etc.), stored by repeated character for lengths 3 through 32.
- `patterns/punctuation/` = punctuation candidates with no repeated punctuation symbol.
- `..` and `--` are excluded as unsupported; separated repeated dots/hyphens remain valid and are filed under repetition.
- `_` may repeat freely, including consecutively; pure underscore runs live in `single-repeater/punctuation/underscore.txt`.

## Runtime configuration

- `DATA_PATH` defaults to `./data`.
- `DATA_PROGRESS_FILE` defaults to `./results/data_progress.json`.
- `DATA_BATCH_SIZE` defaults to `20` candidates per check cycle.
- Run `npm run data:audit` to verify encoding, duplicates, generated cardinalities,
  clean hostnames, and `.br` / `.com` / other placement.
- Run `npm run data:normalize` to normalize the library in place.
  The JSON report includes the number of normalized values, and a second dry run
  must report `changedFiles: 0`.

## Raridade e popularidade (Fase 4)

- `rarity_score` (0-100): curto (1-4) + padrões de espaço pequeno
  (repeater, `aabb`) no topo; nomes cobiçados ganham +15.
- Cobiçado = top-1k Forebears (`names/forebears/*.txt`) ou `trends.tsv`
  com score >= 50. `l9 d --scan <filtro> --worthy 50` lista os mais
  raros para resgate manual (★ = cobiçado).
- `dataset/trends.tsv` (opcional, `term\tscore0-100`): gerado por
  `py tools/trends_import.py --csv base.csv` (ou `--terms` com pytrends).

## Provenance

- Character/word lists: the original English list collection and its README.
- English and Brazilian Portuguese refined lists: the original ICF/TWL-derived files.
- Popular names: forebears.io (Fase 1: seed `brazil-forenames` top ~1000;
  rode `py tools/forebears_import.py --countries brazil,japan,united-states,spain`
  na sua maquina para completar; o site rate-limita, use `--delay 3`).
  Os antigos `names/{brazil,japan,united-states}.txt` foram removidos.
- FMHY: starred main recommendations from https://fmhy.net/, extraction dated 2026-08-06.
  Only the canonical per-section main recommendations were retained; redundant category,
  subcategory, full-extraction, CSV, and JSON views were excluded from candidate data.
- Popular brands and clothing: the original curated domain lists.

