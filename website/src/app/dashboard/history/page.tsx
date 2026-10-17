"use client";

import { useEffect, useMemo, useState } from "react";
import { AtSign, Database, Download, FolderOpen, Layers, Search, Shield, Trash2 } from "lucide-react";
import { downloadFrees, type FreeNick } from "@/lib/available";

interface HistoryRow {
  query: string;
  type: "username" | "email" | "phone" | "people" | "breach" | "exposure" | "data-exposure" | "breach-username" | "bulk" | "dataset";
  results: number;
  status: string;
  time: string;
  /** Só disponíveis — o único conteúdo baixável da linha. */
  available: string[];
}

/** Reconstrói FreeNick[] a partir da linha compacta. */
function rowFrees(r: HistoryRow): FreeNick[] {
  if (r.type === "bulk" || r.type === "dataset") {
    return r.available.map((nick) => ({ nick, platform: "" }));
  }
  return r.available.map((platform) => ({ nick: r.query, platform }));
}

/** Começa vazio — o backend agora persiste em history.json e não reseta no boot. */
const INITIAL: HistoryRow[] = [];

const TYPE_META: Record<string, { label: string; icon: typeof AtSign; tone: string }> = {
  "data-exposure": { label: "data · exposure", icon: Shield, tone: "text-[#d9a13b]" },
  "breach-username": { label: "breach · username", icon: Database, tone: "text-[#d987a8]" },
  username: { label: "username", icon: AtSign, tone: "text-heading" },
  email: { label: "email", icon: AtSign, tone: "text-sub" },
  phone: { label: "phone", icon: AtSign, tone: "text-sub" },
  people: { label: "people", icon: AtSign, tone: "text-sub" },
  breach: { label: "breach", icon: Database, tone: "text-[#d987a8]" },
  exposure: { label: "exposure", icon: Shield, tone: "text-[#d9a13b]" },
  bulk: { label: "bulk", icon: Layers, tone: "text-heading" },
  dataset: { label: "dataset", icon: FolderOpen, tone: "text-heading" },
};

const FILTERS = ["All types", "Username", "Bulk", "Dataset", "Email", "Phone", "People", "Breach", "Exposure"];

function toRow(h: { query: string; queryType?: string; resultCount?: number; status?: string; time?: string; available?: string[] }): HistoryRow {
  const t = (h.queryType ?? "username").toLowerCase();
  const type = (["username", "email", "phone", "people", "breach", "exposure", "data-exposure", "breach-username", "bulk", "dataset"] as const).includes(t as HistoryRow["type"])
    ? (t as HistoryRow["type"])
    : "username";
  const available = Array.isArray(h.available) ? h.available : [];
  return {
    query: h.query,
    type,
    results: available.length,
    status: h.status ?? "completed",
    time: h.time ?? "just now",
    available,
  };
}

const TTL_OPTIONS = [
  { label: "30 min", value: 1800 },
  { label: "6 horas", value: 21600 },
  { label: "24 horas", value: 86400 },
  { label: "7 dias", value: 604800 },
  { label: "Só manual", value: 0 },
];

