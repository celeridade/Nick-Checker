# PROMPT.md — L96A1 UI Template (base para LMArena)

> COMO USAR: cole TUDO abaixo de `--- PROMPT COMEÇA AQUI ---` no LMArena. É um prompt de TEMPLATE de layout (base reutilizável), sem dados reais, sem backend, só estrutura visual para você ajustar depois.
> Este cabeçalho em português NÃO vai para o LMArena.

--- PROMPT COMEÇA AQUI ---

# Build a dark monochrome SaaS dashboard UI template called L96A1

## 1. Goal — UI template only, not a functional tool

Build a **reusable front-end UI base / layout template** (landing + dashboard shell + cards + tables + settings) in Next.js App Router + Tailwind CSS.

Important: this is a **design/layout exercise only**. There is NO real search, NO real backend, NO real personal data. Use ONLY fictional placeholder content like `john_doe`, `jane_smith`, `user@example.com`, `+1 (555) 0100` to demonstrate layout. All cards, tables and counts are static samples for spacing and component structure. I will wire real logic later myself.

Style: black / gray / white monochrome, clean, polished, organized, easy to scan. Tactical minimal vibe. No blue accent. Primary button = white bg + black text. Secondary = transparent + gray border.

## 2. Stack

- Next.js App Router + Tailwind. Static template with small client-side interactions only (tabs, filter input that filters visible sample cards, table/cards toggle, accordion, dropdown that downloads the visible sample list as JSON/CSV/TXT).
- Fonts: `DM Sans` for UI, `JetBrains Mono` for numbers/handles/labels.
- Routes with same design system: `/` landing, `/dashboard/search`, `/pricing`, `/login`, `/username-search`.
- Responsive 3 cols → 2 → 1. Sidebar 260px collapses to top bar on mobile.
- No auth, no fetch to real APIs, no images for branding.

## 3. Design tokens

- BG: `#000000` page, `#0A0A0A` sidebar, `#111111` navbar pill, `#161616` cards, `#1A1A1A` raised, `#1E1E1E` inputs, `#222222` icon squares.
- Borders `#2E2E2E` / `#424242`, radius 10-14 cards, 8-10 inputs/buttons, full-round pills/chips.
- Text `#FFFFFF` headings, `#ECECEC` body, `#A1A1A1` secondary, `#737373` muted.
- Only functional colors: green text for `Completed / FOUND`, red border/text for one `Delete` card. Everything else gray. No blue anywhere.
- Eyebrows 11-12px uppercase mono tracking wide gray. H1 52-60px 800 white. H2 32-40px. Card title 15-17px. Body 14-15px gray.
- Focus: 2px white outline. Hover: border `#2E2E2E` → `#424242`.

## 4. Brand art in code (no images)

Build two CSS/text components:

**`<SniperRifleAscii>`**: monospaced white-on-black scene: top-left blocky title `L96A1 SNIPER`, below a long side-view rifle drawn with `- | _ / \ [ ] ( ) . :` (scope with turrets on top, long barrel to slotted tip on right, textured stock left, trigger, `A`-shaped bipod mid-right, small crate + 3 bullets + grass tufts `\/`, dotted baseline, blinking `_` cursor). Subtle diagonal shimmer sweep + pulsing `⌖`. Use in: (a) hero background large faint, (b) empty states small centered, (c) loading skeleton shimmer.

**`<L96A1Wordmark>`**: two-line big block `L96A1 / SNIPER` built from `#` + `| _ / \` outlines, white on black. Mini 22px version for navbar/sidebar/footer + simplified `L9` favicon square.

## 5. Landing `/` — sections top to bottom

**Navbar**: floating pill `#111111` border, max 1100px: left mini wordmark + `L96A1`, center `Demo Features Security Pricing Blog` gray, right white `Dashboard` button.

