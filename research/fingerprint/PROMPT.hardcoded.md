# PROMPT.hardcoded — L96A1 Sniper Web (versão completa com dados, NÃO usar no LMArena)

> ARQUIVO DE ARQUIVO: esta era a versão hardcoded do PROMPT.md antes da sanitização para LMArena. Mantida para uso futuro local (outra ferramenta / sessão com acesso aos arquivos). NÃO colar no LMArena — ele bloqueia esse conteúdo.
> Versão atual sanitizada (template): `PROMPT.md`.

# PROMPT.md — L96A1 Sniper Web Template (zero-contexto para LMArena)

> COMO USAR: o LMArena não tem acesso ao seu código, HAR ou screenshots. Copie TUDO abaixo da linha `--- PROMPT COMEÇA AQUI ---` e cole no LMArena como um único prompt. Ele vai gerar o template do zero. Depois você reutiliza o código gerado no seu projeto.
> Idioma do prompt: inglês (melhor resultado no LMArena). Textos da UI: inglês literal entre aspas. Este cabeçalho em português NÃO vai para o LMArena.

--- PROMPT COMEÇA AQUI ---

# Build the L96A1 Sniper web template from scratch

## 1. Product vision

Build a dark, clean, polished, organized, easy-to-use OSINT username-search web app called **L96A1 Sniper**.

UX inspiration: a parallel username / email / phone search dashboard with live streaming results, full-profile cards, breach lookup, history and settings — but with a **total rebrand**. Do NOT use the name, logo, copy or blue accent of `fingerprint.to`. Everything must be original L96A1 branding in a strict **black / gray / white monochrome** palette.

Tagline style: tactical, minimal, sniper scope aesthetic. No playful colors. No gradients. No blue buttons. Primary button = white background + black text. Secondary = transparent with gray border.

## 2. Stack and constraints

- Next.js App Router + Tailwind CSS. No real backend. UI + simulated streaming + fetch-ready adapter only.
- Fonts: `DM Sans` for UI + `JetBrains Mono` for numbers, code, handles, labels. System fallback ok.
- Routes (same design system on all): `/` landing, `/dashboard/search?q=`, `/pricing`, `/login`, `/username-search`.
- No images for branding. The 2 brand arts below must be recreated in code as ASCII / CSS / SVG components. Never use `<img>` for them.
- No Cloudflare Turnstile, no GTM, no Sentry, no auth logic. Mock everything.
- Responsive: 3 cols → 2 cols → 1 col. Sidebar collapses to top bar on mobile.

## 3. Design tokens (strict monochrome, no blue)

Use exactly these:

- Background: `#000000` page, `#0A0A0A` sidebar/navbar, `#111111` navbar pill, `#161616` cards, `#1A1A1A` raised surface, `#1E1E1E` inputs, `#222222` empty-state icon square.
- Borders: `#2E2E2E` default, `#424242` strong / focus. Border-width 1px, radius 10-14px for cards, 8-10px for inputs/buttons, full-round for pills/chips.
- Text: `#FFFFFF` headings / handles, `#ECECEC` body, `#A1A1A1` secondary, `#737373` muted / placeholders.
- Functional only: `#22C55E`-ish green for `Completed / done / FOUND` text, `#EF4444`-ish red only for `Delete account` border/text. Everything else stays gray. Yellows/oranges from breach-type labels are allowed only as tiny text labels, not as fills.
- Eyebrow labels: 11-12px, letter-spacing 0.2em, uppercase, mono, color `#A1A1A1`.
- H1 landing: 52-60px, 800 weight, white, tight leading. H2 sections: 32-40px, 700. Card title: 15-17px semibold white. Body: 14-15px `#A1A1A1`.
- Prohibited: `#1e6dd5` or any blue fill, blue links, blue Verified badge. Replace Verified with white bold check + gray border, or plain white `✓ Verified` text. Links inside cards are white underline on hover.
- Focus state: 2px white outline. Hover: border lightens from `#2E2E2E` to `#424242`, no color shift.

## 4. Brand identity recreated in code (mandatory)

Create two components:

