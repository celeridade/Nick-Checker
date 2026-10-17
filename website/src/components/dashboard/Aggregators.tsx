"use client";

import { ChevronRight } from "lucide-react";
import { AGGREGATIONS, bannerFallback, type SearchResult } from "@/lib/results";

function AggCard({ title, count, children }: { title: string; count: number; children: React.ReactNode }) {
  return (
    <div className="rounded-xl border border-line bg-card p-4">
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-[13px] font-semibold text-heading">
          {title} <span className="font-mono text-[12px] text-mute">({count})</span>
        </h3>
        <button className="flex items-center gap-1 text-[11px] text-mute transition-colors hover:text-sub">
          <ChevronRight size={12} /> Expand
        </button>
      </div>
      {children}
    </div>
  );
}

/** CSS-drawn avatar tiles: silhouettes, initials, circuit crops, dot fields. */
function AvatarTile({ i }: { i: number }) {
  const kind = i % 6;
  if (kind === 3 || kind === 5) {
    return (
      // eslint-disable-next-line @next/next/no-img-element
      <img
        src="/images/circuit-dark.jpg"
        alt=""
        loading="lazy"
        className="size-9 rounded-full border border-line object-cover opacity-70"
        style={{ objectPosition: `${(i * 13) % 80}% ${(i * 29) % 80}%` }}
      />
    );
  }
  if (kind === 1 || kind === 4) {
    return (
      <span className="grid size-9 place-items-center rounded-full border border-line bg-iconbox font-mono text-[12px] font-bold text-sub">
        {["a", "X", "A", "j", "d", ""][i % 5]}
      </span>
    );
  }
  // gray silhouette fallback
  return (
    <span className="grid size-9 place-items-center overflow-hidden rounded-full border border-line bg-iconbox">
      <svg viewBox="0 0 24 24" className="mt-1.5 size-7 text-[#4a4a4a]" fill="currentColor" aria-hidden>
        <circle cx="12" cy="8.5" r="3.6" />
        <path d="M4.5 21c0-4.2 3.4-6.8 7.5-6.8s7.5 2.6 7.5 6.8v1.5h-15Z" />
      </svg>
    </span>
  );
}

function BannerThumb({ i, color }: { i: number; color?: string }) {
  // Placeholder sólido — nunca fica sem imagem.
  const palette = ["#1b1b22", "#1c222b", "#231d2b", "#1d2b23"];
  const bg = color ?? palette[i % palette.length];
  return <div className="aspect-[8/3] w-full rounded-lg border border-line" style={{ backgroundColor: bg }} />;
}

const chip = "rounded-full border border-line bg-raised px-2.5 py-1 font-mono text-[11px] text-sub";

export default function Aggregators({ results = [] }: { results?: SearchResult[] }) {
  const pfps = results.map((r) => r.pfpUrl).filter((u): u is string => !!u);
  const banners = results.map((r) => r.bannerUrl).filter((u): u is string => !!u);
  // Discord sem banner custom ainda tem banner sólido (blurple) — conta como banner.
  const hasDiscord = results.some((r) => r.platform.toLowerCase() === "discord");
  const picCount = Math.max(AGGREGATIONS.pictures.count, pfps.length || AGGREGATIONS.pictures.count);
  const bannerCount = Math.max(
    AGGREGATIONS.banners.count,
    banners.length + (hasDiscord && banners.length === 0 ? 1 : 0)
  );
  // Chips derivam dos resultados (antes hardcoded p/ adoecido).
  const uniq = (xs: (string | null | undefined)[]) => [...new Set(xs.filter((x): x is string => !!x && x.trim().length > 0))];
  const usernames = results.length > 0 ? uniq(results.map((r) => r.username)) : AGGREGATIONS.usernames;
  const names = results.length > 0 ? uniq(results.map((r) => r.displayName)) : AGGREGATIONS.names;
  const locations = results.length > 0 ? uniq(results.map((r) => r.location)) : AGGREGATIONS.locations;
  const emails = results.length > 0 ? uniq(results.flatMap((r) => r.emails ?? [])) : AGGREGATIONS.emails;
  return (
    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      <AggCard title="Profile pictures" count={picCount}>
        <div className="flex flex-wrap gap-2">
          {pfps.slice(0, 12).map((u) => (
            // eslint-disable-next-line @next/next/no-img-element
            <img key={u} src={u} alt="" loading="lazy" className="size-9 rounded-full border border-line object-cover" />
          ))}
          {Array.from({ length: Math.max(0, Math.min(12, 12 - Math.min(pfps.length, 12))) }).map((_, i) => (
            <AvatarTile key={`ph-${i}`} i={i} />
          ))}
        </div>
      </AggCard>
      <AggCard title="Banners" count={bannerCount}>
        <div className="grid grid-cols-2 gap-2">
          {banners.slice(0, 4).map((u) => (
            // eslint-disable-next-line @next/next/no-img-element
            <img key={u} src={u} alt="" loading="lazy" className="aspect-[8/3] w-full rounded-lg border border-line object-cover" />
          ))}
          {hasDiscord && banners.length === 0 && (
            <div className="aspect-[8/3] w-full rounded-lg border border-line" style={{ backgroundColor: bannerFallback("discord") }} />
          )}
          {Array.from({
            length: Math.max(
              0,
              4 - banners.slice(0, 4).length - (hasDiscord && banners.length === 0 ? 1 : 0)
            ),
          }).map((_, i) => (
            <BannerThumb key={`ph-${i}`} i={i} color={bannerFallback(results[i]?.platform ?? `ph-${i}`)} />
          ))}
        </div>
      </AggCard>
      <AggCard title="Usernames" count={usernames.length}>
        <div className="flex flex-wrap gap-1.5">{usernames.length > 0 ? usernames.map((u) => <span key={u} className={chip}>{u}</span>) : <span className={chip}>—</span>}</div>
      </AggCard>
      <AggCard title="Names" count={names.length}>
        <div className="flex flex-wrap gap-1.5">{names.length > 0 ? names.map((n) => <span key={n} className={chip}>{n}</span>) : <span className={chip}>—</span>}</div>
      </AggCard>
      <AggCard title="Locations" count={locations.length}>
        <div className="flex flex-wrap gap-1.5">{locations.length > 0 ? locations.map((l) => <span key={l} className={chip}>{l}</span>) : <span className={chip}>—</span>}</div>
      </AggCard>
      <AggCard title="Emails" count={emails.length}>
        <div className="flex flex-wrap gap-1.5">{emails.length > 0 ? emails.map((e) => <span key={e} className={chip}>{e}</span>) : <span className={chip}>—</span>}</div>
      </AggCard>
    </div>
  );
}
