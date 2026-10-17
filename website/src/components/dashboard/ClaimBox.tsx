"use client";

import { useState } from "react";
import { Check, Download } from "lucide-react";
import { downloadFrees, type FreeNick } from "@/lib/available";

/** Disponíveis para claim (só Available entra aqui) + download txt/md/csv/pdf. */
export default function ClaimBox({ frees, title }: { frees: FreeNick[]; title: string }) {
  const [open, setOpen] = useState(false);
  const nicks = [...new Set(frees.map((f) => f.nick))].sort();
  if (nicks.length === 0) return null;

  const byNick = new Map<string, string[]>();
  for (const f of frees) {
    if (!byNick.has(f.nick)) byNick.set(f.nick, []);
    if (!byNick.get(f.nick)!.includes(f.platform)) byNick.get(f.nick)!.push(f.platform);
  }

  return (
    <div className="rounded-xl border border-found/40 bg-card p-4">
      <div className="flex flex-wrap items-center gap-2">
        <p className="flex items-center gap-1.5 text-sm font-semibold text-heading">
          <Check size={14} className="text-found" /> {nicks.length} livres para claim
        </p>
        <div className="ml-auto flex gap-1.5">
          <button
            onClick={() => setOpen(!open)}
            className="rounded-lg border border-line px-3 py-1.5 text-[12px] text-sub hover:border-linestrong hover:text-heading"
          >
            {open ? "Ocultar" : "Ver nicks"}
          </button>
          {(["txt", "md", "csv", "pdf"] as const).map((k) => (
            <button
              key={k}
              onClick={() => downloadFrees(k, frees, title)}
              className="flex items-center gap-1 rounded-lg border border-line px-2.5 py-1.5 font-mono text-[11px] text-sub hover:border-linestrong hover:text-heading"
            >
              <Download size={11} /> .{k}
            </button>
          ))}
        </div>
      </div>
      {open && (
        <div className="mt-3 flex max-h-56 flex-wrap gap-1.5 overflow-y-auto">
          {nicks.map((n) => (
            <span
              key={n}
              title={`livre em: ${(byNick.get(n) ?? []).join(", ")}`}
              className="rounded-full border border-line bg-pill px-2.5 py-1 font-mono text-[11px] text-heading"
            >
              @{n} <span className="text-mute">· {(byNick.get(n) ?? []).length}</span>
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