### 4.1 `<SniperRifleAscii>`
Recreate this scene with monospaced ASCII characters in white `#ECECEC` on black:
- Top-left pixel-style title `L96A1 SNIPER` in blocky 5px-high font, characters made of small squares.
- Below, a long side-view sniper rifle spanning ~80% width, drawn with `- | _ / \ [ ] ( ) . :` characters: scope on top with 3 adjustment turrets, long thin barrel to the right ending in a slotted muzzle brake, stock on the left with dotted `:::` texture, trigger + magazine, bipod legs forming an `A` shape under the middle-right, plus a small ammo crate and 3 upright bullets to the right of the bipod, small grass tufts `\/` scattered, and a dotted baseline `...` under everything.
- Bottom-left: a small solid block cursor `_` that blinks.
- Add a subtle diagonal shimmer wave animation (a lighter gray band sweeping across every 4s) and a pulsing crosshair `⌖` near the scope. Pure CSS opacity animation, 60fps friendly.
- Usage (3 places, mandatory): (a) right side of landing hero behind/above the dashboard mock, large; (b) dashboard empty states instead of generic magnifier icon, small centered; (c) streaming/loading skeleton shimmer while cards arrive.

### 4.2 `<L96A1Wordmark>`
Big-block two-line logo:
- Line 1: `L96A1`, Line 2: `SNIPER`, each letter 7-9 rows tall, built from `#` fills with `| _ / \` outlines, pure white `#FFFFFF` on black, tight tracking, centered.
- Usage: navbar/sidebar logo (mini 22-26px height version + text `L96A1` next to it), landing hero small badge above H1, footer logo, favicon (simplified `L9` mono square).

Never render these as images. They are text/CSS.

## 5. Public landing page `/` (top to bottom)

### 5.1 Navbar
Floating pill bar, `#111111`, border `#2E2E2E`, radius 12px, max-width 1100px centered, padding 12px 20px. Left: `<L96A1Wordmark mini>` + `L96A1` bold white. Center links (14px gray, white on hover): `Demo, Features, Security, Pricing, Blog`. Right: white solid button `Dashboard`.

### 5.2 Hero
Two columns, left 45% / right 55%, black dotted grid background (tiny `#1A1A1A` dots).
- Left: H1 `Search Public Profiles Across 700+ Platforms` (make `700+` outlined stroke text, rest solid white). Subtitle gray 16px: `Enter a username, email or phone. We check 700+ platforms and stream every hit back in seconds.` Buttons row: primary white `Go to dashboard →` + outline gray `Free demo ⌕`. Below: stats row mono white numbers + gray labels: `700+ platforms | 10s avg search | $0 to start`.
- Right: layered dashboard mock window (dark chrome with 3 dots + URL bar `.../dashboard/search?q=john_doe`) showing: search input `john_doe` + white `Search ⚡2` button, line `38 profiles found · 2.8s`, pills `Discovered: @johndoe99·Reddit @jdoe_yt·YouTube + Hide`, filter row `Filter results... | Filters 38 found | Export`, grid of 6 mini cards: `YOUTUBE John Doe @jdoe_yt tech reviews 24.8K followers 166 Posts Joined Mar 18 2017 +1 more field`, `GITHUB john-doe backend engineer 567 Followers Location San Francisco CA Joined Jun 20 2018 +2 more fields`, `TWITTER John Doe @john-doe SWE 3.8K followers 2.1K following 1.2K Posts Location SF Bay Area`, `DISCORD john_doe#0001 server admin`, `SPOTIFY John Doe music lover`, `REDDIT johndoe99 r/just mod`, `MEDIUM John Doe`, `LINKEDIN John Doe Senior Software Engineer`. Behind the mock, faint large `<SniperRifleAscii>`.

### 5.3 How it works
Eyebrow `HOW IT WORKS`, H2 `Three steps with zero setup`, 3 equal cards:
- `01 Pick your lookup` text `Username, email, or phone. Choose a search type and enter your query.` + mini UI: 3 tabs `Username|Email|Phone` + input `johndoe` + arrow button.
- `02 Watch results stream` text `Sources are checked in parallel. Hits appear the instant they're confirmed.` + mono list rows `github.com FOUND / twitter.com FOUND / reddit.com FOUND / instagram.com FOUND` with green FOUND right-aligned.
- `03 Explore findings` text `Dig into profiles, cross-reference identities, filter and sort results.` + chips `xbox 3, ebay 2, reddit 1, steam 2, linkedin 1, github 2, twitter 1, discord 1, twitch 1` + toggle chips `✓ Verified, Private, Location` + `Clear all`.

