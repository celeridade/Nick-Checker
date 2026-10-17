# Alphabetical / first-character organization

Every logical candidate list in this directory is split by the candidate's first character while preserving its previous classification.

Examples:
- `by-ending/ing/a.txt` contains candidates from the `ing` ending list that start with `a`.
- `by-length/4-letters/b.txt` contains candidates from the 4-letter list that start with `b`.
- `common/common-100/t.txt` contains candidates from the common-100 list that start with `t`.
- `domain-names/3.txt` contains domain-name candidates that start with `3`.

Numeric leading characters use `0.txt` through `9.txt` when present.
Candidates inside each partition are lexicographically sorted.
Only non-empty first-character partitions are stored.
