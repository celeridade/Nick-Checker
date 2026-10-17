"use client";

import { useEffect, useState } from "react";
import { Lock, X } from "lucide-react";
import { bannerFallback, faviconUrl, fmtCount, fmtDate, type SearchResult } from "@/lib/results";
import { applyRich, fetchEnrich } from "@/lib/enrich";

export default function ProfileModal({
  result,
  onClose,
}: {
  result: SearchResult | null;
  onClose: () => void;
}) {
  const [enriched, setEnriched] = useState<SearchResult | null>(null);
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  // Troca de resultado reseta o enrich durante o render (sem efeito).
  const [prevResult, setPrevResult] = useState(result);
  if (result !== prevResult) {
    setPrevResult(result);
    setEnriched(null);
  }

  // Enrich sob demanda (API separada — nunca bloqueia o scan).
  useEffect(() => {
    if (!result) return;
    if (result.pfpUrl && result.bannerUrl && result.bio) return;
    let cancelled = false;
    fetchEnrich(result.platform, result.username).then((rich) => {
      if (!cancelled && rich) setEnriched((prev) => (prev ? prev : applyRich(result, rich)));
    });
    return () => {
      cancelled = true;
    };
  }, [result]);

  if (!result) return null;
  const view = enriched ?? result;

  const stats = [
    view.followers !== null && `${fmtCount(view.followers)} Followers`,
    view.following !== null && `${fmtCount(view.following)} Following`,
    view.posts !== null && `${fmtCount(view.posts)} Posts`,
  ].filter(Boolean) as string[];

  const rows: Array<[string, string | null]> = [
    ["Location", view.location],
    ["Bio", view.bio],
    ["Website", view.website],
    ["Joined", fmtDate(view.createdAt)],
    ["Last active", fmtDate(view.lastActive)],
    ["Profile", view.profileUrl],
    ["Source", view.platformUrl],
    ["Private", view.isPrivate ? "Yes" : "No"],
    ["Fetch time", view.duration],
  ];

  const raw = JSON.stringify(
    {
      platform: view.platform,
      platformUrl: view.platformUrl,
      username: view.username,
      displayName: view.displayName,
      pfpUrl: view.pfpUrl,
      bannerUrl: view.bannerUrl,
      userId: view.userId,
      bio: view.bio,
      followers: view.followers,
      following: view.following,
      posts: view.posts,
      likes: view.likes,
      isVerified: view.isVerified,
      emails: view.emails,
      website: view.website,
      createdAt: view.createdAt,
      lastActive: view.lastActive,
      isPrivate: view.isPrivate,
      locked: view.locked,
      extras: view.extras,
      profileUrl: view.profileUrl,
      sourceQuery: view.sourceQuery,
      duration: view.duration,
    },
    null,
    2
  );

  const banner = view.bannerUrl ?? null;

  return (
    <div
      className="fixed inset-0 z-[60] flex items-center justify-center bg-black/85 p-4"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
      aria-label={`${view.platform} profile`}
    >
      <div
        className="card-enter max-h-[88vh] w-full max-w-xl overflow-y-auto rounded-xl border border-line bg-card"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="relative h-28 overflow-hidden">
          {/* Banner sólido sempre — nunca fica sem imagem */}
          <div className="absolute inset-0" style={{ backgroundColor: bannerFallback(view.platform) }} />
          {banner && (
            // eslint-disable-next-line @next/next/no-img-element
            <img
              src={banner}
              alt=""
              className="absolute inset-0 h-full w-full object-cover"
              onError={(e) => {
                (e.currentTarget as HTMLImageElement).remove();
              }}
            />
          )}
          <div className="absolute inset-0 bg-black/40" />
          <button
            onClick={onClose}
            aria-label="Close"
            className="absolute top-3 right-3 grid size-8 place-items-center rounded-lg border border-line bg-black/70 text-sub hover:text-heading"
          >
            <X size={15} />
          </button>
        </div>
        <div className="relative px-6 pb-6">
          <div className="absolute -top-9 left-6 grid size-[72px] place-items-center overflow-hidden rounded-full border-2 border-card bg-iconbox">
            {view.pfpUrl ? (
              // eslint-disable-next-line @next/next/no-img-element
              <img src={view.pfpUrl} alt={`@${view.username}`} className="size-full object-cover" loading="lazy" />
            ) : (
              <svg viewBox="0 0 24 24" className="mt-2 size-12 text-[#4a4a4a]" fill="currentColor" aria-hidden>
                <circle cx="12" cy="8.5" r="3.8" />
                <path d="M4.5 21c0-4.2 3.4-7 7.5-7s7.5 2.8 7.5 7v1.5h-15Z" />
              </svg>
            )}
          </div>
          <div className="pt-11">
            <div className="flex items-center gap-2">
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img src={faviconUrl(view.platformUrl)} alt="" width={14} height={14} loading="lazy" className="rounded-sm" />
              <span className="font-mono text-[11px] uppercase tracking-wider text-sub">{view.platform}</span>
              <span className="font-mono text-[11px] font-bold text-heading">✓ Verified</span>
              {view.locked && (
                <span className="ml-auto flex items-center gap-1 rounded-full border border-line px-2 py-0.5 font-mono text-[9px] tracking-widest text-mute">
                  <Lock size={9} /> LOCKED
                </span>
              )}
            </div>
            <h3 className="mt-1.5 text-xl font-bold text-heading">{view.displayName ?? view.username}</h3>
            <p className="font-mono text-sm text-mute">@{view.username}</p>

            {stats.length > 0 && (
              <div className="mt-3 flex gap-4 font-mono text-xs text-sub">
                {stats.map((s) => <span key={s}>{s}</span>)}
              </div>
            )}

            <div className="mt-4 space-y-2 border-t border-line pt-4 text-[13px]">
              {rows.filter(([, v]) => v).map(([k, v]) => (
                <div key={k} className="flex gap-2">
                  <span className="w-24 shrink-0 text-mute">{k}:</span>
                  {k === "Profile" && v?.startsWith("http") ? (
                    <a
                      href={v}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="truncate text-body underline underline-offset-2 hover:text-heading"
                      title="Abrir página onde o nick está registrado"
                    >
                      {v} ↗
                    </a>
                  ) : (
                    <span className="truncate text-body">{v}</span>
                  )}
                </div>
              ))}
            </div>

            {Object.keys(view.extras).length > 0 && (
              <div className="mt-4 rounded-lg border border-line bg-raised p-3">
                <p className="mb-2 flex items-center gap-1.5 font-mono text-[10px] tracking-widest text-mute">
                  <Lock size={9} /> MASKED FIELDS — UNLOCK WITH A PAID PLAN
                </p>
                <div className="grid grid-cols-2 gap-x-4 gap-y-1.5">
                  {Object.values(view.extras).map((v, i) => (
                    <p key={i} className="truncate font-mono text-[11px] text-sub">
                      {v.split(":")[0]} <span className="tracking-widest text-mute">••••••</span>
                    </p>
                  ))}
                </div>
              </div>
            )}

            <details className="mt-4 rounded-lg border border-line bg-surface">
              <summary className="cursor-pointer px-3 py-2 font-mono text-[11px] text-sub select-none">raw JSON</summary>
              <pre className="max-h-52 overflow-auto border-t border-line p-3 font-mono text-[10.5px] leading-5 text-mute">{raw}</pre>
            </details>
          </div>
        </div>
      </div>
    </div>
  );
}