### 5.4 Capabilities
Eyebrow `CAPABILITIES`, H2 `One query checks every platform in parallel`, sub `Full profiles, linked accounts, and a complete search history. Filter, sort, and export your results.` 3 cards top + 2 bottom:
- `REAL-TIME / Results as they're found / Every platform checked independently. Results stream to your screen as they arrive.` + mini 3x3 card collage + footer `9 results ... card view`.
- `DEEP SEARCH / Recursive discovery / New usernames and emails trigger fresh searches across 700+ platforms.` + chips `@johndoe ✓, @john.dev GitHub profile ✓, @jd.codes GitLab profile ✓, @jdoe_84 Reddit profile ✓` + dotted scatter field.
- `COVERAGE / 700+ platforms / Social, gaming, dev tools, forums.` + pixel dot-grid map (gray dots, white dots = hit) + legend `■ hit ■ miss ... 45`.
- Bottom row: `VIEWS / Table & card views / Full profiles with followers, bios, and locations. Browse in table or card layout.` with toggle `TABLE|CARDS` + 6 mini cards `INSTAGRAM John 3.1K / GITHUB 3.1K / LINKEDIN / DRIBBBLE UI/UX 898 Followers / TWITTER 15.2K Fol - 842 Flw / TELEGRAM 58 Followers` each with colored top strip (muted pink/blue gradients allowed only as thin 6px strip), `+N more fields`. And `LOGGING / Full audit trail / Every query logged with full history.` with mini table `QUERY|TYPE|#|STATUS|TIME` rows `johndoe username 7 done 2m ago, jdoe@mail.com email 3 done 14m ago, hunter2 data 42 done 22m ago...` footer `25 searches ... search history`.

### 5.5 Full profile feature
Split: left eyebrow `FULL PROFILES`, H2 `Every match is a full profile. Organized for review.` (`Organized for review.` in gray), paragraph `Bio, banner, follower counts, pinned repos, joined dates, sponsored orgs. We extract every field a platform exposes and render it inline. Open the banner, expand the bio, dig into raw fields.` + mono hint `try expanding +more fields →`. Right: large card with top banner image (circuit board photo, dark), overlapping round avatar, `GITHUB ↗ ✓ / Michael Carter / @user`, bio 2 lines + `Show more`, `3.1K Followers 184 Following 287 Posts`, rows `Location: Portland, OR / Email: user@example.com / Website: example.com (white link) / Joined: Mar 12, 2014 / Last active: Apr 22, 2025`, expander `▶ +7 more fields`.

### 5.6 Security
Eyebrow `SECURITY`, H2 `Your searches stay protected`, centered paragraph `Results are pulled from public sources and processed transiently rather than written to our database. The query and result count are saved to your history; the findings are returned to your browser without creating a stored archive.` 2x2 cards: `ACCOUNTS / Credentials protected at rest / When you use a password, it is stored only as a bcrypt hash. Session cookies are stored as one-way digests...`, `OSINT / Public data only / We only surface publicly available information. No hacking, no breaching...`, `AUDIT / Full audit trail / Every search logged to a complete history...`, `PRIVACY / Your data, your call / We don't sell your personal information...`.

### 5.7 API coming soon
Eyebrow `API COMING SOON` pill outline, H2 `Fetch any platform with one GET.` sub `Every platform we support sits behind a predictable URL: /v1/{platform}/{handle}. Plain HTTP, JSON out. No SDK, no client library...` + 3 checks `Same enriched profiles / Predictable URLs / Per-key auth with rate limits`. Button white-outline `Join the API waitlist` + note `We'll reply when keys are ready.` Right: code window with tabs `Go Node.js Python Ruby cURL`, file `main.go`, Go snippet fetching `https://api.l96a1.local/v1/linkedin/johndoe` with `Authorization: Bearer fp_live_xxx`, line numbers, mono syntax gray/white.

