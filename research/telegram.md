# Research — Telegram (`telegram.rs`)

Só snapshots logged-out via proxy (t.me é estático, sem login).

## Logged-out (via proxy = igual ao checker)
| estado | `t.me/<nick>` |
|---|---|
| taken @durov | 200 (~11 KB) + `<title>Telegram: View @durov</title>` + foto (`tgme_page_photo_image`) + `tgme_page_title` + subscribers + `og:title`=nome + bio |
| disponível @zzqxkvw123nonexist | 200 (~9,7 KB) + `<title>Telegram: Contact @zzqxkvw123nonexist</title>` + SEM foto + SEM subscribers + `og:title`="Telegram: Contact @…" + `og:description` vazia |
- ACHADO CRÍTICO: página disponível TEM `tgme_page_action` + botão
  "Send Message" → marcador taken atual dispara em nick LIVRE
  (**falso Taken**, o erro mais caro p/ sniper).
- Decisão limpa: title `View @` + foto + subscribers = taken autoritativo;
  title `Contact @<nick>` (self-ref) + sem foto = avail autoritativo.

## Gap do checker atual (`telegram.rs:27-30`)
Sinal único `tgme_page_action` presente nos dois estados → falso Taken
em disponível. Reescrever com os dois sinais acima (2+ evidências).
Banido/deletado: t.me não distingue (sem exemplo) → `Unknown` se ambíguo.
