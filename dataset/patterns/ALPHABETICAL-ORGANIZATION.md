# Alphabetical / first-character organization

Every candidate list under `patterns` is split by the candidate's first character while preserving all previous classification dimensions.

Example:
`repetition/alphanumeric/3C/letters/3c-two-letters-repetition/a.txt` contains only candidates from that repetition class that start with `a`.

Special leading-character filenames:
- `dot.txt` = starts with `.`
- `hyphen.txt` = starts with `-`
- `underscore.txt` = starts with `_`

Candidates inside each file are lexicographically sorted. No candidate is duplicated across logical pattern files.

## Single repeater exception

`repetition/single-repeater/` is organized by the repeated character instead of by the candidate's first-character partition, because both are the same value. For example, `letters/a.txt` contains `aaa`, `aaaa`, and longer runs of `a`.
