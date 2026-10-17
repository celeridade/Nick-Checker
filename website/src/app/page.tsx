import Link from "next/link";
import {
  ArrowRight, Crosshair, Database, Eye, FileClock, Fingerprint,
  LayoutGrid, Lock, Radar, ScanSearch, ShieldCheck, Table2, Zap,
} from "lucide-react";
import SiteNav from "@/components/SiteNav";
import SiteFooter from "@/components/SiteFooter";
import L96A1Wordmark from "@/components/L96A1Wordmark";
import { faviconUrl } from "@/lib/results";

/* ---------------------------------------------------------------- helpers */

const btnWhite =
  "inline-flex items-center gap-2 rounded-lg bg-white px-5 py-2.5 text-sm font-semibold text-black transition-colors hover:bg-body";
const btnOutline =
  "inline-flex items-center gap-2 rounded-lg border border-linestrong px-5 py-2.5 text-sm font-semibold text-body transition-colors hover:border-white hover:text-white";
const card = "rounded-xl border border-line bg-card";

function Eyebrow({ children }: { children: React.ReactNode }) {
  return <p className="eyebrow">{children}</p>;
}

function SectionHead({
  eyebrow, title, sub, center,
}: { eyebrow: string; title: React.ReactNode; sub?: string; center?: boolean }) {
  return (
    <div className={`max-w-2xl ${center ? "mx-auto text-center" : ""}`}>
      <Eyebrow>{eyebrow}</Eyebrow>
      <h2 className="mt-3 text-[32px] font-bold leading-tight tracking-tight text-heading sm:text-[38px]">
        {title}
      </h2>
      {sub && <p className="mt-4 text-[15px] leading-relaxed text-sub">{sub}</p>}
    </div>
  );
}

/* Hero mock mini-card ----------------------------------------------------- */
function HeroMiniCard({
  platform, url, name, handle, bio, lines, more,
}: {
  platform: string; url: string; name: string; handle?: string;
  bio?: string; lines?: string[]; more?: string;
}) {
  return (
    <div className="rounded-lg border border-line bg-raised p-3 text-left">
      <div className="flex items-center gap-1.5">
        {/* eslint-disable-next-line @next/next/no-img-element */}
        <img src={faviconUrl(url)} alt="" width={12} height={12} loading="lazy" className="rounded-sm" />
        <span className="font-mono text-[10px] uppercase tracking-wider text-sub">{platform}</span>
      </div>
      <p className="mt-1.5 truncate text-[12px] font-semibold text-heading">{name}</p>
      {handle && <p className="truncate font-mono text-[10px] text-mute">@{handle}</p>}
      {bio && <p className="mt-1 truncate text-[10px] text-mute">{bio}</p>}
      {lines?.map((l) => (
        <p key={l} className="mt-0.5 truncate text-[10px] text-mute">{l}</p>
      ))}
      {more && <p className="mt-1.5 text-[10px] text-mute">▸ {more}</p>}
    </div>
  );
}

/* ------------------------------------------------------------------- page */

