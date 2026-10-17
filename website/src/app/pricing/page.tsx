"use client";

import { useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import SiteNav from "@/components/SiteNav";
import SiteFooter from "@/components/SiteFooter";

function PlanButton({ plan, children }: { plan: "pro" | "max"; children: React.ReactNode }) {
  const router = useRouter();
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState<string | null>(null);
  const go = async () => {
    setBusy(true);
    setMsg(null);
    try {
      const res = await fetch("/api/billing/checkout", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ plan }),
      });
      const data = await res.json();
      if (res.status === 401) {
        router.push("/login?next=/pricing");
        return;
      }
      if (!res.ok) throw new Error(data?.error ?? `HTTP ${res.status}`);
      router.push("/dashboard/search");
    } catch (e) {
      setMsg(e instanceof Error ? e.message : "falha no checkout");
    } finally {
      setBusy(false);
    }
  };
  return (
    <div>
      <button onClick={go} disabled={busy} className={btnWhite + " w-full disabled:opacity-50"}>
        {busy ? "…" : children}
      </button>
      {msg && <p className="mt-2 text-center text-[12px] text-danger">{msg}</p>}
    </div>
  );
}

const btnWhite =
  "inline-flex w-full items-center justify-center rounded-lg bg-white px-5 py-2.5 text-sm font-semibold text-black transition-colors hover:bg-body";
const btnOutline =
  "inline-flex w-full items-center justify-center rounded-lg border border-linestrong px-5 py-2.5 text-sm font-semibold text-body transition-colors hover:border-white hover:text-white";

const COMPARE: Array<[string, string, string, string]> = [
  ["Tokens", "500 / mo", "2,500 / mo", "2,500+"],
  ["Every search type", "✓", "✓", "✓"],
  ["Real-time streaming", "✓", "✓", "✓"],
  ["Table & card views", "✓", "✓", "✓"],
  ["Search history", "✓", "✓", "✓"],
  ["Export CSV / JSON / TXT", "✓", "✓", "✓"],
  ["Deep Search", "—", "✓", "✓"],
  ["Priority support", "—", "✓", "✓"],
  ["Early access", "—", "✓", "✓"],
  ["API access", "Coming soon", "Coming soon", "Coming soon"],
  ["Webhooks", "—", "—", "✓"],
  ["SSO / SAML", "—", "—", "✓"],
  ["Custom contracts", "—", "—", "✓"],
  ["Dedicated manager", "—", "—", "✓"],
  ["Volume discounts", "—", "—", "✓"],
];

