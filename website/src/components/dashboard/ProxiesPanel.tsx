"use client";

import { useCallback, useEffect, useState } from "react";
import { Activity, RefreshCw } from "lucide-react";

interface Ranked {
  url: string;
  protocol: string;
  latency_ms: number;
  success: number;
  fails: number;
  healthy: boolean;
  targets_ok?: string;
  dead?: boolean;
}

const TARGETS = [
  { id: "http://gstatic.com/generate_204", label: "gstatic" },
  { id: "https://discord.com/", label: "discord" },
  { id: "https://github.com/", label: "github" },
];

export default function ProxiesPanel() {
  const [total, setTotal] = useState(0);
  const [alive, setAlive] = useState(0);
  const [file, setFile] = useState("");
  const [ranked, setRanked] = useState<Ranked[]>([]);
  const [testing, setTesting] = useState(false);
  const [pruning, setPruning] = useState(false);
  const [pruneMsg, setPruneMsg] = useState<string | null>(null);
  const [targets, setTargets] = useState<string[]>(TARGETS.map((t) => t.id));
  const [error, setError] = useState<string | null>(null);

  // Busca pura (sem setState): reusada pelo health-check e pelo mount.
  const fetchProxies = useCallback(async () => {
    const res = await fetch("/api/l96a1/proxies");
    const data = await res.json();
    return {
      total: data.total ?? 0,
      alive: data.alive ?? 0,
      file: data.file ?? "",
    };
  }, []);

  const load = useCallback(async () => {
    try {
      const p = await fetchProxies();
      setTotal(p.total);
      setAlive(p.alive);
      setFile(p.file);
    } catch {
      /* offline */
    }
  }, [fetchProxies]);

  // Mount: assina o pool uma vez (setStates só após await + guarda alive).
  useEffect(() => {
    let alive = true;
    const run = async () => {
      try {
        const p = await fetchProxies();
        if (!alive) return;
        setTotal(p.total);
        setAlive(p.alive);
        setFile(p.file);
      } catch {
        /* offline */
      }
    };
    void run();
    return () => {
      alive = false;
    };
  }, [fetchProxies]);

  const test = async () => {
    setTesting(true);
    setError(null);
    setPruneMsg(null);
    try {
      const res = await fetch("/api/l96a1/proxies/test", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ targets }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data?.error ?? `HTTP ${res.status}`);
      setRanked(data.ranked ?? []);
      load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "falha no health-check");
    } finally {
      setTesting(false);
    }
  };

  const prune = async () => {
    if (!window.confirm("Remover do proxies.txt SÓ os 100% mortos (flaky preservados)? Backup .bak automático.")) return;
    setPruning(true);
    setPruneMsg(null);
    try {
      const res = await fetch("/api/l96a1/proxies/prune", { method: "POST" });
      const data = await res.json();
      if (!res.ok) throw new Error(data?.error ?? `HTTP ${res.status}`);
      setPruneMsg(`✓ ${data.removed ?? 0} removidos, ${data.kept ?? "?"} mantidos (backup ${data.backup ?? "—"})`);
      load();
    } catch (e) {
      setPruneMsg(e instanceof Error ? e.message : "prune falhou");
    } finally {
      setPruning(false);
    }
  };

  const toggleTarget = (id: string) =>
    setTargets((prev) => (prev.includes(id) ? prev.filter((t) => t !== id) : [...prev, id]));

  return (
    <div className="space-y-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-xl font-bold text-heading">Proxies</h1>
          <p className="mt-0.5 font-mono text-[12px] text-mute">{file || "pool"} · cooldown por par (proxy, plataforma)</p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <div className="flex gap-1.5">
            {TARGETS.map((t) => (
              <button
                key={t.id}
                onClick={() => toggleTarget(t.id)}
                className={`rounded-full border px-2.5 py-1 font-mono text-[11px] transition-colors ${
                  targets.includes(t.id) ? "border-white bg-white text-black" : "border-line text-mute hover:text-sub"
                }`}
              >
                {t.label}
              </button>
            ))}
          </div>
          <button
            onClick={test}
            disabled={testing || targets.length === 0}
            className="flex items-center gap-2 rounded-lg bg-white px-4 py-2 text-[13px] font-semibold text-black hover:bg-body disabled:opacity-50"
          >
            <RefreshCw size={13} className={testing ? "animate-spin" : ""} />
            {testing ? "Testando…" : "Health-check"}
          </button>
          <button
            onClick={prune}
            disabled={pruning || testing}
            className="flex items-center gap-2 rounded-lg border border-danger px-4 py-2 text-[13px] font-semibold text-danger hover:bg-danger/10 disabled:opacity-50"
          >
            {pruning ? "Removendo…" : "Remover bad"}
          </button>
        </div>
      </div>
      {pruneMsg && <p className="text-[13px] text-sub">{pruneMsg}</p>}

      <div className="grid grid-cols-2 gap-px overflow-hidden rounded-xl border border-line bg-line sm:grid-cols-3">
        {[
          ["Pool total", String(total)],
          ["Vivos", String(alive)],
          ["Rota direta", total === 0 ? "ativa" : "fallback"],
        ].map(([k, v]) => (
          <div key={k} className="bg-card px-4 py-3">
            <p className="font-mono text-[10px] tracking-widest text-mute uppercase">{k}</p>
            <p className="mt-1 font-mono text-lg font-bold text-heading">{v}</p>
          </div>
        ))}
      </div>
      {error && <p className="text-[13px] text-danger">{error}</p>}

      {ranked.length > 0 && (
        <div className="overflow-x-auto rounded-xl border border-line bg-card">
          <table className="w-full min-w-[560px] text-sm">
            <thead>
              <tr className="border-b border-line text-left font-mono text-[10px] tracking-widest text-mute">
                {["PROXY", "PROTO", "LATÊNCIA", "ALVOS", "OK", "FALHAS", "SAÚDE"].map((h) => (
                  <th key={h} className="px-4 py-3 font-medium">{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {ranked.map((r, i) => (
                <tr key={`${r.url}#${i}`} className={`border-b border-line/60 last:border-0 hover:bg-raised ${r.dead ? "opacity-60" : ""}`}>
                  <td className="max-w-[260px] truncate px-4 py-2.5 font-mono text-[12px] text-heading">{r.url}</td>
                  <td className="px-4 py-2.5 font-mono text-[12px] text-sub">{r.protocol}</td>
                  <td className="px-4 py-2.5 font-mono text-[12px] text-sub">{Math.round(r.latency_ms)}ms</td>
                  <td className="px-4 py-2.5 font-mono text-[12px] text-sub">{r.targets_ok ?? "—"}</td>
                  <td className="px-4 py-2.5 font-mono text-[12px] text-sub">{r.success}</td>
                  <td className="px-4 py-2.5 font-mono text-[12px] text-sub">{r.fails}</td>
                  <td className="px-4 py-2.5 font-mono text-[12px]">
                    {r.dead
                      ? <span className="text-danger">bad</span>
                      : r.healthy
                        ? <span className="flex items-center gap-1 text-found"><Activity size={11} /> ok</span>
                        : <span className="text-amber-300">flaky</span>}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
