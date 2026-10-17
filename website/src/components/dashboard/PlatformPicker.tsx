"use client";

import { useEffect, useState } from "react";

interface Plat {
  key: string;
  name: string;
  group: string;
}

/** Fallback se a API estiver fora: espelha l96a1_checkers::GROUPS. */
const GROUPS_FALLBACK = [
  "all", "social", "dev", "gaming", "streaming", "music", "community",
  "archived", "art", "blog", "business", "coding", "dating", "finance",
  "health", "hobby", "images", "misc", "news", "political", "shopping",
  "tech", "video", "xx NSFW xx",
];

/** Seletor input + chips clean (Fase 5). Espelha Bulk↔Social. */
export default function PlatformPicker({
  value,
  onChange,
}: {
  value: string;
  onChange: (v: string) => void;
}) {
  const [plats, setPlats] = useState<Plat[]>([]);
  const [groups, setGroups] = useState<string[]>(GROUPS_FALLBACK);
  const [showAll, setShowAll] = useState(false);

  useEffect(() => {
    fetch("/api/l96a1/platforms")
      .then((r) => r.json())
      .then((j) => {
        if (Array.isArray(j?.platforms)) setPlats(j.platforms);
        if (Array.isArray(j?.groups) && j.groups.length > 0)
          setGroups(["all", ...j.groups.filter((g: unknown) => typeof g === "string")]);
      })
      .catch(() => undefined);
  }, []);

  const parts = value
    .split(",")
    .map((s) => s.trim().toLowerCase())
    .filter(Boolean);

  const toggle = (tok: string) => {
    const t = tok.toLowerCase();
    if (t === "all") {
      onChange("all");
      return;
    }
    let next = parts.filter((p) => p !== "all");
    if (next.includes(t)) next = next.filter((p) => p !== t);
    else next = [...next, t];
    onChange(next.length === 0 ? "all" : next.join(","));
  };

  const active = (tok: string) => parts.includes(tok.toLowerCase()) || (tok === "all" && (parts.length === 0 || parts.includes("all")));

  return (
    <div className="space-y-2">
      <input
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="all | social | github,instagram"
        spellCheck={false}
        className="w-full rounded-lg border border-line bg-input px-3 py-2 font-mono text-[12px] text-heading placeholder:text-mute focus:border-linestrong focus:outline-none"
      />
      <div className="flex max-h-24 flex-wrap gap-1.5 overflow-y-auto">
        {groups.map((g) => (
          <button
            key={g}
            onClick={() => toggle(g)}
            className={`rounded-full border px-2.5 py-1 font-mono text-[11px] transition-colors ${
              active(g) ? "border-white bg-white text-black" : "border-line text-sub hover:border-linestrong hover:text-heading"
            }`}
          >
            {g}
          </button>
        ))}
        {plats.length > 0 && (
          <button
            onClick={() => setShowAll(!showAll)}
            className="rounded-full border border-line px-2.5 py-1 font-mono text-[11px] text-mute hover:text-sub"
          >
            {showAll ? "− nativas" : `+ nativas (${plats.length})`}
          </button>
        )}
      </div>
      {showAll && plats.length > 0 && (
        <div className="flex max-h-28 flex-wrap gap-1.5 overflow-y-auto rounded-lg border border-line bg-raised p-2">
          {plats.map((p) => (
            <button
              key={p.key}
              title={`${p.name} · ${p.group}`}
              onClick={() => toggle(p.key)}
              className={`rounded-full border px-2 py-0.5 font-mono text-[10.5px] transition-colors ${
                active(p.key) ? "border-white bg-white text-black" : "border-line text-sub hover:border-linestrong hover:text-heading"
              }`}
            >
              {p.key}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