**Hero** (2 col, dotted `#1A1A1A` dot background): left H1 `Search Profiles Across Many Platforms` (`Many` in outline stroke style), sub gray `Type a sample handle to preview the layout. This template streams sample cards for layout demo only.`, buttons `Go to dashboard →` white + `Free demo` outline, stats `700+ sample items | 10s demo | $0 to start`. Right: browser mock window with fake URL bar, inside: sample input `john_doe` + `Search` button, line `38 sample cards · 2.8s`, pills `Sample: @sample99 · Sample @sample_yt + Hide`, filter row `Filter samples... | 38 | Export`, 6 generic mini cards (title + @handle + 1-line bio + `12.4K followers · 300 Following · 120 Posts` + `Joined Mar 2017` + `+1 more field`). Faint rifle art behind.

**How it works**: eyebrow + H2 `Three steps with zero setup`, 3 cards: `01 Pick a filter` (tabs + input + arrow), `02 Watch list update` (rows `sample-one FOUND / sample-two FOUND` green right), `03 Explore layout` (chips `tag 3 tag 2...` + `✓ Featured Private Location`).

**Capabilities**: H2 `One query filters every sample card in parallel`, 3 cards `REAL-TIME Results as they're found / DEEP-LIST Related samples / COVERAGE Many categories` each with abstract mini UI (collage, chips `@sample ✓`, dot-grid `■ hit ■ miss`), plus row `VIEWS Table & card views` (toggle + 6 mini cards with thin muted top strip + `+N more`) and `LOGGING Sample history table` (`QUERY|TYPE|#|STATUS|TIME` with fictional rows).

**Full card feature**: split — left `Every item is a full card. Organized for review.` + paragraph + `try expanding +more fields →`, right large sample card: banner (dark abstract), avatar circle, `SAMPLE ↗ ✓ / Sample Name / @sample`, 2-line lorem-free generic bio + `Show more`, `3.1K Followers 184 Following 287 Posts`, rows `Location: Portland, OR / Email: user@example.com / Website: example.com / Joined: Mar 12, 2014 / Active: Apr 22, 2025`, `▶ +7 more fields`.

**Security**: H2 `Designed to stay clean`, 4 cards `ACCOUNTS / AUDIT / SAMPLES / PRIVACY` with 2-line generic template copy (no OSINT/breach claims).

**API block**: `API COMING SOON` pill, H2 `Fetch any sample with one GET.`, sub `/v1/{category}/{id}. Plain HTTP, JSON out.`, checks x3, `Join waitlist` outline button. Right code window tabs `Go Node Python Ruby cURL` with short generic fetch snippet to `https://api.example.local/v1/sample/john_doe`.

**Proof + FAQ + CTA + footer**: 2 quote cards (generic fictional quotes about "clean template"), accordion FAQ (`How does the template work? / Is data real? No, all samples fictional / How do I customize? ...`) with `+`, CTA box `Pick a layout and start building.` + 2 buttons, footer 4 col `PRODUCT / SAMPLES / COMPANY / LEGAL` + `© 2026 L96A1`.

**Pricing section**: `Find your layout plan`, 3 cards `Starter $30 / Pro $100 BEST / Custom`, token-bar graphic, checklist, `INCLUDED: many samples / live demo list / filters / history` strip, compare table, small FAQ.

**Logged-out demo form**: centered tabs + big input (`Enter a sample handle...`) + Search button + disclaimer line + centered rifle art + `Browse sample layouts / Type above to preview.` + footer.

## 6. Dashboard `/dashboard/search` — shell + states

**Shell**: sidebar `#0A0A0A` 260px: logo, nav `Overview (magnifier), Directory (people), Records (database), History (clock), Settings (gear)` active pill `#1A1A1A`, bottom `CREDITS 23/25 bar + Resets Oct 31 Unlock more`, divider, `user@example.com FREE plan + Log out`. Content max 1100px padding 24 gap 16.

**Top**: pill tabs `Handle|Email|Phone` + big input `Enter a sample handle...` + white `→ Preview` button. Empty center: rifle art small + `Browse sample cards` + `Type above to preview the layout.`

