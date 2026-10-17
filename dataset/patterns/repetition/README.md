# Repetition patterns

This directory contains username candidates with repeated characters.

## Layout

- `single-repeater/`: the username is made from one character only.
  - `letters/a.txt` through `letters/z.txt`
  - `numbers/0.txt` through `numbers/9.txt`
  - `punctuation/underscore.txt`
  - each file contains lengths `3..32` in ascending order
- `letters/`, `numbers/`, `alphanumeric/`, `punctuation/`: other repetition patterns for the exhaustive `2C`, `3C`, and `4C` sets.

## Rules

- Every candidate under `patterns/` should belong to exactly one `.txt` file.
- Pure single-character repeats of length 3 or more belong only in `single-repeater/`.
- Two-character repeats (`aa`, `00`, etc.) remain in the `2C` exhaustive repetition sets.
- Consecutive dots (`..`) and consecutive hyphens (`--`) are unsupported and excluded.
- Consecutive underscores are allowed.
- `mixed` repetition means more than one repeated symbol family exists in the same candidate.
- Example: `a3a4` -> `alphanumeric/4C/letters`; `a33a` -> `alphanumeric/4C/mixed`.

The `single-repeater` range is intentionally broader than the exhaustive 2-4 character grids so a username checker can test long repeated handles without generating combinatorial datasets for every longer length.