### 5.8 Social proof + FAQ + CTA + Footer
- `REAL USERS / What people are saying` + 2 large quote cards: `// / Bruhhh, your OSINT tool is killer! Maybe one of the most comprehensive ones I've used yet! / VIA REDDIT DM` and `It is so refreshing to see someone get it "right" and release a tool that delivers results alongside the heavyweights... / U/JASPER_NORTON ON R/OSINTTOOLS`.
- `FAQ / Frequently asked questions` accordion (closed, `+` right): `How does L96A1 work? / Is L96A1 a data broker? / How do I remove my information? / What platforms are searched? / How accurate are results? / Is it free? / Does it store results? / What can I use it for? / Do I need technical skills?`
- CTA box bordered: mini wordmark + `Pick a plan and start searching.` + sub `Every plan covers the same 700+ platforms with live results.` + buttons `Go to dashboard →` white + `Demo ⌕` outline.
- Footer 4 columns on `#0A0A0A`: brand + `Made in the USA / © 2026 L96A1` + `PRODUCT Features Pricing Security` + `SEARCH OSINT Tools Username Email Phone` + `COMPANY About Blog Contact` + `LEGAL Opt Out Privacy Terms`.

### 5.9 Pricing page content (also used as section)
Title `Find your search plan`, sub `Every paid plan includes all search types and exports. Choose your token allowance, with Deep Search on the Max plan.` + `✓ Monthly billing ✓ Cancel anytime ✓ Instant access`. 3 cards: `Pro $30/mo For solo work... TOKENS 500 ... Full People Search profiles, Unmasked Data Search, Export CSV JSON TXT [Get Pro]`, `Max $100/mo BEST VALUE For people who search every day TOKENS 2,500 ... Everything in Pro + Deep Search + Priority support + Early access [Get Max]` (highlighted white border), `Enterprise Custom For teams... 2,500+ ... API (coming soon) SSO Custom integrations Dedicated manager [Contact us]`. Below strip `INCLUDED ON EVERY PLAN: 700+ platforms / Real-time streaming / Every search type / Search history`. Compare table `Every plan, side by side` rows `Tokens, Every search type ✓, Real-time ✓, Table & card ✓, History ✓, Export ✓, Deep Search (Max/Ent ✓), Priority, Early access, API Coming soon, Webhooks, SSO, Contracts, Manager, Discounts`. FAQ tokens accordion `Can I try for free? / Can I switch later? / What is a token? / What happens when I use up tokens? / When do tokens reset? ...`

### 5.10 Public demo search pages (logged-out)
Same navbar + centered tabs `Username|Email|Phone` (active = white pill) + large input (`Enter a username... / Enter an email address... / Enter a phone number...`) with left magnifier + right crosshair icon + white-dim `→ Search` button + disclaimer `By using this search you agree to our Terms and Privacy Policy. Don't want to appear? Opt out.` + center empty `<SniperRifleAscii small>` + `Search social accounts / Enter a query above to get started.` + same footer.

## 6. App dashboard (logged-in, main focus)

### 6.1 Shell
Left sidebar 260px `#0A0A0A` right-border `#2E2E2E`, full height: top logo mini wordmark + `L96A1`, nav list with icons (14px gray): `Social Search (magnifier), People Search (people), Data Search (database), History (clock), Settings (gear)` — active item pill `#1A1A1A` white text. Bottom: `TOKENS 23/25` tiny label + thin progress bar (white fill) + `Resets Oct 31 ... Unlock more` row, divider, `user@example.com` white 13px + `FREE plan` gray + `Log out` with exit icon.
Top of content: centered pill tabs + big input + action button. Content max-width 1100px, padding 24px, gap 16px. All surfaces monochrome.

### 6.2 Social Search
Tabs `Username|Email|Phone`. Input placeholder `Enter a username...` focused state white border. Button `→ Search ⚡ 1` (white bg black text, lightning + cost). Center empty: `<SniperRifleAscii small>` + `Search social accounts` bold white + `Enter a query above to get started.` gray.

