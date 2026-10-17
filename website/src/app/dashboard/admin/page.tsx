"use client";

import { useCallback, useEffect, useState } from "react";
import { Plus, ShieldCheck } from "lucide-react";
import { OwnerBadge } from "@/components/dashboard/DashboardSidebar";

interface Row {
  id: number;
  email: string;
  plan: string;
  role: string;
  tokensUsed: number;
  tokensLimit: number;
  createdAt: string;
}

const PLANS = ["free", "pro", "max", "enterprise", "owner"] as const;

function PlanBadge({ plan, role }: { plan: string; role: string }) {
  const isOwner = role === "owner" || plan === "owner";
  if (isOwner) {
    return (
      <span className="inline-flex items-center gap-1.5 rounded-full border border-white bg-white px-2.5 py-0.5 font-mono text-[11px] font-bold text-black">
        <ShieldCheck size={11} /> OWNER · ilimitado
      </span>
    );
  }
  const tone =
    plan === "max"
      ? "border-white/60 text-heading"
      : plan === "pro"
        ? "border-line text-sub"
        : "border-line text-mute";
  return (
    <span className={`inline-flex items-center rounded-full border bg-pill px-2.5 py-0.5 font-mono text-[11px] capitalize ${tone}`}>
      {plan}
    </span>
  );
}