export default function PricingPage() {
  return (
    <div className="min-h-screen bg-page">
      <SiteNav />
      <main className="mx-auto max-w-[1100px] px-6 pt-40 pb-24">
        <div className="mx-auto max-w-2xl text-center">
          <p className="eyebrow">PRICING</p>
          <h1 className="mt-3 text-[38px] font-extrabold tracking-tight text-heading sm:text-[46px]">Find your search plan</h1>
          <p className="mt-4 text-[15px] leading-relaxed text-sub">
            Every paid plan includes all search types and exports. Choose your token allowance, with Deep Search on
            the Max plan.
          </p>
          <div className="mt-5 flex flex-wrap justify-center gap-x-5 gap-y-1.5 text-sm text-body">
            {["Monthly billing", "Cancel anytime", "Instant access"].map((t) => (
              <span key={t} className="flex items-center gap-1.5"><span className="font-mono font-bold text-heading">✓</span>{t}</span>
            ))}
          </div>
        </div>

        <div className="mt-14 grid gap-4 md:grid-cols-3">
          {/* Pro */}
          <div className="flex flex-col rounded-xl border border-line bg-card p-7">
            <h2 className="text-lg font-semibold text-heading">Pro</h2>
            <p className="mt-4 text-4xl font-extrabold text-heading">$30<span className="text-base font-medium text-mute">/mo</span></p>
            <p className="mt-2 text-sm text-sub">For solo work and occasional deep dives.</p>
            <div className="mt-5 border-t border-line pt-5">
              <p className="eyebrow">TOKENS</p>
              <p className="mt-1.5 font-mono text-2xl font-bold text-heading">500</p>
            </div>
            <ul className="mt-5 flex-1 space-y-2.5 text-sm text-sub">
              {["Full People Search profiles", "Unmasked Data Search", "Export CSV · JSON · TXT"].map((t) => (
                <li key={t} className="flex gap-2"><span className="font-mono font-bold text-heading">✓</span>{t}</li>
              ))}
            </ul>
            <div className="mt-7"><PlanButton plan="pro">Get Pro</PlanButton></div>
          </div>

          {/* Max */}
          <div className="relative flex flex-col rounded-xl border border-white bg-card p-7">
            <span className="absolute -top-3 left-1/2 -translate-x-1/2 rounded-full bg-white px-3 py-0.5 font-mono text-[10px] font-bold tracking-[0.2em] text-black">BEST VALUE</span>
            <h2 className="text-lg font-semibold text-heading">Max</h2>
            <p className="mt-4 text-4xl font-extrabold text-heading">$100<span className="text-base font-medium text-mute">/mo</span></p>
            <p className="mt-2 text-sm text-sub">For people who search every day.</p>
            <div className="mt-5 border-t border-line pt-5">
              <p className="eyebrow">TOKENS</p>
              <p className="mt-1.5 font-mono text-2xl font-bold text-heading">2,500</p>
            </div>
            <ul className="mt-5 flex-1 space-y-2.5 text-sm text-sub">
              {["Everything in Pro", "Deep Search — recursive discovery", "Priority support", "Early access"].map((t) => (
                <li key={t} className="flex gap-2"><span className="font-mono font-bold text-heading">✓</span>{t}</li>
              ))}
            </ul>
            <div className="mt-7"><PlanButton plan="max">Get Max</PlanButton></div>
          </div>

          {/* Enterprise */}
          <div className="flex flex-col rounded-xl border border-line bg-card p-7">
            <h2 className="text-lg font-semibold text-heading">Enterprise</h2>
            <p className="mt-4 text-4xl font-extrabold text-heading">Custom</p>
            <p className="mt-2 text-sm text-sub">For teams, agencies, and integrators.</p>
            <div className="mt-5 border-t border-line pt-5">
              <p className="eyebrow">TOKENS</p>
              <p className="mt-1.5 font-mono text-2xl font-bold text-heading">2,500+</p>
            </div>
            <ul className="mt-5 flex-1 space-y-2.5 text-sm text-sub">
              {["API (coming soon)", "SSO · Custom integrations", "Dedicated manager"].map((t) => (
                <li key={t} className="flex gap-2"><span className="font-mono font-bold text-heading">✓</span>{t}</li>
              ))}
            </ul>
            <div className="mt-7"><Link href="/#contact" className={btnOutline}>Contact us</Link></div>
          </div>
        </div>

        <div className="mt-8 flex flex-wrap items-center justify-center gap-x-6 gap-y-2 rounded-xl border border-line bg-surface px-6 py-4">
          <span className="eyebrow">INCLUDED ON EVERY PLAN:</span>
          {["700+ platforms", "Real-time streaming", "Every search type", "Search history"].map((t) => (
            <span key={t} className="text-sm text-sub">· {t}</span>
          ))}
        </div>

        {/* Compare */}
        <div className="mt-20">
          <h2 className="text-center text-[28px] font-bold tracking-tight text-heading">Every plan, side by side</h2>
          <div className="mt-8 overflow-x-auto rounded-xl border border-line bg-card">
            <table className="w-full min-w-[560px] text-sm">
              <thead>
                <tr className="border-b border-line text-left">
                  <th className="px-5 py-3.5 font-medium text-mute">Feature</th>
                  <th className="px-5 py-3.5 font-semibold text-heading">Pro</th>
                  <th className="px-5 py-3.5 font-semibold text-heading">Max</th>
                  <th className="px-5 py-3.5 font-semibold text-heading">Enterprise</th>
                </tr>
              </thead>
              <tbody>
                {COMPARE.map(([f, p, m, e]) => (
                  <tr key={f} className="border-b border-line/60 last:border-0">
                    <td className="px-5 py-3 text-sub">{f}</td>
                    <td className={`px-5 py-3 ${p === "✓" ? "font-mono font-bold text-heading" : "text-sub"}`}>{p}</td>
                    <td className={`px-5 py-3 ${m === "✓" ? "font-mono font-bold text-heading" : "text-sub"}`}>{m}</td>
                    <td className={`px-5 py-3 ${e === "✓" ? "font-mono font-bold text-heading" : "text-sub"}`}>{e}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* Token FAQ */}
        <div className="mx-auto mt-20 max-w-[720px]">
          <h2 className="text-center text-[28px] font-bold tracking-tight text-heading">Token questions</h2>
          <div className="mt-8 divide-y divide-line rounded-xl border border-line bg-card">
            {[
              ["Can I try for free?", "Yes. Every account starts on the free plan with 25 tokens a month — no card required."],
              ["Can I switch later?", "Anytime. Upgrades apply instantly and are prorated; downgrades take effect at the next billing cycle."],
              ["What is a token?", "One token = one username checked. Single search costs 1 token; bulk and dataset jobs cost 1 token per nick."],
              ["What happens when I use up tokens?", "You can keep viewing past results and history, and searches resume the moment tokens reset."],
              ["When do tokens reset?", "30 days after signup (and every 30 days), shown next to the token meter in your sidebar."],
            ].map(([q, a]) => (
              <details key={q} className="group px-6 py-4">
                <summary className="flex cursor-pointer list-none items-center justify-between gap-4 text-[15px] font-medium text-heading [&::-webkit-details-marker]:hidden">
                  {q}<span className="font-mono text-lg text-mute transition-transform group-open:rotate-45">+</span>
                </summary>
                <p className="mt-3 text-sm leading-relaxed text-sub">{a}</p>
              </details>
            ))}
          </div>
        </div>
      </main>
      <SiteFooter />
    </div>
  );
}
