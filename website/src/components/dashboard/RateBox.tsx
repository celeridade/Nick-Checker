"use client";

import { useState } from "react";
import { AlertTriangle, Download, Eye, EyeOff } from "lucide-react";
import type { WarnInfo } from "@/lib/results";
import { downloadFile } from "@/lib/export";

/** RateBox — espelho do ClaimBox para diagnóstico (Fase 4). */
type View = "hidden" | "pills" | "table";

export default function RateBox({ warns, title }: { warns: WarnInfo[]; title: string }) {
  // Estado único: "Ocultar" sempre recolhe tudo (antes a tabela ficava presa).
  const [view, setView] = useState<View>("hidden");
  const [revealed, setRevealed] = useState<Set<string>>(new Set());
  const [realProxy, setRealProxy] = useState<Map<string, string>>(new Map());

  if (warns.length === 0) return null;

  const rates = warns.filter((w) => w.reason === "rate").length;
  const errors = warns.filter((w) => w.reason === "error").length;
  const unknowns = warns.length - rates - errors;

  const toggleReveal = async (masked: string) => {
    if (revealed.has(masked)) {
      setRevealed((prev) => {
        const n = new Set(prev);
        n.delete(masked);
        return n;
      });
      return;
    }
    try {
      const r = await fetch(`/api/l96a1/proxies/unmask?url=${encodeURIComponent(masked)}`);
      const j = await r.json();
      if (r.ok && typeof j?.url === "string") {
        setRealProxy((prev) => new Map(prev).set(masked, j.url));
        setRevealed((prev) => new Set(prev).add(masked));
      }
    } catch {
      /* mantém mascarado */
    }
  };

  const doTxt = () => {
    const lines = [
      `L96A1 — rate/error/unknown (${warns.length})`,
      `scan: ${title} | exported: ${new Date().toISOString()}`,
      "=".repeat(60),
      ...warns.map(
        (w) => `@${w.nick}  ${w.platform}  ${w.reason}  evidence=${w.evidence || "-"}  proxy=${w.proxy}  ${w.latency_ms}ms`,
      ),
    ];
    downloadFile(`l96a1-warns-${Date.now()}.txt`, lines.join("\n"), "text/plain");
  };

  return (
    <div className="rounded-xl border border-amber-500/40 bg-card p-4">
      <div className="flex flex-wrap items-center gap-2">
        <p className="flex items-center gap-1.5 text-sm font-semibold text-heading">
          <AlertTriangle size={14} className="text-amber-400" /> {warns.length} avisos
          <span className="font-mono text-[11px] font-normal text-mute">
            {rates} rate · {errors} error · {unknowns} unknown
          </span>
        </p>
        <div className="ml-auto flex gap-1.5">
          {view === "hidden" ? (
            <button
              onClick={() => setView("pills")}
              className="rounded-lg border border-line px-3 py-1.5 text-[12px] text-sub hover:border-linestrong hover:text-heading"
            >
              Ver avisos
            </button>
          ) : (
            <button
              onClick={() => setView("hidden")}
              className="rounded-lg border border-line px-3 py-1.5 text-[12px] text-sub hover:border-linestrong hover:text-heading"
            >
              Ocultar
            </button>
          )}
          {view !== "hidden" && (
            <button
              onClick={() => setView(view === "table" ? "pills" : "table")}
              className="rounded-lg border border-line px-3 py-1.5 text-[12px] text-sub hover:border-linestrong hover:text-heading"
            >
              {view === "table" ? "Resumir" : "Detalhes"}
            </button>
          )}
          <button
            onClick={doTxt}
            className="flex items-center gap-1 rounded-lg border border-line px-2.5 py-1.5 font-mono text-[11px] text-sub hover:border-linestrong hover:text-heading"
          >
            <Download size={11} /> .txt
          </button>
        </div>
      </div>
      {view === "pills" && (
        <div className="mt-3 flex max-h-56 flex-wrap gap-1.5 overflow-y-auto">
          {warns.map((w, i) => (
            <span
              key={`${w.key}-${w.nick}-${i}`}
              title={`${w.reason} · evidence=${w.evidence || "-"} · proxy=${w.proxy}`}
              className="rounded-full border border-amber-500/30 bg-pill px-2.5 py-1 font-mono text-[11px] text-heading"
            >
              @{w.nick} <span className="text-mute">· {w.platform} · {w.reason}</span>
            </span>
          ))}
        </div>
      )}
      {view === "table" && (
        <div className="mt-3 overflow-x-auto rounded-lg border border-line">
          <table className="w-full min-w-[720px] text-[12px]">
            <thead>
              <tr className="border-b border-line text-left font-mono text-[10px] tracking-widest text-mute">
                {["NICK", "PLATFORM", "REASON", "EVIDENCE", "LAT", "PROXY", ""].map((h) => (
                  <th key={h} className="px-3 py-2 font-medium">{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {warns.map((w, i) => {
                const isRev = revealed.has(w.proxy);
                const shown = isRev ? realProxy.get(w.proxy) ?? w.proxy : w.proxy;
                return (
                  <tr key={`${w.key}-${w.nick}-${i}`} className="border-b border-line/60 last:border-0">
                    <td className="px-3 py-2 font-mono text-sub">@{w.nick}</td>
                    <td className="px-3 py-2 font-mono text-heading">{w.platform}</td>
                    <td className="px-3 py-2 font-mono text-amber-300">{w.reason}</td>
                    <td className="max-w-[220px] truncate px-3 py-2 font-mono text-mute" title={w.evidence}>{w.evidence || "—"}</td>
                    <td className="px-3 py-2 font-mono text-sub">{w.latency_ms}ms</td>
                    <td className="max-w-[220px] truncate px-3 py-2 font-mono text-sub" title={shown}>{shown}</td>
                    <td className="px-3 py-2">
                      <button
                        onClick={() => toggleReveal(w.proxy)}
                        aria-label={isRev ? "Mascarar proxy" : "Desmascarar proxy"}
                        className="grid size-7 place-items-center rounded-lg border border-line text-mute hover:text-heading"
                      >
                        {isRev ? <EyeOff size={13} /> : <Eye size={13} />}
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