export default function AdminPage() {
  const [rows, setRows] = useState<Row[]>([]);
  const [allowed, setAllowed] = useState<boolean | null>(null);
  const [email, setEmail] = useState("");
  const [amount, setAmount] = useState(100);
  const [planEmail, setPlanEmail] = useState("");
  const [plan, setPlan] = useState<string>("pro");
  const [msg, setMsg] = useState<string | null>(null);
  const [planMsg, setPlanMsg] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [planBusy, setPlanBusy] = useState(false);

  const load = useCallback(async () => {
    try {
      const me = await fetch("/api/auth/me").then((r) => r.json());
      if (me?.role !== "owner") {
        setAllowed(false);
        return;
      }
      setAllowed(true);
      const res = await fetch("/api/admin/users");
      const data = await res.json();
      if (res.ok) setRows(data.users ?? []);
    } catch {
      setAllowed(false);
    }
  }, []);

  // Mount: gate de owner + lista (setStates só após await + guarda alive).
  useEffect(() => {
    let alive = true;
    const run = async () => {
      try {
        const me = await fetch("/api/auth/me").then((r) => r.json());
        if (!alive) return;
        if (me?.role !== "owner") {
          setAllowed(false);
          return;
        }
        setAllowed(true);
        const res = await fetch("/api/admin/users");
        const data = await res.json();
        if (alive && res.ok) setRows(data.users ?? []);
      } catch {
        if (alive) setAllowed(false);
      }
    };
    void run();
    return () => {
      alive = false;
    };
  }, []);

  const grant = async () => {
    setBusy(true);
    setMsg(null);
    try {
      const res = await fetch("/api/admin/tokens", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, addTokens: amount, resetUsed: true }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data?.error ?? `HTTP ${res.status}`);
      setMsg(`✓ ${data.email}: limite ${data.tokensLimit}, uso zerado`);
      setEmail("");
      load();
    } catch (e) {
      setMsg(e instanceof Error ? e.message : "falha");
    } finally {
      setBusy(false);
    }
  };

  const changePlan = async () => {
    setPlanBusy(true);
    setPlanMsg(null);
    try {
      const res = await fetch("/api/admin/tokens", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email: planEmail, plan, resetUsed: true }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data?.error ?? `HTTP ${res.status}`);
      setPlanMsg(`✓ ${data.email}: agora é ${data.plan} (limite ${data.tokensLimit})`);
      setPlanEmail("");
      load();
    } catch (e) {
      setPlanMsg(e instanceof Error ? e.message : "falha");
    } finally {
      setPlanBusy(false);
    }
  };

  if (allowed === false) {
    return (
      <div className="rounded-xl border border-line bg-card px-6 py-12 text-center">
        <p className="text-[15px] font-semibold text-heading">Área do owner</p>
        <p className="mt-1.5 text-sm text-mute">Sua conta não tem acesso a esta página.</p>
      </div>
    );
  }

  return (
    <div className="space-y-5">
      <div>
        <h1 className="flex items-center gap-2 text-xl font-bold text-heading">
          <ShieldCheck size={18} className="text-heading" /> Admin
        </h1>
        <p className="mt-0.5 text-sm text-mute">Só você vê esta página: tokens + plano de qualquer conta.</p>
      </div>

      <div className="rounded-xl border border-line bg-card p-4">
        <p className="eyebrow mb-3">ADICIONAR TOKENS</p>
        <div className="flex flex-wrap gap-2">
          <input
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="email da conta"
            className="min-w-[220px] flex-1 rounded-lg border border-line bg-input px-3 py-2 text-sm text-heading placeholder:text-mute focus:border-linestrong focus:outline-none"
          />
          <input
            type="number"
            value={amount}
            onChange={(e) => setAmount(Number(e.target.value) || 0)}
            className="w-28 rounded-lg border border-line bg-input px-3 py-2 font-mono text-sm text-heading focus:border-linestrong focus:outline-none"
          />
          <button
            onClick={grant}
            disabled={busy || !email.trim()}
            className="flex items-center gap-1.5 rounded-lg bg-white px-4 py-2 text-[13px] font-semibold text-black hover:bg-body disabled:opacity-50"
          >
            <Plus size={13} /> {busy ? "…" : "Adicionar + zerar uso"}
          </button>
        </div>
        {msg && <p className="mt-2 text-[13px] text-sub">{msg}</p>}
      </div>

      <div className="rounded-xl border border-line bg-card p-4">
        <p className="eyebrow mb-3">MUDAR PLANO DA CONTA</p>
        <div className="flex flex-wrap gap-2">
          <input
            value={planEmail}
            onChange={(e) => setPlanEmail(e.target.value)}
            placeholder="email da conta"
            className="min-w-[220px] flex-1 rounded-lg border border-line bg-input px-3 py-2 text-sm text-heading placeholder:text-mute focus:border-linestrong focus:outline-none"
          />
          <select
            value={plan}
            onChange={(e) => setPlan(e.target.value)}
            className="rounded-lg border border-line bg-input px-3 py-2 font-mono text-[13px] text-heading focus:border-linestrong focus:outline-none"
          >
            {PLANS.map((p) => (
              <option key={p} value={p}>
                {p === "owner" ? "owner · ilimitado (só você)" : p}
              </option>
            ))}
          </select>
          <button
            onClick={changePlan}
            disabled={planBusy || !planEmail.trim()}
            className="flex items-center gap-1.5 rounded-lg border border-linestrong px-4 py-2 text-[13px] font-semibold text-heading hover:border-white disabled:opacity-50"
          >
            {planBusy ? "…" : "Aplicar plano + zerar uso"}
          </button>
        </div>
        {planMsg && <p className="mt-2 text-[13px] text-sub">{planMsg}</p>}
        <p className="mt-2 text-[12px] text-mute">
          Owner é distinto de todos os premium: ilimitado, sem cobrança, com selo próprio. Use com cuidado — só para sua conta.
        </p>
      </div>

      <div className="overflow-x-auto rounded-xl border border-line bg-card">
        <table className="w-full min-w-[640px] text-sm">
          <thead>
            <tr className="border-b border-line text-left font-mono text-[10px] tracking-widest text-mute">
              {["EMAIL", "PLANO", "ROLE", "USO", "DESDE", ""].map((h) => (
                <th key={h} className="px-4 py-3 font-medium">{h}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.id} className="border-b border-line/60 last:border-0 hover:bg-raised">
                <td className="px-4 py-3 font-mono text-[13px] text-heading">
                  <span className="flex items-center gap-1.5">
                    {r.email} {r.role === "owner" && <OwnerBadge size={12} />}
                  </span>
                </td>
                <td className="px-4 py-3 text-[13px] capitalize text-sub">
                  <PlanBadge plan={r.role === "owner" ? "owner" : r.plan} role={r.role} />
                </td>
                <td className="px-4 py-3 font-mono text-[12px] text-sub">{r.role}</td>
                <td className="px-4 py-3 font-mono text-[13px] text-sub">{r.tokensUsed}/{r.tokensLimit}</td>
                <td className="px-4 py-3 text-[13px] text-mute">{new Date(r.createdAt).toLocaleDateString()}</td>
                <td className="px-4 py-3">
                  <select
                    defaultValue=""
                    onChange={async (e) => {
                      const v = e.target.value;
                      if (!v) return;
                      e.target.value = "";
                      await fetch("/api/admin/tokens", {
                        method: "POST",
                        headers: { "Content-Type": "application/json" },
                        body: JSON.stringify({ email: r.email, plan: v, resetUsed: true }),
                      });
                      load();
                    }}
                    className="rounded-lg border border-line bg-input px-2 py-1 font-mono text-[11px] text-sub focus:border-linestrong focus:outline-none"
                    title={`Trocar plano de ${r.email}`}
                  >
                    <option value="">trocar plano…</option>
                    {PLANS.map((p) => (
                      <option key={p} value={p}>{p}</option>
                    ))}
                  </select>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {rows.length === 0 && <p className="px-4 py-8 text-center text-sm text-mute">Nenhuma conta.</p>}
      </div>
    </div>
  );
}