export default function HistoryPage() {
  const [rows, setRows] = useState(INITIAL);
  const [filter, setFilter] = useState("All types");
  const [text, setText] = useState("");
  const [enabled, setEnabled] = useState(true);
  const [checkedSize, setCheckedSize] = useState<number | null>(null);
  const [checkedTtl, setCheckedTtl] = useState<number>(86400);

  // Histórico real do motor (BFF /api/history); fallback mock se offline.
  useEffect(() => {
    fetch("/api/history")
      .then((r) => r.json())
      .then((data) => {
        if (Array.isArray(data?.history) && data.history.length > 0) {
          setRows(data.history.map(toRow));
        }
      })
      .catch(() => undefined);
    fetch("/api/l96a1/checked")
      .then((r) => r.json())
      .then((data) => {
        if (typeof data?.size === "number") setCheckedSize(data.size);
        if (typeof data?.ttl_secs === "number") setCheckedTtl(data.ttl_secs);
      })
      .catch(() => undefined);
  }, []);

  /** Apaga 1 linha (libera re-check do nick) ou tudo (limpa checados junto). */
  const deleteRow = async (query: string, type: HistoryRow["type"]) => {
    setRows((prev) => prev.filter((r) => r.query !== query));
    try {
      await fetch(`/api/history?query=${encodeURIComponent(query)}`, { method: "DELETE" });
    } catch {
      /* local já saiu */
    }
    if (type === "username") {
      try {
        await fetch(`/api/l96a1/checked?nick=${encodeURIComponent(query)}`, { method: "DELETE" });
        const r = await fetch("/api/l96a1/checked").then((x) => x.json()).catch(() => null);
        if (typeof r?.size === "number") setCheckedSize(r.size);
      } catch {
        /* ignora */
      }
    }
  };

  const clearAll = async () => {
    setRows([]);
    try {
      await fetch("/api/history", { method: "DELETE" });
      await fetch("/api/l96a1/checked", { method: "DELETE" });
      setCheckedSize(0);
    } catch {
      /* ignora */
    }
  };

  const clearChecked = async () => {
    try {
      await fetch("/api/l96a1/checked", { method: "DELETE" });
      setCheckedSize(0);
    } catch {
      /* ignora */
    }
  };

  const changeTtl = async (v: number) => {
    setCheckedTtl(v);
    try {
      await fetch("/api/l96a1/checked", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ ttl_secs: v }),
      });
    } catch {
      /* ignora */
    }
  };

  const visible = useMemo(() => {
    return rows.filter((r) => {
      if (filter !== "All types" && !TYPE_META[r.type].label.includes(filter.toLowerCase())) return false;
      if (text && !r.query.toLowerCase().includes(text.trim().toLowerCase())) return false;
      return true;
    });
  }, [rows, filter, text]);

  return (
    <div className="space-y-5">
      <div className="flex flex-wrap items-center justify-between gap-3 pt-2">
        <div>
          <h1 className="text-xl font-bold text-heading">History</h1>
          <p className="mt-0.5 text-sm text-mute">{rows.length} past searches</p>
        </div>
        <div className="flex items-center gap-3">
          <span className="text-[13px] text-mute">Search history</span>
          <button
            onClick={() => setEnabled(!enabled)}
            className="rounded-full border border-line bg-input px-0.5 py-0.5"
            role="switch"
            aria-checked={enabled}
            aria-label="Toggle search history"
          >
            <span className={`flex items-center gap-1 rounded-full px-2.5 py-1 text-[11px] font-semibold ${enabled ? "bg-white text-black" : ""}`}>On</span>
            <span className={`flex items-center gap-1 rounded-full px-2.5 py-1 text-[11px] font-semibold ${!enabled ? "bg-white text-black" : "text-mute"}`}>Off</span>
          </button>
          <button
            onClick={clearAll}
            className="flex items-center gap-1.5 rounded-lg border border-line px-3 py-2 text-[13px] text-sub transition-colors hover:border-linestrong hover:text-heading"
          >
            <Trash2 size={13} /> Clear history
          </button>
        </div>
      </div>

      {/* Já-checados (não-livres): pula re-check até expirar ou limpar */}
      <div className="flex flex-wrap items-center gap-2 rounded-xl border border-line bg-card px-4 py-3">
        <p className="text-[13px] text-sub">
          <span className="font-semibold text-heading">{checkedSize ?? "—"}</span> nicks já-checados (não-livres) salvos
          <span className="text-mute"> · apagar o history libera re-check</span>
        </p>
        <div className="ml-auto flex flex-wrap items-center gap-2">
          <label className="flex items-center gap-1.5 text-[12px] text-mute">
            limpar em
            <select
              value={checkedTtl}
              onChange={(e) => changeTtl(Number(e.target.value))}
              className="rounded-lg border border-line bg-input px-2 py-1.5 font-mono text-[12px] text-heading focus:border-linestrong focus:outline-none"
            >
              {TTL_OPTIONS.map((o) => (
                <option key={o.value} value={o.value}>{o.label}</option>
              ))}
            </select>
          </label>
          <button
            onClick={clearChecked}
            className="flex items-center gap-1.5 rounded-lg border border-line px-3 py-1.5 text-[12px] text-sub hover:border-linestrong hover:text-heading"
          >
            <Trash2 size={12} /> Limpar checados
          </button>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        {FILTERS.map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`rounded-full border px-3 py-1.5 text-[12px] transition-colors ${
              filter === f ? "border-white bg-white text-black" : "border-line text-sub hover:border-linestrong hover:text-heading"
            }`}
          >
            {f}
          </button>
        ))}
        <div className="relative ml-auto">
          <Search size={13} className="absolute top-1/2 left-3 -translate-y-1/2 text-mute" />
          <input
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder="Filter queries"
            className="w-48 rounded-lg border border-line bg-input py-2 pl-8 pr-3 text-[13px] text-heading placeholder:text-mute focus:border-linestrong focus:outline-none"
          />
        </div>
      </div>

      <div className={`overflow-x-auto rounded-xl border border-line bg-card ${enabled ? "" : "opacity-40"}`}>
        <table className="w-full min-w-[800px] text-sm">
          <thead>
            <tr className="border-b border-line text-left font-mono text-[10px] tracking-widest text-mute">
              {["QUERY", "TYPE", "LIVRES", "STATUS", "TIME", "BAIXAR", ""].map((h) => (
                <th key={h} className="px-4 py-3 font-medium">{h}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {visible.map((r, i) => {
              const meta = TYPE_META[r.type];
              const Icon = meta.icon;
              const frees = rowFrees(r);
              const cancelled = r.status.toLowerCase() === "cancelled";
              return (
                <tr key={i} className="border-b border-line/60 last:border-0 hover:bg-raised">
                  <td className="max-w-[220px] truncate px-4 py-3.5 font-mono text-[13px] text-heading">{r.query}</td>
                  <td className="px-4 py-3.5">
                    <span className={`flex items-center gap-1.5 font-mono text-[11px] ${meta.tone}`}>
                      <Icon size={12} /> {meta.label}
                    </span>
                  </td>
                  <td className="px-4 py-3.5 font-mono text-[13px]">
                    {r.results > 0 ? <span className="text-found">{r.results}</span> : <span className="text-mute">0</span>}
                  </td>
                  <td className={`px-4 py-3.5 font-mono text-[12px] ${cancelled ? "text-amber-300" : "text-found"}`}>{cancelled ? "Cancelled" : "Completed"}</td>
                  <td className="px-4 py-3.5 text-[13px] text-mute">{r.time}</td>
                  <td className="px-4 py-3.5">
                    {frees.length > 0 ? (
                      <span className="flex gap-1">
                        {(["txt", "md", "csv", "pdf"] as const).map((k) => (
                          <button
                            key={k}
                            onClick={() => downloadFrees(k, frees, r.query)}
                            title={`Baixar disponíveis (.${k})`}
                            className="flex items-center gap-1 rounded-md border border-line px-2 py-1 font-mono text-[10px] text-sub hover:border-linestrong hover:text-heading"
                          >
                            <Download size={10} />.{k}
                          </button>
                        ))}
                      </span>
                    ) : (
                      <span className="font-mono text-[11px] text-mute">—</span>
                    )}
                  </td>
                  <td className="px-4 py-3.5">
                    <button
                      onClick={() => deleteRow(r.query, r.type)}
                      title={`Apagar "${r.query}" (libera re-check)`}
                      className="grid size-7 place-items-center rounded-lg border border-line text-mute hover:border-danger hover:text-danger"
                    >
                      <Trash2 size={13} />
                    </button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
        {visible.length === 0 && (
          <p className="px-4 py-10 text-center text-sm text-mute">No searches match.</p>
        )}
      </div>
      {!enabled && (
        <p className="text-center font-mono text-[11px] tracking-widest text-mute">HISTORY PAUSED — NEW SEARCHES WON&apos;T BE LOGGED</p>
      )}
    </div>
  );
}
