"use client";

import { useCallback, useEffect, useState } from "react";
import { Crosshair, Play, Plus, Trash2 } from "lucide-react";

export default function SnipePanel() {
  const [watchlist, setWatchlist] = useState<string[]>([]);
  const [nick, setNick] = useState("");
  const [status, setStatus] = useState<{
    running?: boolean;
    last_run?: string | null;
    next_in_secs?: number;
    watchlist_size?: number;
    cycles?: number;
    hits_total?: number;
    last_hits?: string[];
  } | null>(null);
  const [config, setConfig] = useState<{ interval_secs?: number; platforms?: string; webhook_configured?: boolean } | null>(null);
  const [webhook, setWebhook] = useState<{ url_configured?: boolean; mode?: string } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  // Busca pura (sem setState): reusada por edit/run-once e pelo polling.
  const fetchSnipe = useCallback(async () => {
    const [s, w, wh] = await Promise.all([
      fetch("/api/l96a1/snipe/status").then((r) => r.json()),
      fetch("/api/l96a1/snipe/watchlist").then((r) => r.json()),
      fetch("/api/l96a1/webhooks").then((r) => r.json()),
    ]);
    return {
      status: s.status ?? null,
      config: s.config ?? null,
      watchlist: w.watchlist ?? [],
      webhook: wh,
    };
  }, []);

  const load = useCallback(async () => {
    try {
      const d = await fetchSnipe();
      setStatus(d.status);
      setConfig(d.config);
      setWatchlist(d.watchlist);
      setWebhook(d.webhook);
    } catch {
      setError("engine offline");
    }
  }, [fetchSnipe]);

  // Polling 10s: cada tick é uma assinatura (setStates só após await + alive).
  useEffect(() => {
    let alive = true;
    const tick = async () => {
      try {
        const d = await fetchSnipe();
        if (!alive) return;
        setStatus(d.status);
        setConfig(d.config);
        setWatchlist(d.watchlist);
        setWebhook(d.webhook);
      } catch {
        if (alive) setError("engine offline");
      }
    };
    void tick();
    const t = setInterval(() => void tick(), 10000);
    return () => {
      alive = false;
      clearInterval(t);
    };
  }, [fetchSnipe]);

  const edit = async (action: "add" | "remove", n: string) => {
    setError(null);
    try {
      const res = await fetch("/api/l96a1/snipe/watchlist", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ action, nick: n }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data?.error ?? `HTTP ${res.status}`);
      setWatchlist(data.watchlist ?? []);
      setNick("");
    } catch (e) {
      setError(e instanceof Error ? e.message : "falha");
    }
  };

  const runOnce = async () => {
    setBusy(true);
    setError(null);
    try {
      const res = await fetch("/api/l96a1/snipe/run-once", { method: "POST" });
      const data = await res.json();
      if (!res.ok) throw new Error(data?.error ?? `HTTP ${res.status}`);
      setTimeout(load, 2000);
    } catch (e) {
      setError(e instanceof Error ? e.message : "falha");
    } finally {
      setBusy(false);
    }
  };

  const testWebhook = async () => {
    try {
      const res = await fetch("/api/l96a1/webhooks/test", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({}),
      });
      const data = await res.json();
      if (!data.sent) throw new Error(data?.error ?? "não enviado");
    } catch (e) {
      setError(e instanceof Error ? e.message : "webhook falhou");
    }
  };

  return (
    <div className="space-y-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-xl font-bold text-heading">Sniper 24h</h1>
          <p className="mt-0.5 text-sm text-mute">
            Observa a watchlist a cada {config?.interval_secs ?? "—"}s e notifica liberados no webhook
            {webhook?.url_configured ? "" : " (webhook ausente)"}.
          </p>
        </div>
        <button
          onClick={runOnce}
          disabled={busy || status?.running}
          className="flex items-center gap-2 rounded-lg bg-white px-4 py-2 text-[13px] font-semibold text-black hover:bg-body disabled:opacity-50"
        >
          <Play size={13} /> {status?.running ? "Rodando…" : "Rodar agora"}
        </button>
      </div>

      <div className="grid grid-cols-2 gap-px overflow-hidden rounded-xl border border-line bg-line sm:grid-cols-4">
        {[
          ["Alvos", String(status?.watchlist_size ?? watchlist.length)],
          ["Ciclos", String(status?.cycles ?? 0)],
          ["Liberados", String(status?.hits_total ?? 0)],
          ["Último ciclo", status?.last_run ? new Date(Number(status.last_run) * 1000).toLocaleString() : "—"],
        ].map(([k, v]) => (
          <div key={k} className="bg-card px-4 py-3">
            <p className="font-mono text-[10px] tracking-widest text-mute uppercase">{k}</p>
            <p className="mt-1 truncate font-mono text-lg font-bold text-heading">{v}</p>
          </div>
        ))}
      </div>

      {(status?.last_hits?.length ?? 0) > 0 && (
        <div className="rounded-xl border border-found/40 bg-card p-4">
          <p className="font-mono text-[11px] tracking-widest text-found">ÚLTIMOS LIBERADOS</p>
          <div className="mt-2 flex flex-wrap gap-1.5">
            {status!.last_hits!.map((h) => (
              <span key={h} className="rounded-full border border-line bg-pill px-2.5 py-0.5 font-mono text-[11px] text-heading">{h}</span>
            ))}
          </div>
        </div>
      )}

      <div className="rounded-xl border border-line bg-card p-4">
        <div className="flex gap-2">
          <div className="relative flex-1">
            <Crosshair size={13} className="absolute top-1/2 left-3 -translate-y-1/2 text-mute" />
            <input
              value={nick}
              onChange={(e) => setNick(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && nick.trim() && edit("add", nick)}
              placeholder="nick para observar…"
              className="w-full rounded-lg border border-line bg-input py-2 pl-8 pr-3 font-mono text-[13px] text-heading placeholder:text-mute focus:border-linestrong focus:outline-none"
            />
          </div>
          <button
            onClick={() => nick.trim() && edit("add", nick)}
            className="flex items-center gap-1.5 rounded-lg bg-white px-4 py-2 text-[13px] font-semibold text-black hover:bg-body"
          >
            <Plus size={13} /> Observar
          </button>
          <button onClick={testWebhook} className="rounded-lg border border-line px-4 py-2 text-[13px] text-sub hover:border-linestrong hover:text-heading">
            Testar webhook
          </button>
        </div>
        {error && <p className="mt-2 text-[13px] text-danger">{error}</p>}
        <div className="mt-3 flex flex-wrap gap-1.5">
          {watchlist.map((u) => (
            <span key={u} className="flex items-center gap-1.5 rounded-full border border-line bg-pill px-2.5 py-1 font-mono text-[11px] text-sub">
              {u}
              <button onClick={() => edit("remove", u)} aria-label={`Remover ${u}`} className="text-mute hover:text-danger">
                <Trash2 size={11} />
              </button>
            </span>
          ))}
          {watchlist.length === 0 && <p className="text-[13px] text-mute">Watchlist vazia — adicione o primeiro alvo acima.</p>}
        </div>
      </div>
    </div>
  );
}
