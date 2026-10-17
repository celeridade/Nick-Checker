"use client";

import { useCallback, useEffect, useState } from "react";
import { RefreshCw } from "lucide-react";

interface RouteInfo {
  key: string;
  name: string;
  group: string;
  route: string;
  concurrency: number;
}

export default function DoctorPanel() {
  const [data, setData] = useState<{
    version?: string;
    timeout_secs?: number;
    max_concurrent?: number;
    proxies?: { total: number; alive: number; file: string };
    platforms?: { total: number; routes: RouteInfo[] };
    dataset_dir?: string;
    github_token?: boolean;
    discord_tokens?: number;
    discord_tokens_health?: Array<{ id: string; state: string; retry_in_secs: number }>;
  } | null>(null);
  const [catalogs, setCatalogs] = useState<{
    handwritten?: number;
    generated?: number;
    total_indexed?: number;
  } | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Busca pura (sem setState): reusada pelo refresh manual e pelo mount.
  const fetchDoctor = useCallback(async () => {
    const [d, c] = await Promise.all([
      fetch("/api/l96a1/doctor").then((r) => r.json()),
      fetch("/api/l96a1/catalogs").then((r) => r.json()),
    ]);
    if (d?.error && !d?.version) throw new Error(d.error);
    return { d, c };
  }, []);

  const load = useCallback(async () => {
    setError(null);
    try {
      const { d, c } = await fetchDoctor();
      setData(d);
      setCatalogs(c);
    } catch (e) {
      setError(e instanceof Error ? e.message : "engine offline");
    }
  }, [fetchDoctor]);

  // Mount: assina os dados uma vez (setStates só após await + guarda alive).
  useEffect(() => {
    let alive = true;
    const run = async () => {
      try {
        const { d, c } = await fetchDoctor();
        if (!alive) return;
        setData(d);
        setCatalogs(c);
      } catch (e) {
        if (alive) setError(e instanceof Error ? e.message : "engine offline");
      }
    };
    void run();
    return () => {
      alive = false;
    };
  }, [fetchDoctor]);

  const routes = data?.platforms?.routes ?? [];
  const groups = new Map<string, RouteInfo[]>();
  for (const r of routes) {
    if (!groups.has(r.group)) groups.set(r.group, []);
    groups.get(r.group)!.push(r);
  }

  return (
    <div className="space-y-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-xl font-bold text-heading">Doctor</h1>
          <p className="mt-0.5 font-mono text-[12px] text-mute">
            v{data?.version ?? "—"} · timeout {data?.timeout_secs ?? "—"}s · conc {data?.max_concurrent ?? "—"}
          </p>
        </div>
        <button
          onClick={load}
          className="flex items-center gap-2 rounded-lg border border-line px-4 py-2 text-[13px] text-sub hover:border-linestrong hover:text-heading"
        >
          <RefreshCw size={13} /> Recarregar
        </button>
      </div>
      {error && <p className="text-[13px] text-danger">{error}</p>}

      <div className="grid grid-cols-2 gap-px overflow-hidden rounded-xl border border-line bg-line sm:grid-cols-4">
        {[
          ["Hand-written", String(catalogs?.handwritten ?? "—")],
          ["Geradas mf:/wm:", String(catalogs?.generated ?? "—")],
          ["Indexadas", String(catalogs?.total_indexed ?? "—")],
          ["Proxies vivos", data ? `${data.proxies?.alive}/${data.proxies?.total}` : "—"],
        ].map(([k, v]) => (
          <div key={k} className="bg-card px-4 py-3">
            <p className="font-mono text-[10px] tracking-widest text-mute uppercase">{k}</p>
            <p className="mt-1 font-mono text-lg font-bold text-heading">{v}</p>
          </div>
        ))}
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <div className="rounded-xl border border-line bg-card p-5">
          <h3 className="text-[15px] font-semibold text-heading">Tokens</h3>
          <div className="mt-3 space-y-2 font-mono text-[12px]">
            <p className="text-sub">GitHub: <span className={data?.github_token ? "text-found" : "text-mute"}>{data?.github_token ? "configurado" : "ausente"}</span></p>
            <p className="text-sub">Discord pool: <span className="text-heading">{data?.discord_tokens ?? "—"}</span></p>
            {(data?.discord_tokens_health?.length ?? 0) > 0 && (
              <div className="space-y-1 pt-1">
                {(data?.discord_tokens_health ?? []).map((t) => (
                  <p key={t.id} className="flex items-center justify-between gap-2 text-sub">
                    <span title="conta identificada pelos últimos 4 chars (sem segredo)">{t.id}</span>
                    {t.state === "ok" ? (
                      <span className="text-found">ok</span>
                    ) : t.state === "dead" ? (
                      <span className="text-danger">morto (401) — troque o token</span>
                    ) : (
                      <span className="text-amber-300">rate — volta em {Math.ceil(t.retry_in_secs / 60)}min</span>
                    )}
                  </p>
                ))}
              </div>
            )}
            <p className="text-sub">Dataset: <span className="text-heading">{data?.dataset_dir ?? "—"}</span></p>
          </div>
        </div>
        <div className="rounded-xl border border-line bg-card p-5">
          <h3 className="text-[15px] font-semibold text-heading">Matriz de rotas</h3>
          <p className="mt-0.5 text-[12px] text-mute">direct · sticky-proxy · pool/direct por plataforma</p>
          <div className="mt-3 max-h-64 space-y-3 overflow-y-auto">
            {[...groups.entries()].map(([g, list]) => (
              <div key={g}>
                <p className="font-mono text-[10px] tracking-widest text-mute uppercase">{g} ({list.length})</p>
                <div className="mt-1 flex flex-wrap gap-1.5">
                  {list.map((r) => (
                    <span key={r.key} title={`${r.name} · ${r.route} · conc ${r.concurrency}`} className="rounded-full border border-line bg-pill px-2 py-0.5 font-mono text-[10.5px] text-sub">
                      {r.key} <span className="text-mute">{r.route}</span>
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
