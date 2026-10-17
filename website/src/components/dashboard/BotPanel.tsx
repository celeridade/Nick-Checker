"use client";

import { useCallback, useEffect, useState } from "react";
import { Bot, Hammer } from "lucide-react";

export default function BotPanel() {
  const [status, setStatus] = useState<{
    token_configured?: boolean;
    guild_id?: number;
    prefix?: string;
    captcha?: boolean;
    commands?: string[];
    note?: string;
    last_setup?: { ran_at: string; verify_channel_id: number; channels: number; log: string[] } | null;
  } | null>(null);
  const [guild, setGuild] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [ok, setOk] = useState<string | null>(null);

  // Busca pura (sem setState): reusada pelo setup e pelo mount.
  const fetchStatus = useCallback(async () => {
    const res = await fetch("/api/l96a1/bot/status");
    return res.json();
  }, []);

  const load = useCallback(async () => {
    try {
      setStatus(await fetchStatus());
    } catch {
      setError("engine offline");
    }
  }, [fetchStatus]);

  // Mount: assina o status uma vez (setState só após await + guarda alive).
  useEffect(() => {
    let alive = true;
    const run = async () => {
      try {
        const s = await fetchStatus();
        if (alive) setStatus(s);
      } catch {
        if (alive) setError("engine offline");
      }
    };
    void run();
    return () => {
      alive = false;
    };
  }, [fetchStatus]);

  const setup = async () => {
    setBusy(true);
    setError(null);
    setOk(null);
    try {
      const res = await fetch("/api/l96a1/bot/setup", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(guild.trim() ? { guild_id: Number(guild) } : {}),
      });
      const data = await res.json();
      if (!res.ok || !data.ok) throw new Error(data?.error ?? `HTTP ${res.status}`);
      setOk(`${data.summary.channels} canais + verify <#${data.summary.verify_channel_id}>`);
      load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "setup falhou");
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-bold text-heading">Bot Discord</h1>
        <p className="mt-0.5 text-sm text-mute">Setup do servidor via web · gateway 24h como sidecar · captcha próprio</p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <div className="rounded-xl border border-line bg-card p-5">
          <p className="flex items-center gap-2 text-[15px] font-semibold text-heading"><Bot size={15} /> Status</p>
          <div className="mt-3 space-y-2 font-mono text-[12px] text-sub">
            <p>Token: <span className={status?.token_configured ? "text-found" : "text-danger"}>{status?.token_configured ? "configurado" : "ausente (BOT_TOKEN)"}</span></p>
            <p>Guild: <span className="text-heading">{status?.guild_id || "—"}</span></p>
            <p>Prefixo: <span className="text-heading">{status?.prefix || "!"}</span> · captcha: <span className="text-heading">{status?.captcha ? "on" : "off"}</span></p>
            {status?.last_setup && (
              <p>Último setup: <span className="text-heading">{status.last_setup.channels} canais</span></p>
            )}
          </div>
          <div className="mt-4 flex gap-2">
            <input
              value={guild}
              onChange={(e) => setGuild(e.target.value)}
              placeholder="guild_id (ou usa env)"
              className="w-full rounded-lg border border-line bg-input px-3 py-2 font-mono text-[12px] text-heading placeholder:text-mute focus:border-linestrong focus:outline-none"
            />
            <button
              onClick={setup}
              disabled={busy}
              className="flex shrink-0 items-center gap-1.5 rounded-lg bg-white px-4 py-2 text-[13px] font-semibold text-black hover:bg-body disabled:opacity-50"
            >
              <Hammer size={13} /> {busy ? "Criando…" : "Setup"}
            </button>
          </div>
          {error && <p className="mt-2 text-[13px] text-danger">{error}</p>}
          {ok && <p className="mt-2 text-[13px] text-found">✓ {ok}</p>}
          {status?.last_setup && (
            <div className="mt-3 max-h-40 space-y-1 overflow-y-auto rounded-lg border border-line bg-raised p-2.5 font-mono text-[11px] text-sub">
              {status.last_setup.log.map((l, i) => <p key={i}>{l}</p>)}
            </div>
          )}
        </div>

        <div className="rounded-xl border border-line bg-card p-5">
          <h3 className="text-[15px] font-semibold text-heading">Comandos</h3>
          <p className="mt-0.5 text-[12px] text-mute">Prefixo + /slash · moderação só admin</p>
          <div className="mt-3 flex flex-wrap gap-1.5">
            {(status?.commands ?? []).map((c) => (
              <span key={c} className="rounded-full border border-line bg-pill px-2.5 py-1 font-mono text-[11px] text-sub">/{c}</span>
            ))}
          </div>
          <div className="mt-4 rounded-lg border border-line bg-raised p-3 font-mono text-[11px] leading-5 text-sub">
            <p><span className="text-heading">!check {"<nick>"}</span> — disponibilidade ao vivo</p>
            <p><span className="text-heading">!snipe add/remove/list</span> — watchlist do sniper</p>
            <p><span className="text-heading">!ban !kick !mute !purge !slowmode !lock !unlock !userinfo !nuke</span> — moderação</p>
            <p><span className="text-heading">!purge 100 · !purge 100 @user · !purge 100 bots · !purge all</span> — lote 100 + 1s (rate-limit)</p>
            <p><span className="text-heading">!verify</span> — reemite o captcha de 5 chars</p>
          </div>
          <p className="mt-3 text-[11px] leading-relaxed text-mute">
            Se <span className="font-mono text-sub">!x</span> não responde e <span className="font-mono text-sub">/x</span> sim: ative{" "}
            <span className="text-heading">Message Content Intent</span> no portal (Bot → Privileged Gateway Intents) e
            reinvite com <span className="font-mono text-sub">applications.commands</span>. Ver{" "}
            <span className="font-mono text-sub">docs/WEBHOOKS_E_BOT.md</span>.
          </p>
          <p className="mt-3 text-[11px] leading-relaxed text-mute">
            Motor em poise 0.7 (serenity 0.12): cada comando é híbrido — <span className="font-mono text-sub">!x</span> e{" "}
            <span className="font-mono text-sub">/x</span> executam o mesmo código.
          </p>
        </div>
      </div>
    </div>
  );
}
