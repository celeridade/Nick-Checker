"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";

function SettingsCard({
  title, sub, danger, children,
}: { title: string; sub: string; danger?: boolean; children: React.ReactNode }) {
  return (
    <div className={`rounded-xl border bg-card p-5 ${danger ? "border-danger/60" : "border-line"}`}>
      <h3 className={`text-[15px] font-semibold ${danger ? "text-danger" : "text-heading"}`}>{title}</h3>
      <p className="mt-0.5 text-[12px] text-mute">{sub}</p>
      <div className="mt-4">{children}</div>
    </div>
  );
}

interface Me {
  email: string;
  plan: string;
  role?: string;
  provider: string;
  tokens: { used: number; limit: number; remaining: number; unlimited?: boolean };
  resetAt: string | null;
}

export default function SettingsPage() {
  const router = useRouter();
  const [historyOn, setHistoryOn] = useState(true);
  const [me, setMe] = useState<Me | null>(null);
  const [current, setCurrent] = useState("");
  const [next, setNext] = useState("");
  const [pwMsg, setPwMsg] = useState<string | null>(null);
  const [delConfirm, setDelConfirm] = useState(false);

  useEffect(() => {
    fetch("/api/auth/me")
      .then((r) => (r.ok ? r.json() : null))
      .then((data) => data?.email && setMe(data))
      .catch(() => undefined);
  }, []);

  const isOwner = me?.role === "owner" || me?.tokens.unlimited === true;
  const left = isOwner ? "∞" : String(me ? me.tokens.remaining : 0);
  const limit = me?.tokens.limit ?? 25;
  const usedNum = me?.tokens.used ?? 0;
  const pct = isOwner ? 0 : Math.min(100, Math.round((usedNum / Math.max(1, limit)) * 100));

  const setPassword = async () => {
    setPwMsg(null);
    try {
      const res = await fetch("/api/auth/password", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ current, next }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data?.error ?? `HTTP ${res.status}`);
      setPwMsg("✓ senha atualizada");
      setCurrent("");
      setNext("");
    } catch (e) {
      setPwMsg(e instanceof Error ? e.message : "falha");
    }
  };

  const logout = async () => {
    await fetch("/api/auth/logout", { method: "POST" }).catch(() => undefined);
    router.push("/");
    router.refresh();
  };

  const deleteAccount = async () => {
    if (!delConfirm) {
      setDelConfirm(true);
      return;
    }
    await fetch("/api/auth/me", { method: "DELETE" }).catch(() => undefined);
    router.push("/");
    router.refresh();
  };

  return (
    <div className="space-y-6 pt-2">
      <div>
        <h1 className="text-xl font-bold text-heading">Settings</h1>
        <p className="mt-0.5 text-sm text-mute">Manage your account preferences</p>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {/* Plan */}
        <SettingsCard title="Plan" sub="Your subscription and billing">
          <p className="flex items-center gap-2 text-sm capitalize text-heading">
            {isOwner ? "owner 👑" : (me?.plan ?? "…")} <span className="text-mute">·</span> <span className="text-mute">{isOwner ? "tokens ilimitados" : `${limit} tokens a month`}</span>
          </p>
          <Link
            href="/pricing"
            className="mt-3 inline-block rounded-lg bg-white px-4 py-2 text-[13px] font-semibold text-black transition-colors hover:bg-body"
          >
            Upgrade
          </Link>
          <div className="mt-5">
            <div className="flex items-baseline justify-between text-[11px] text-mute">
              <span>Tokens left</span>
              <span className="font-mono"><span className="text-heading">{left}</span>{isOwner ? "" : `/${limit}`}</span>
            </div>
            <div className="mt-1.5 h-1 overflow-hidden rounded-full bg-line">
              <div className="h-full rounded-full bg-white" style={{ width: `${100 - pct}%` }} />
            </div>
            <p className="mt-1.5 text-[11px] text-mute">
              {me?.resetAt ? `Resets ${new Date(me.resetAt).toLocaleDateString()}` : "Resets monthly"}
            </p>
          </div>
        </SettingsCard>

        {/* Authentication */}
        <SettingsCard title="Authentication" sub="Your method">
          <p className="text-sm capitalize text-heading">
            Signed in with {me?.provider === "email" ? "email + password" : (me?.provider ?? "…")}
          </p>
          <div className="mt-3 space-y-2">
            <input
              type="password"
              value={current}
              onChange={(e) => setCurrent(e.target.value)}
              placeholder="Current password (blank if none)"
              autoComplete="current-password"
              className="w-full rounded-lg border border-line bg-input px-3 py-2 text-[13px] text-heading placeholder:text-mute focus:border-linestrong focus:outline-none"
            />
            <input
              type="password"
              value={next}
              onChange={(e) => setNext(e.target.value)}
              placeholder="New password (8+ chars)"
              autoComplete="new-password"
              className="w-full rounded-lg border border-line bg-input px-3 py-2 text-[13px] text-heading placeholder:text-mute focus:border-linestrong focus:outline-none"
            />
            {pwMsg && <p className="text-[12px] text-sub">{pwMsg}</p>}
            <button
              onClick={setPassword}
              className="rounded-lg border border-linestrong px-4 py-2 text-[13px] font-semibold text-body transition-colors hover:border-white hover:text-white"
            >
              Set a password
            </button>
          </div>
          <button
            onClick={logout}
            className="mt-3 text-[12px] text-mute underline underline-offset-2 hover:text-sub"
          >
            Log out everywhere
          </button>
        </SettingsCard>

        {/* Email */}
        <SettingsCard title="Email Address" sub="Where codes go">
          <p className="flex items-center gap-2 font-mono text-[13px] text-heading">
            {me?.email ?? "…"}
            <span className="font-bold text-found">✓</span>
          </p>
          <p className="mt-2 text-[12px] text-mute">Verified and receiving security codes.</p>
        </SettingsCard>

        {/* Search history */}
        <SettingsCard title="Search History" sub="Turn on or off">
          <div className="flex items-center justify-between">
            <span className="text-sm text-sub">Enable search history</span>
            <button
              onClick={() => setHistoryOn(!historyOn)}
              className="rounded-full border border-line bg-input px-0.5 py-0.5"
              role="switch"
              aria-checked={historyOn}
            >
              <span className={`flex items-center gap-1 rounded-full px-2.5 py-1 text-[11px] font-semibold ${historyOn ? "bg-white text-black" : ""}`}>On</span>
              <span className={`flex items-center gap-1 rounded-full px-2.5 py-1 text-[11px] font-semibold ${!historyOn ? "bg-white text-black" : "text-mute"}`}>Off</span>
            </button>
          </div>
        </SettingsCard>

        {/* Delete account */}
        <SettingsCard title="Delete Account" sub="Close account and log out everywhere" danger>
          <p className="text-[13px] leading-relaxed text-sub">
            Permanently removes your account, plan, and search history. This cannot be undone.
          </p>
          <button
            onClick={deleteAccount}
            className="mt-3 rounded-lg border border-danger px-4 py-2 text-[13px] font-semibold text-danger transition-colors hover:bg-danger/10"
          >
            {delConfirm ? "Click again to confirm" : "Delete account"}
          </button>
        </SettingsCard>
      </div>
    </div>
  );
}