### 6.3 People Search (5 tabs, same shell)
Tabs `Email|Phone|Name|Address|Map`. Empty title always `Search for anyone` + `Select a search type above to get started.` Forms:
- Email: single input `Email address` + `Search ⚡1`.
- Phone: input `+1 Phone number` with tiny flag + `Search ⚡1`.
- Name: two inputs `First name *` + `Last name *` top row, `City | State | ZIP` bottom row + Search.
- Address: `Street address *` full width + `City * | State * | ZIP *` + Search.
- Map: full-bleed satellite map placeholder, top-center pill `Search this area ⚡1`, bottom-left toggle `Satellite|Map`, bottom-right `+/-` zoom.

### 6.4 Data Search (2 tabs)
Tabs `Breach|Exposure` + field row + toggles `= Exact (active white) / * Wildcard (gray)` + right link `Filter by database >`.
- Breach empty: left select `Username` + input placeholder + `→ Check ⚡1` + center database icon + `Run a data search / Pick a field above to get started.`
- Exposure empty: left select `Any` + input `email, username, domain, IP...` + Check + center icon + `Run an exposure search / Search sample logs. Pick a field above to get started.`
- Breach with results (sample `sample_handle`): banner `Unlock records / Searching is included on every plan. Viewing records needs a paid plan. [Unlock →]`, line `2 loaded records` + `Export ▾` right, summary strip `Total records 2 | Databases 2 | First seen 2022-01 | Last seen 2025-12`, then stacked blocks: header + rows `EMAIL •••••• / USERNAME sample(white link) / AVATAR URL ••• / CREATED AT ••• / FOLLOWINGS COUNT ••• / ID ••• / LAST MODIFIED ••• / LIKES COUNT ••• / PERMALINK URL •••`, second block similar. Masked values are gray dot strings.
- Exposure with no hit: gray box `No exposure found / Nothing matches this term.`

### 6.5 Results: Social Search (hero state, sample `sample_handle`)
Input filled sample handle, banner `Unlock profiles and extra fields / Searching is included on every plan. Profile links and extra fields need a paid plan. [Unlock →]`, centered `49 sample cards found · 20.8s` bold + `First seen Nov 9, 2005 · Last seen Sep 30, 2026` (dates white bold).
Toolbar row: left search `Filter results...` + `Filters` button with funnel icon + `49 found` gray, right icon buttons `sort, table, grid, Export ▾`.
Aggregators grid 3 cols:
- `Profile pictures (42) + Expand` with ~12 round avatars in wrapped rows.
- `Banners (9) + Expand` with 4 dark thumbnails 2x2.
- `Usernames (1)` chip sample.
- `Names (8)` chips sample names.
- `Locations (3)` chips sample regions.
- `Emails (4)` chips masked sample contacts.
Cards grid 3 cols, each card: top row small square favicon + `PLATFORM ↗ 🔒` (platform white uppercase 12px mono + external arrow + lock), second line `@handle` or `Display Name` bold white + `@handle` gray below, stats line, `Joined:` / `Last active:` gray, `Location:` if present, bottom expander `🔒 +N more fields` gray. Include in order sample platforms: `DOCKERHUB / ANILIST / SOURCEHUT / MATRIX / GITLAB / FRAGMENT / DIVISION2 / XBOX / DISQUS / DEVIANTART`, plus generic fillers to reach visual density. Locked fields stay masked.

### 6.6 History
Header `History` + `6 past searches` + right `Search history On|Off toggle + Clear history` + filter tabs `All types Username Email Phone People Breach Exposure` + right `Filter queries` input. Table `QUERY|TYPE|RESULTS|STATUS|TIME` with fictional sample rows only.

### 6.7 Settings
Header `Settings / Manage your account preferences`. Grid 3 cols: `Plan / Your subscription and billing / Free 25 tokens a month [Upgrade white] / Tokens left 18/25 bar / Resets Oct 31`, `Authentication / Your method / Signed in with Sample + Set password`, `Devices / Where signed in / Sample Browser ... This device`, `Email Address / Where codes go / user@example.com + ✓`, `Search History / Turn on or off / Enable search history [On|Off]`, `Delete Account (red title/border) / Close account and log out everywhere / ... [Delete account outline red]`.

