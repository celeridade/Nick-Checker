# Research — Minecraft (`minecraft.rs`)

## API Mojang lookup (via proxy)
| estado | `api.mojang.com/.../lookup/name/<nick>` |
|---|---|
| taken (Notch) | 200 + `"id"`, `"name"` |
| disponível @gibberish | **404** (143 B) |
- API antiga SEGUE VIVA — lógica atual CONFIRMADA (200+id=taken, 204/404=avail).
- Riscos p/ implementação: (1) rate agressivo já mitigado (300ms + 16conc —
  manter); (2) migração p/ `api.minecraftservices.com` no horizonte —
  monitorar sunset e preparar fallback; (3) 429 → barreira sempre.