**Directory variant** (same shell, tabs `Email|Phone|Name|Address|Map`): empty `Browse samples / Pick a filter above.` Forms: Email single input, Phone `+1 (555) 0100`, Name `First* Last* / City State ZIP`, Address `Street* / City* State* ZIP*`, Map full-bleed stylized gray map placeholder (no real tiles) with `Preview area` pill + `Satellite|Map` toggle + `+/-`.

**Records variant** (tabs `List A|List B` + `= Exact / * Contains` toggles): empty states with icon + `Run a sample filter / Pick a filter above.` Results state: banner `Unlock extra columns / Preview included. Extra columns need a plan. [Unlock →]`, line `2 sample rows` + `Export ▾`, strip `Total 2 | Groups 2 | From 2022-01 | To 2025-12`, stacked blocks `◉ Sample Group ... 2025-12` with rows `EMAIL •••••• / HANDLE sample (white) / AVATAR ••• / CREATED ••• ...` masked dots, second block similar. Empty-hit box `No samples found / Nothing matches this filter.`

**Results state** (main): input filled `john_doe`, banner `Unlock extra columns... [Unlock →]`, centered `49 sample cards · 20.8s` + `First Jan 9, 2005 · Last Sep 30, 2026` gray. Toolbar `Filter samples... | Filters 49 | Export ▾ + view icons`. Aggregator grid 3 col: `Avatars (12)` round placeholders + `Expand`, `Banners (4)` dark thumbs 2x2, `Handles (1)` chip, `Names (6)` chips, `Regions (3)` chips, `Contacts (4)` chips with `user@example.com` style placeholders. Cards grid 3 col: favicon square + `CATEGORY ↗` mono + `@handle` bold + stats `0 Followers 0 Following` + `Joined: ...` + `🔒 +N more fields`. Use ~10 detailed samples + generic fillers for density. All masked/locked.

**History**: `History / 6 past previews` + `History On|Off + Clear` + tabs `All Handle Email Phone Directory Lists` + `Filter` input. Table fictional rows only (`john_doe handle 49 Completed 1h ago` etc.).

**Settings**: `Settings / Manage template preferences`. Grid: `Plan Free 25 credits [Upgrade] bar 18/25`, `Sign-in `Signed in with Sample` + [Set password]`, `Devices Sample Browser ... This device`, `Email user@example.com ✓`, `History On|Off`, `Delete (red border)`.

## 7. Behavior (front-end only)

- Typing filters visible sample cards client-side. Counter updates. Pills appear.
- Simulated append: 1 sample card per 400ms with `FOUND` flash until ~12 cards, then stop. No network.
- Export downloads visible samples as JSON/CSV/TXT. Toggle switches table (`ITEM|COUNT|REGION|CREATED|ACTIVE`) vs cards. `+N more` expands inline. Card click opens modal (banner + rows + raw JSON block of fictional sample object below).
- Fictional sample object shape for every card:
```json
{"site":"SampleSite","siteUrl":"https://example.com","handle":"john_doe","title":"Sample Name","followers":0,"following":0,"createdAt":"2025-01-01T00:00:00Z","locked":true,"profileUrl":"https://example.com/john_doe"}
```
- Favicons: gray square with first letter (no external fetch).

## 8. Acceptance

- [ ] Pure monochrome, no blue, rifle + wordmark in hero/navbar/empty/loading.
- [ ] Landing + pricing + login + dashboard shell render from one design system.
- [ ] Sample stream (~400ms/card), filter, export, table/cards, expand, modal all work on fictional samples.
- [ ] All copy uses fictional placeholders only (`john_doe`, `user@example.com`). No real emails, no real handles, no breach/stealer claims.
- [ ] Responsive 3→2→1, DM Sans + JetBrains Mono.

Build now as clean Next.js + Tailwind components. No lorem ipsum, use the exact strings above.

--- FIM DO PROMPT ---
