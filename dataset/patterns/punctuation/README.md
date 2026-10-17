# Punctuation patterns

Base punctuation files contain candidates with no repeated characters.
Single punctuation types are split into `dot`, `hyphen`, and `underscore` and then by total length.
`mixed` is split by total length, exact punctuation set, and remaining character composition.
Candidates with any repeated character are moved to `patterns/repetition/punctuation`.
