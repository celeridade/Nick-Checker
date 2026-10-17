"use client";

import { useState } from "react";
import { ArrowUpRight, Lock } from "lucide-react";
import { faviconUrl, fmtCount, fmtDate, type SearchResult } from "@/lib/results";

export default function ResultCard({
  result,
  flash,
  onOpen,
}: {
  result: SearchResult;
  flash?: boolean;
  onOpen: (r: SearchResult) => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const disabled = !result.hasChecker;
  const extraCount = Object.keys(result.extras).length;
  const dateBits = [
    result.createdAt && `Joined: ${fmtDate(result.createdAt)}`,
    result.lastActive && `Last active: ${fmtDate(result.lastActive)}`,
  ].filter(Boolean) as string[];

  return (
    <div
      onClick={() => !disabled && onOpen(result)}
      className={`card-enter relative overflow-hidden rounded-xl border border-line bg-card p-4 text-left transition-colors ${
        disabled ? "cursor-not-allowed opacity-45" : "cursor-pointer hover:border-linestrong"
      }`}
    >
      {flash && (
        <div className="found-flash pointer-events-none absolute inset-0 z-10 flex items-center justify-center bg-white/10">
          <span className="rounded-full bg-white px-3 py-1 font-mono text-[11px] font-bold tracking-widest text-black">FOUND</span>
        </div>
      )}

      <div className="flex items-center gap-2">
        {/* eslint-disable-next-line @next/next/no-img-element */}
        <img
          src={faviconUrl(result.platformUrl)}
          alt=""
          width={16}
          height={16}
          loading="lazy"
          className="size-4 rounded-sm bg-iconbox"
        />
        {disabled ? (
          <span className="font-mono text-[11px] font-semibold tracking-wider text-heading uppercase">
            {result.platform}
          </span>
        ) : (
          <a
            href={result.profileUrl}
            target="_blank"
            rel="noopener noreferrer"
            onClick={(e) => e.stopPropagation()}
            className="flex items-center gap-1 font-mono text-[11px] font-semibold tracking-wider text-heading uppercase hover:underline underline-offset-2"
            title={`Abrir ${result.platform} de @${result.username} (estilo fingerprint.to)`}
          >
            {result.platform}
            <ArrowUpRight size={12} className="text-mute" />
          </a>
        )}
        {!disabled && <span className="sr-only">{result.profileUrl}</span>}
        {result.locked && <Lock size={11} className="text-mute" />}
        {result.cached && (
          <span title="Veio do cache de já-checados (sem rede nesta passada)" className="rounded-full border border-line px-2 py-0.5 font-mono text-[9px] tracking-widest text-mute">CACHED</span>
        )}
        {disabled && (
          <span className="ml-auto rounded-full border border-line px-2 py-0.5 font-mono text-[9px] tracking-widest text-mute">SOON</span>
        )}
      </div>

      <div className="mt-2.5 min-h-[38px]">
        {result.displayName && (
          <p className="truncate text-[14px] font-semibold text-heading">{result.displayName}</p>
        )}
        <p className={`truncate font-mono text-[12px] ${result.displayName ? "text-mute" : "text-[13px] font-semibold text-heading"}`}>
          @{result.username}
        </p>
      </div>

      <div className="mt-1 flex flex-wrap gap-x-3 gap-y-0.5 font-mono text-[11px] text-sub">
        {result.followers !== null && <span>{fmtCount(result.followers)} Followers</span>}
        {result.following !== null && <span>{fmtCount(result.following)} Following</span>}
        {result.posts !== null && <span>{fmtCount(result.posts)} Posts</span>}
      </div>

      <div className="mt-1.5 space-y-0.5 text-[11px] text-mute">
        {dateBits.map((d) => <p key={d}>{d}</p>)}
        {result.location && <p>Location: {result.location}</p>}
      </div>

      {extraCount > 0 && (
        <div
          className="mt-3 flex items-center gap-1.5 border-t border-line pt-2.5 text-[11px] text-mute"
          onClick={(e) => {
            e.stopPropagation();
            if (!disabled) setExpanded((v) => !v);
          }}
        >
          <Lock size={10} /> +{extraCount} more field{extraCount === 1 ? "" : "s"}
          <span className="ml-auto">{expanded ? "▾" : "▸"}</span>
        </div>
      )}

      {expanded && (
        <div className="mt-2.5 space-y-1.5 rounded-lg border border-line bg-raised p-2.5">
          {Object.values(result.extras).map((v, i) => (
            <p key={i} className="font-mono text-[10px] text-sub">
              {v.split(":")[0]}: <span className="tracking-widest text-mute">••••••••</span>
            </p>
          ))}
        </div>
      )}
    </div>
  );
}