## 7. Inline mock data (local use only, fictional samples)

Use sample query by default: `sample_handle`. Simulate streaming: reveal 1 card every 400ms with white `FOUND` flash, update header counter, show pills as discovered.

Full sample list (49 placeholders, in order): `Xbox, AniList, DockerHub, Disqus, Sourcehut, Fragment, Matrix, Division2, Kofi, DeviantArt, Letterboxd, GitLab, Medal, R6, Lichess, Discord, RegisteredEmails, Reddit, Pastebin, Poe, GitHub, Unsplash, Trakt, Fortnite, SoundCloud, Roblox, HuggingFace, Twitter/X, Twitch, Genius, Spotify, MySpace, Wordpress, Pinterest, LastFM, YouTube, Xvideos, Codeberg, Habbo, Telegram, XDAForums, Blogspot, TikTok, Ebay, RuneScape, NameMC, Pixabay, GunsLol, Minecraft`.

One result JSON shape to follow for every card:

```json
{"platform":"GitHub","platformUrl":"https://github.com","username":"sample_handle","displayName":"Sample","pfpUrl":null,"followers":0,"following":0,"posts":0,"createdAt":"2025-12-09T00:00:00Z","lastActive":"2025-10-02T00:00:00Z","isPrivate":false,"locked":true,"extras":{"1":"..."},"profileUrl":"https://github.com/sample_handle","sourceQuery":"","duration":"115ms"}
```

Event order to simulate in code: `connected {"status":"connected"} → ratelimit {"limit":10,"remaining":9} → start {"total":748} → loop(result + checked{"total":N}) → done {"type":"done","total":748}`. Expose switch `?live=1` that will later call `fetch('/api/l96a1/search?q=')` reading `text/event-stream` in this exact format with zero UI changes.

Other mocks: `GET /api/rate-limit → {"search":{"used":0,"limit":10,"remaining":10},"quota":{"used":2,"limit":25,"remaining":23},"plan":"free"}`, `GET /api/history → {"history":[{"query":"sample_handle","queryType":"username","resultCount":49,"status":"completed"}],"total":4}`, `POST /api/data-search {"field":"email"} → {"field":"email","found":3,"results":[{"email":"user@example.com","source":{"name":"SampleDB"}}]}`.

Map UI filters to backend groups (show as filter chips, platforms without checker show gray `SOON` badge and disabled card): `social/dev/gaming/streaming/music/community/manifest/wmn`. Missing-checker examples show `SOON`: AniList, DockerHub, Sourcehut, Codeberg, HuggingFace, Kofi, DeviantArt, Medal, Division2/R6/Fortnite trackers, Trakt, Poe, Wordpress, Blogspot, Ebay, Habbo, XDA, NameMC, Pixabay, Matrix, Fragment, RuneScape, Disqus, Genius.

Export dropdown must actually download current filtered results as JSON/CSV/TXT. Table/Cards toggle switches columns `PLATFORM|FOLLOWERS|LOCATION|CREATED|ACTIVE` vs card grid. `+N more fields` expands inline. Clicking a card opens full-profile modal (banner + avatar + all fields + raw JSON).

Favicons: gray square with first letter (no external fetch in template; wire `t2.gstatic` later with `loading="lazy"`).

## 8. Acceptance

- [ ] Landing + pricing + login render in pure monochrome, no blue, with rifle + wordmark in hero/navbar/footer/empty/loading.
- [ ] `/dashboard/search?q=sample_handle` streams 49 cards at ~400ms each, counter + pills update, toolbar filter + export + table/cards toggle work.
- [ ] People 5 tabs, Data 2 tabs, History table, Settings cards all match specs above.
- [ ] `?live=1` reads SSE in the exact event shape without UI changes. Platforms without checker show `SOON`.
- [ ] Responsive 3→2→1, DM Sans + JetBrains Mono, lazy favicons, real CSV/JSON/TXT export.

Build it now as production-quality Next.js + Tailwind components with clean folder structure. No placeholders, no lorem ipsum, use the exact strings above.

--- FIM DO PROMPT ---