export default function LandingPage() {
  return (
    <div className="min-h-screen bg-page">
      <SiteNav />

      {/* ---------------------------------------------------------- HERO */}
      <section className="dotgrid relative overflow-hidden pt-36 pb-20 lg:pt-44">
        <div className="mx-auto grid max-w-[1200px] items-center gap-14 px-6 lg:grid-cols-[45%_55%]">
          <div>
            <div className="inline-flex items-center gap-2.5 rounded-full border border-line bg-pill px-3.5 py-1.5">
              <span className="pulse-scope font-mono text-sm text-heading">⌖</span>
              <span className="font-mono text-[11px] tracking-[0.2em] text-sub">L96A1 SNIPER — OSINT SEARCH</span>
            </div>
            <h1 className="mt-6 text-[44px] font-extrabold leading-[1.04] tracking-tight text-heading sm:text-[54px] lg:text-[58px]">
              Search Public Profiles Across <span className="stroke-text">700+</span> Platforms
            </h1>
            <p className="mt-5 max-w-md text-base leading-relaxed text-sub">
              Enter a username, email or phone. We check 700+ platforms and stream every hit back in seconds.
            </p>
            <div className="mt-8 flex flex-wrap gap-3">
              <Link href="/dashboard/search" className={btnWhite}>
                Go to dashboard <ArrowRight size={15} />
              </Link>
              <Link href="/username-search" className={btnOutline}>
                Free demo <span className="font-mono">⌕</span>
              </Link>
            </div>
            <div className="mt-9 flex items-center gap-5 font-mono text-xs text-mute">
              <span><span className="text-heading">700+</span> platforms</span>
              <span className="text-line">|</span>
              <span><span className="text-heading">10s</span> avg search</span>
              <span className="text-line">|</span>
              <span><span className="text-heading">$0</span> to start</span>
            </div>
          </div>

          <div className="relative">
            <div
              aria-hidden
              className="pointer-events-none absolute -top-24 -right-16 -bottom-24 -left-16 dotgrid-lg fade-edges"
            />
            <div className="relative z-10 rounded-xl border border-line bg-surface shadow-[0_28px_80px_-18px_rgba(0,0,0,0.9)]">
              <div className="flex items-center gap-2 border-b border-line px-4 py-2.5">
                <span className="size-2.5 rounded-full bg-iconbox" />
                <span className="size-2.5 rounded-full bg-iconbox" />
                <span className="size-2.5 rounded-full bg-iconbox" />
                <div className="ml-3 flex-1 truncate rounded-md border border-line bg-input px-3 py-1 font-mono text-[11px] text-mute">
                  .../dashboard/search?q=john_doe
                </div>
              </div>
              <div className="space-y-3 p-4">
                <div className="flex gap-2">
                  <div className="relative flex-1">
                    <ScanSearch size={14} className="absolute top-1/2 left-3 -translate-y-1/2 text-mute" />
                    <div className="rounded-lg border border-line bg-input py-2 pl-9 pr-3 text-sm text-heading">john_doe</div>
                  </div>
                  <div className="flex items-center gap-1.5 rounded-lg bg-white px-4 text-sm font-semibold text-black">
                    Search <Zap size={12} fill="currentColor" /> 2
                  </div>
                </div>
                <p className="text-center text-[12px]">
                  <span className="font-semibold text-heading">38 profiles found</span>
                  <span className="text-mute"> · 2.8s</span>
                </p>
                <div className="flex flex-wrap items-center gap-1.5 text-[11px]">
                  <span className="text-mute">Discovered:</span>
                  {["@johndoe99 · Reddit", "@jdoe_yt · YouTube"].map((c) => (
                    <span key={c} className="rounded-full border border-line bg-pill px-2.5 py-0.5 font-mono text-[10px] text-sub">{c}</span>
                  ))}
                  <span className="rounded-full border border-line px-2.5 py-0.5 text-[10px] text-mute">+ Hide</span>
                </div>
                <div className="flex items-center gap-2 border-t border-line pt-3 text-[11px]">
                  <div className="flex-1 truncate rounded-md border border-line bg-input px-2.5 py-1.5 text-mute">Filter results...</div>
                  <span className="flex items-center gap-1 rounded-md border border-line px-2 py-1.5 text-sub"><Fingerprint size={11} /> Filters</span>
                  <span className="text-mute">38 found</span>
                  <span className="ml-auto flex items-center gap-1 rounded-md border border-line px-2 py-1.5 text-sub">Export</span>
                </div>
                <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-3">
                  <HeroMiniCard platform="YOUTUBE" url="https://youtube.com" name="John Doe" handle="jdoe_yt" bio="tech reviews" lines={["24.8K followers · 166 Posts", "Joined Mar 18, 2017"]} more="+1 more field" />
                  <HeroMiniCard platform="GITHUB" url="https://github.com" name="john-doe" bio="backend engineer" lines={["567 Followers", "Location: San Francisco, CA", "Joined Jun 20, 2018"]} more="+2 more fields" />
                  <HeroMiniCard platform="TWITTER" url="https://x.com" name="John Doe" handle="john-doe" bio="SWE" lines={["3.8K followers · 2.1K following", "1.2K Posts · SF Bay Area"]} />
                  <HeroMiniCard platform="DISCORD" url="https://discord.com" name="john_doe#0001" bio="server admin" />
                  <HeroMiniCard platform="SPOTIFY" url="https://spotify.com" name="John Doe" bio="music lover" />
                  <HeroMiniCard platform="REDDIT" url="https://reddit.com" name="johndoe99" bio="r/just mod" />
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* -------------------------------------------------- HOW IT WORKS */}
      <section id="demo" className="border-t border-line py-24">
        <div className="mx-auto max-w-[1100px] px-6">
          <SectionHead center eyebrow="HOW IT WORKS" title="Three steps with zero setup" />
          <div className="mt-12 grid gap-4 md:grid-cols-3">
            {/* 01 */}
            <div className={card + " p-6"}>
              <p className="font-mono text-sm text-mute">01</p>
              <h3 className="mt-2 text-[16px] font-semibold text-heading">Pick your lookup</h3>
              <p className="mt-2 text-sm leading-relaxed text-sub">Username, email, or phone. Choose a search type and enter your query.</p>
              <div className="mt-5 rounded-lg border border-line bg-raised p-3">
                <div className="flex gap-1 rounded-md bg-input p-1">
                  {["Username", "Email", "Phone"].map((t, i) => (
                    <span key={t} className={`flex-1 rounded px-2 py-1 text-center text-[11px] ${i === 0 ? "bg-white font-semibold text-black" : "text-mute"}`}>{t}</span>
                  ))}
                </div>
                <div className="mt-2 flex items-center gap-2 rounded-md border border-line bg-input px-3 py-2">
                  <span className="flex-1 font-mono text-sm text-heading">johndoe</span>
                  <span className="grid size-6 place-items-center rounded bg-white text-black"><ArrowRight size={12} /></span>
                </div>
              </div>
            </div>
            {/* 02 */}
            <div className={card + " p-6"}>
              <p className="font-mono text-sm text-mute">02</p>
              <h3 className="mt-2 text-[16px] font-semibold text-heading">Watch results stream</h3>
              <p className="mt-2 text-sm leading-relaxed text-sub">Sources are checked in parallel. Hits appear the instant they're confirmed.</p>
              <div className="mt-5 space-y-1.5 rounded-lg border border-line bg-raised p-3 font-mono text-[11px]">
                {["github.com", "twitter.com", "reddit.com", "instagram.com"].map((d, i) => (
                  <div key={d} className="flex items-center justify-between rounded-md border border-line bg-pill px-2.5 py-1.5">
                    <span className="text-sub"><span className="mr-1.5 text-mute">{String(i + 1).padStart(2, "0")}</span>{d}</span>
                    <span className="font-semibold text-found">FOUND</span>
                  </div>
                ))}
              </div>
            </div>
            {/* 03 */}
            <div className={card + " p-6"}>
              <p className="font-mono text-sm text-mute">03</p>
              <h3 className="mt-2 text-[16px] font-semibold text-heading">Explore findings</h3>
              <p className="mt-2 text-sm leading-relaxed text-sub">Dig into profiles, cross-reference identities, filter and sort results.</p>
              <div className="mt-5 flex flex-wrap gap-1.5 rounded-lg border border-line bg-raised p-3">
                {[["xbox", 3], ["ebay", 2], ["reddit", 1], ["steam", 2], ["linkedin", 1], ["github", 2], ["twitter", 1], ["discord", 1], ["twitch", 1]].map(([n, c]) => (
                  <span key={n} className="rounded-full border border-line bg-pill px-2 py-0.5 font-mono text-[10px] text-sub">{n} <span className="text-heading">{c}</span></span>
                ))}
                <div className="mt-2 flex w-full flex-wrap items-center gap-1.5 border-t border-line pt-2">
                  {["✓ Verified", "Private", "Location"].map((t) => (
                    <span key={t} className="rounded-full border border-line px-2 py-0.5 text-[10px] text-sub">{t}</span>
                  ))}
                  <span className="ml-auto text-[10px] text-mute underline underline-offset-2">Clear all</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ----------------------------------------------------- CAPABILITIES */}
      <section id="features" className="border-t border-line py-24">
        <div className="mx-auto max-w-[1100px] px-6">
          <SectionHead
            eyebrow="CAPABILITIES"
            title="One query checks every platform in parallel"
            sub="Full profiles, linked accounts, and a complete search history. Filter, sort, and export your results."
          />
          <div className="mt-12 grid gap-4 md:grid-cols-3">
            {/* REAL-TIME */}
            <div className={card + " flex flex-col p-6"}>
              <Eyebrow>REAL-TIME</Eyebrow>
              <h3 className="mt-2 text-[16px] font-semibold text-heading">Results as they're found</h3>
              <p className="mt-2 text-sm leading-relaxed text-sub">Every platform checked independently. Results stream to your screen as they arrive.</p>
              <div className="mt-5 grid flex-1 grid-cols-3 gap-1.5">
                {Array.from({ length: 9 }).map((_, i) => (
                  <div key={i} className="rounded-md border border-line bg-raised p-1.5">
                    <div className="h-1 w-3/4 rounded bg-iconbox" />
                    <div className="mt-1.5 h-1 w-1/2 rounded bg-line" />
                    <div className="mt-1 h-1 w-2/3 rounded bg-line" />
                  </div>
                ))}
              </div>
              <p className="mt-4 border-t border-line pt-3 font-mono text-[11px] text-mute">9 results <span className="text-line">···</span> card view</p>
            </div>
            {/* DEEP SEARCH */}
            <div className={card + " flex flex-col p-6"}>
              <Eyebrow>DEEP SEARCH</Eyebrow>
              <h3 className="mt-2 text-[16px] font-semibold text-heading">Recursive discovery</h3>
              <p className="mt-2 text-sm leading-relaxed text-sub">New usernames and emails trigger fresh searches across 700+ platforms.</p>
              <div className="dotgrid mt-5 flex-1 space-y-2 rounded-lg border border-line p-3">
                {[["@johndoe", ""], ["@john.dev", "GitHub profile"], ["@jd.codes", "GitLab profile"], ["@jdoe_84", "Reddit profile"]].map(([h, s]) => (
                  <div key={h} className="flex items-center gap-2 rounded-full border border-line bg-pill px-3 py-1.5 text-[11px]">
                    <span className="font-mono text-heading">{h}</span>
                    {s && <span className="text-mute">{s}</span>}
                    <span className="ml-auto font-semibold text-heading">✓</span>
                  </div>
                ))}
              </div>
            </div>
            {/* COVERAGE */}
            <div className={card + " flex flex-col p-6"}>
              <Eyebrow>COVERAGE</Eyebrow>
              <h3 className="mt-2 text-[16px] font-semibold text-heading">700+ platforms</h3>
              <p className="mt-2 text-sm leading-relaxed text-sub">Social, gaming, dev tools, forums.</p>
              <div className="mt-5 flex-1 rounded-lg border border-line p-4">
                <div className="grid grid-cols-[repeat(24,1fr)] gap-[5px]">
                  {Array.from({ length: 144 }).map((_, i) => {
                    const hit = [3, 11, 18, 27, 34, 41, 52, 58, 66, 71, 79, 88, 95, 103, 111, 119, 127, 135].includes(i);
                    return <span key={i} className={`aspect-square rounded-[2px] ${hit ? "bg-white" : "bg-iconbox"}`} />;
                  })}
                </div>
                <div className="mt-4 flex items-center gap-4 font-mono text-[10px] text-mute">
                  <span><span className="text-white">■</span> hit</span>
                  <span><span className="text-iconbox">■</span> miss</span>
                  <span className="ml-auto">··· 45</span>
                </div>
              </div>
            </div>
            {/* VIEWS */}
            <div className={card + " flex flex-col p-6 md:col-span-1 lg:col-span-2"}>
              <div className="flex items-start justify-between gap-4">
                <div>
                  <Eyebrow>VIEWS</Eyebrow>
                  <h3 className="mt-2 text-[16px] font-semibold text-heading">Table &amp; card views</h3>
                  <p className="mt-2 max-w-sm text-sm leading-relaxed text-sub">Full profiles with followers, bios, and locations. Browse in table or card layout.</p>
                </div>
                <div className="flex rounded-md border border-line bg-input p-1">
                  {["TABLE", "CARDS"].map((t, i) => (
                    <span key={t} className={`flex items-center gap-1 rounded px-2.5 py-1 font-mono text-[10px] ${i === 1 ? "bg-white font-semibold text-black" : "text-mute"}`}>
                      {i === 0 ? <Table2 size={10} /> : <LayoutGrid size={10} />}{t}
                    </span>
                  ))}
                </div>
              </div>
              <div className="mt-5 grid flex-1 grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-6">
                {[
                  ["INSTAGRAM", "John", "3.1K followers", "#7a5a62"],
                  ["GITHUB", "johndev", "3.1K followers", "#5b6b7c"],
                  ["LINKEDIN", "John Doe", "500+ connections", "#6b6b6b"],
                  ["DRIBBBLE", "Jane · UI/UX", "898 Followers", "#7c6455"],
                  ["TWITTER", "@jdoe", "15.2K Fol · 842 Flw", "#5f6b5f"],
                  ["TELEGRAM", "@jdoe", "58 Followers", "#60656e"],
                ].map(([p, n, s, strip]) => (
                  <div key={p} className="overflow-hidden rounded-md border border-line bg-raised">
                    <div className="h-[6px]" style={{ background: strip }} />
                    <div className="p-2">
                      <p className="font-mono text-[9px] uppercase tracking-wider text-mute">{p}</p>
                      <p className="mt-0.5 truncate text-[11px] font-semibold text-heading">{n}</p>
                      <p className="truncate text-[10px] text-mute">{s}</p>
                      <p className="mt-1.5 text-[9px] text-mute">▸ +3 more fields</p>
                    </div>
                  </div>
                ))}
              </div>
            </div>
            {/* LOGGING */}
            <div className={card + " flex flex-col p-6"}>
              <Eyebrow>LOGGING</Eyebrow>
              <h3 className="mt-2 text-[16px] font-semibold text-heading">Full audit trail</h3>
              <p className="mt-2 text-sm leading-relaxed text-sub">Every query logged with full history.</p>
              <div className="mt-5 flex-1 overflow-hidden rounded-lg border border-line">
                <table className="w-full font-mono text-[10px]">
                  <thead>
                    <tr className="border-b border-line text-left text-mute">
                      {["QUERY", "TYPE", "#", "STATUS", "TIME"].map((h) => <th key={h} className="px-2.5 py-1.5 font-medium">{h}</th>)}
                    </tr>
                  </thead>
                  <tbody className="text-sub">
                    {[
                      ["johndoe", "username", "7", "2m ago"],
                      ["jdoe@mail.com", "email", "3", "14m ago"],
                      ["hunter2", "data", "42", "22m ago"],
                      ["+1 415 555…", "phone", "1", "38m ago"],
                    ].map(([q, t, n, tm]) => (
                      <tr key={q} className="border-b border-line/60">
                        <td className="max-w-[80px] truncate px-2.5 py-1.5 text-heading">{q}</td>
                        <td className="px-2.5 py-1.5">{t}</td>
                        <td className="px-2.5 py-1.5">{n}</td>
                        <td className="px-2.5 py-1.5 text-found">done</td>
                        <td className="px-2.5 py-1.5 text-mute">{tm}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <p className="mt-4 border-t border-line pt-3 font-mono text-[11px] text-mute">25 searches <span className="text-line">···</span> search history</p>
            </div>
          </div>
        </div>
      </section>

      {/* ----------------------------------------------------- FULL PROFILE */}
      <section className="border-t border-line py-24">
        <div className="mx-auto grid max-w-[1100px] items-center gap-12 px-6 lg:grid-cols-2">
          <div>
            <Eyebrow>FULL PROFILES</Eyebrow>
            <h2 className="mt-3 text-[32px] font-bold leading-tight tracking-tight text-heading sm:text-[38px]">
              Every match is a full profile. <span className="text-mute">Organized for review.</span>
            </h2>
            <p className="mt-5 text-[15px] leading-relaxed text-sub">
              Bio, banner, follower counts, pinned repos, joined dates, sponsored orgs. We extract every field a
              platform exposes and render it inline. Open the banner, expand the bio, dig into raw fields.
            </p>
            <p className="mt-5 font-mono text-xs text-mute">try expanding +more fields →</p>
          </div>
          <div className={card + " overflow-hidden"}>
            <div className="relative h-28">
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img src="/images/circuit-dark.jpg" alt="" className="h-full w-full object-cover" />
              <div className="absolute inset-0 bg-black/35" />
            </div>
            <div className="relative px-5 pb-5">
              <div className="absolute -top-8 left-5 grid size-16 place-items-center rounded-full border-2 border-card bg-iconbox">
                <Crosshair size={22} className="text-mute" />
              </div>
              <div className="pt-10">
                <div className="flex items-center gap-2">
                  {/* eslint-disable-next-line @next/next/no-img-element */}
                  <img src={faviconUrl("https://github.com")} alt="" width={14} height={14} loading="lazy" className="rounded-sm" />
                  <span className="font-mono text-[11px] uppercase tracking-wider text-sub">GITHUB ↗</span>
                  <span className="font-mono text-[11px] font-bold text-heading">✓</span>
                </div>
                <h3 className="mt-1.5 text-lg font-semibold text-heading">Michael Carter</h3>
                <p className="font-mono text-sm text-mute">@user</p>
                <p className="mt-3 text-sm leading-relaxed text-sub">
                  Backend engineer. Distributed systems, Go, and cold coffee. Building quiet tools for loud problems.
                  Ex-infrastructure at two unicorns — <span className="text-mute">now independently shipping.</span>
                </p>
                <button className="mt-1 text-[13px] text-mute underline underline-offset-2 hover:text-heading">Show more</button>
                <div className="mt-3 flex gap-4 font-mono text-xs text-sub">
                  <span><span className="text-heading">3.1K</span> Followers</span>
                  <span><span className="text-heading">184</span> Following</span>
                  <span><span className="text-heading">287</span> Posts</span>
                </div>
                <div className="mt-4 space-y-2 border-t border-line pt-4 text-[13px]">
                  {[
                    ["Location", "Portland, OR"],
                    ["Email", "user@example.com"],
                    ["Website", "example.com"],
                    ["Joined", "Mar 12, 2014"],
                    ["Last active", "Apr 22, 2025"],
                  ].map(([k, v]) => (
                    <div key={k} className="flex gap-2">
                      <span className="w-24 shrink-0 text-mute">{k}:</span>
                      <span className={k === "Website" ? "text-heading underline underline-offset-2" : "text-body"}>{v}</span>
                    </div>
                  ))}
                  <button className="flex items-center gap-1.5 pt-1 text-[13px] text-mute hover:text-sub">
                    <span className="text-[10px]">▶</span> +7 more fields
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* --------------------------------------------------------- SECURITY */}
      <section id="security" className="border-t border-line py-24">
        <div className="mx-auto max-w-[1100px] px-6">
          <div className="mx-auto max-w-2xl text-center">
            <Eyebrow>SECURITY</Eyebrow>
            <h2 className="mt-3 text-[32px] font-bold leading-tight tracking-tight text-heading sm:text-[38px]">Your searches stay protected</h2>
            <p className="mt-4 text-[15px] leading-relaxed text-sub">
              Results are pulled from public sources and processed transiently rather than written to our database.
              The query and result count are saved to your history; the findings are returned to your browser without
              creating a stored archive.
            </p>
          </div>
          <div className="mt-12 grid gap-4 sm:grid-cols-2">
            {[
              { icon: Lock, tag: "ACCOUNTS", t: "Credentials protected at rest", d: "When you use a password, it is stored only as a bcrypt hash. Session cookies are stored as one-way digests, never as reversible tokens." },
              { icon: Radar, tag: "OSINT", t: "Public data only", d: "We only surface publicly available information. No hacking, no breaching, no private endpoints — ever." },
              { icon: FileClock, tag: "AUDIT", t: "Full audit trail", d: "Every search is logged to a complete history you can review, pause, or wipe at any time from Settings." },
              { icon: ShieldCheck, tag: "PRIVACY", t: "Your data, your call", d: "We don't sell your personal information. Opt out, export, or delete your account whenever you want." },
            ].map(({ icon: Icon, tag, t, d }) => (
              <div key={tag} className={card + " p-6"}>
                <div className="flex items-center gap-3">
                  <span className="grid size-9 place-items-center rounded-lg border border-line bg-iconbox"><Icon size={16} className="text-sub" /></span>
                  <Eyebrow>{tag}</Eyebrow>
                </div>
                <h3 className="mt-4 text-[16px] font-semibold text-heading">{t}</h3>
                <p className="mt-2 text-sm leading-relaxed text-sub">{d}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* --------------------------------------------------------------- API */}
      <section id="api" className="border-t border-line py-24">
        <div className="mx-auto grid max-w-[1100px] items-center gap-12 px-6 lg:grid-cols-2">
          <div>
            <span className="inline-block rounded-full border border-linestrong px-3.5 py-1.5 font-mono text-[11px] tracking-[0.2em] text-sub">API COMING SOON</span>
            <h2 className="mt-5 text-[32px] font-bold leading-tight tracking-tight text-heading sm:text-[38px]">
              Fetch any platform with one GET.
            </h2>
            <p className="mt-5 text-[15px] leading-relaxed text-sub">
              Every platform we support sits behind a predictable URL: <span className="font-mono text-body">/v1/{"{platform}/{handle}"}</span>.
              Plain HTTP, JSON out. No SDK, no client library, no ceremony.
            </p>
            <ul className="mt-6 space-y-2.5">
              {["Same enriched profiles", "Predictable URLs", "Per-key auth with rate limits"].map((t) => (
                <li key={t} className="flex items-center gap-2.5 text-sm text-body">
                  <span className="font-mono font-bold text-heading">✓</span> {t}
                </li>
              ))}
            </ul>
            <div className="mt-8 flex flex-wrap items-center gap-4">
              <Link href="/pricing" className={btnOutline}>Join the API waitlist</Link>
              <span className="text-[13px] text-mute">We'll reply when keys are ready.</span>
            </div>
          </div>
          <div className="overflow-hidden rounded-xl border border-line bg-surface">
            <div className="flex items-center justify-between border-b border-line px-4">
              <div className="flex gap-4">
                {["Go", "Node.js", "Python", "Ruby", "cURL"].map((t, i) => (
                  <span key={t} className={`border-b py-2.5 text-[12px] ${i === 0 ? "border-white text-heading" : "border-transparent text-mute"}`}>{t}</span>
                ))}
              </div>
              <span className="font-mono text-[11px] text-mute">main.go</span>
            </div>
            <pre className="overflow-x-auto p-4 font-mono text-[12px] leading-6">
{[
  ["1",  "package main"],
  ["2",  ""],
  ["3",  'import ('],
  ["4",  '    "fmt"'],
  ["5",  '    "io"'],
  ["6",  '    "net/http"'],
  ["7",  ")"],
  ["8",  ""],
  ["9",  "func main() {"],
  ["10", '    req, _ := http.NewRequest("GET",'],
  ["11", '        "https://api.l96a1.local/v1/linkedin/johndoe", nil)'],
  ["12", '    req.Header.Set("Authorization",'],
  ["13", '        "Bearer fp_live_xxx")'],
  ["14", ""],
  ["15", "    res, _ := http.DefaultClient.Do(req)"],
  ["16", "    defer res.Body.Close()"],
  ["17", ""],
  ["18", "    body, _ := io.ReadAll(res.Body)"],
  ["19", "    fmt.Println(string(body))"],
  ["20", "}"],
].map(([n, code]) => (
  <div key={n} className="flex">
    <span className="w-8 shrink-0 text-right text-[#3a3a3a] select-none">{n}</span>
    <span className="pl-4 whitespace-pre text-sub">
                  {(code as string)
                    .split(/("[^"]*")/)
                    .map((part, i) =>
                      part.startsWith('"') ? (
                        <span key={i} className="text-body">{part}</span>
                      ) : (
                        <span key={i}>{part}</span>
                      )
                    )}
                </span>
  </div>
))}
            </pre>
          </div>
        </div>
      </section>

      {/* -------------------------------------------------------- QUOTES */}
      <section className="border-t border-line py-24">
        <div className="mx-auto max-w-[1100px] px-6">
          <SectionHead eyebrow="REAL USERS" title="What people are saying" />
          <div className="mt-12 grid gap-4 md:grid-cols-2">
            <figure className={card + " p-7"}>
              <p className="font-mono text-2xl text-mute">//</p>
              <blockquote className="mt-4 text-lg leading-relaxed text-body">
                Bruhhh, your OSINT tool is killer! Maybe one of the most comprehensive ones I've used yet!
              </blockquote>
              <figcaption className="mt-5 eyebrow">VIA REDDIT DM</figcaption>
            </figure>
            <figure className={card + " p-7"}>
              <p className="font-mono text-2xl text-mute">&quot;</p>
              <blockquote className="mt-4 text-lg leading-relaxed text-body">
                It is so refreshing to see someone get it &quot;right&quot; and release a tool that delivers results
                alongside the heavyweights I use at work.
              </blockquote>
              <figcaption className="mt-5 eyebrow">U/JASPER_NORTON ON R/OSINTTOOLS</figcaption>
            </figure>
          </div>
        </div>
      </section>

      {/* ------------------------------------------------------------- FAQ */}
      <section id="faq" className="border-t border-line py-24">
        <div className="mx-auto max-w-[820px] px-6">
          <SectionHead center eyebrow="FAQ" title="Frequently asked questions" />
          <div className="mt-12 divide-y divide-line rounded-xl border border-line bg-card">
            {[
              ["How does L96A1 work?", "Enter a username, email, or phone number. L96A1 fans the query out across 700+ platforms in parallel and streams each confirmed hit back the moment it resolves."],
              ["Is L96A1 a data broker?", "No. We do not buy, sell, or archive personal datasets. Results are pulled from public sources on demand and returned to your browser."],
              ["How do I remove my information?", "Use the Opt Out link in the footer. Verified removal requests are honored across our index and cache layers."],
              ["What platforms are searched?", "Social networks, gaming services, dev platforms, forums, marketplaces, music and streaming sites — 700+ and growing weekly."],
              ["How accurate are results?", "Every hit is verified against the platform's live profile page before it streams to your screen. Stale or ambiguous matches are flagged."],
              ["Is it free?", "Yes — the free plan includes 25 tokens a month across every search type. Paid plans raise the token ceiling and unlock Deep Search."],
              ["Does it store results?", "The query and result count are saved to your history. Profile findings are returned transiently without creating a stored archive."],
              ["What can I use it for?", "Legitimate OSINT work: security research, journalist investigations, brand protection, and auditing your own public footprint."],
              ["Do I need technical skills?", "None. Type a query, watch results stream. Filter, sort, and export in one click from the dashboard."],
            ].map(([q, a]) => (
              <details key={q} className="group px-6 py-4 open:pb-6">
                <summary className="flex cursor-pointer list-none items-center justify-between gap-4 text-[15px] font-medium text-heading [&::-webkit-details-marker]:hidden">
                  {q}
                  <span className="font-mono text-lg text-mute transition-transform group-open:rotate-45">+</span>
                </summary>
                <p className="mt-3 max-w-2xl text-sm leading-relaxed text-sub">{a}</p>
              </details>
            ))}
          </div>
        </div>
      </section>

      {/* ------------------------------------------------------------- CTA */}
      <section className="border-t border-line py-24">
        <div className="mx-auto max-w-[1100px] px-6">
          <div className="dotgrid rounded-2xl border border-line bg-card px-6 py-14 text-center sm:px-12">
            <div className="mx-auto w-fit"><L96A1Wordmark size="sm" /></div>
            <h2 className="mt-7 text-[30px] font-bold tracking-tight text-heading sm:text-[36px]">Pick a plan and start searching.</h2>
            <p className="mx-auto mt-3 max-w-md text-[15px] text-sub">Every plan covers the same 700+ platforms with live results.</p>
            <div className="mt-8 flex flex-wrap justify-center gap-3">
              <Link href="/dashboard/search" className={btnWhite}>Go to dashboard <ArrowRight size={15} /></Link>
              <Link href="/username-search" className={btnOutline}>Demo <span className="font-mono">⌕</span></Link>
            </div>
          </div>
        </div>
      </section>

      <SiteFooter />
    </div>
  );
}
