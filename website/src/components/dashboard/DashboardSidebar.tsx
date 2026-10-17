"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { Activity, Bot, Clock3, Crosshair, Database, FolderOpen, Globe, Layers, LogOut, ScanSearch, Settings, ShieldCheck, Users } from "lucide-react";
import L96A1Wordmark from "@/components/L96A1Wordmark";

/** Ícone do owner: escudo com check — combina com "dono/acesso total".
 * Nunca usar emoji 👑 (removido): emoji não tem contraste no tema mono. */
export function OwnerBadge({ size = 13 }: { size?: number }) {
  return (
    <span
      title="owner — acesso total"
      className="inline-flex items-center justify-center rounded-full border border-white/40 bg-white px-1 py-0.5 align-middle"
    >
      <ShieldCheck size={size} className="text-black" />
    </span>
  );
}

const NAV = [
  { href: "/dashboard/search", label: "Social Search", icon: ScanSearch },
  { href: "/dashboard/bulk", label: "Bulk Scan", icon: Layers },
  { href: "/dashboard/dataset", label: "Dataset", icon: FolderOpen },
  { href: "/dashboard/snipe", label: "Sniper", icon: Crosshair },
  { href: "/dashboard/proxies", label: "Proxies", icon: Globe },
  { href: "/dashboard/bot", label: "Bot", icon: Bot },
  { href: "/dashboard/doctor", label: "Doctor", icon: Activity },
  { href: "/dashboard/people", label: "People Search", icon: Users },
  { href: "/dashboard/data", label: "Data Search", icon: Database },
  { href: "/dashboard/history", label: "History", icon: Clock3 },
  { href: "/dashboard/settings", label: "Settings", icon: Settings },
];

function NavItems({ pathname, isOwner }: { pathname: string; isOwner: boolean }) {
  const items = isOwner ? [...NAV, { href: "/dashboard/admin", label: "Admin", icon: ShieldCheck }] : NAV;
  return (
    <>
      {items.map(({ href, label, icon: Icon }) => {
        const active = pathname.startsWith(href);
        return (
          <Link
            key={href}
            href={href}
            className={`flex shrink-0 items-center gap-2.5 rounded-lg px-3 py-2 text-sm transition-colors ${
              active ? "bg-raised text-heading" : "text-sub hover:text-heading"
            }`}
          >
            <Icon size={15} />
            {label}
          </Link>
        );
      })}
    </>
  );
}

interface Me {
  email: string;
  plan: string;
  role?: string;
  tokens: { used: number; limit: number; remaining: number; unlimited?: boolean };
  resetAt: string | null;
}

export default function DashboardSidebar() {
  const pathname = usePathname();
  const router = useRouter();
  const [me, setMe] = useState<Me | null>(null);

  useEffect(() => {
    fetch("/api/auth/me")
      .then((r) => (r.ok ? r.json() : null))
      .then((data) => data?.email && setMe(data))
      .catch(() => undefined);
  }, []);

  const logout = async () => {
    await fetch("/api/auth/logout", { method: "POST" }).catch(() => undefined);
    router.push("/");
    router.refresh();
  };

  const isOwner = me?.role === "owner" || me?.tokens.unlimited === true;
  const used = me?.tokens.used ?? 0;
  const limit = me?.tokens.limit ?? 25;
  const leftLabel = isOwner ? "∞" : String(limit - used);
  const pct = Math.min(100, Math.round((used / Math.max(1, limit)) * 100));
  const resetLabel = me?.resetAt
    ? `Resets ${new Date(me.resetAt).toLocaleDateString("en-US", { month: "short", day: "numeric" })}`
    : "Resets monthly";

  return (
    <>
      {/* Desktop sidebar */}
      <aside className="fixed inset-y-0 left-0 z-40 hidden w-[260px] flex-col border-r border-line bg-surface lg:flex">
        <Link href="/" className="flex items-center gap-2.5 px-5 py-5">
          <L96A1Wordmark size="mini" />
          <span className="text-[15px] font-bold tracking-tight text-heading">L96A1</span>
        </Link>
        <nav className="flex-1 space-y-1 overflow-y-auto px-3 pt-2">
          <NavItems pathname={pathname} isOwner={isOwner} />
        </nav>
        <div className="px-5 pb-5">
          <div className="mb-4">
            <div className="flex items-baseline justify-between">
              <span className="eyebrow">TOKENS</span>
              <span className="font-mono text-[11px] text-sub">
                <span className="text-heading">{leftLabel}</span>{isOwner ? "" : `/${limit}`}
              </span>
            </div>
            <div className="mt-2 h-1 overflow-hidden rounded-full bg-line">
              <div className="h-full rounded-full bg-white" style={{ width: `${100 - pct}%` }} />
            </div>
            <div className="mt-2 flex items-center justify-between text-[11px]">
              <span className="text-mute">{resetLabel}</span>
              <Link href="/pricing" className="text-sub underline underline-offset-2 hover:text-heading">
                Unlock more
              </Link>
            </div>
          </div>
          <div className="border-t border-line pt-4">
            <p className="flex items-center gap-1.5 truncate text-[13px] font-medium text-heading">
              <span className="truncate">{me?.email ?? "…"}</span> {isOwner && <OwnerBadge />}
            </p>
            <p className="mt-0.5 text-[11px] uppercase text-mute">
              {isOwner ? <span className="font-semibold text-heading">owner · ilimitado</span> : `${me?.plan ?? "free"} plan`}
            </p>
            <button
              onClick={logout}
              className="mt-3 flex w-full items-center gap-2 rounded-lg border border-line px-3 py-2 text-[13px] text-sub transition-colors hover:border-linestrong hover:text-heading"
            >
              <LogOut size={14} /> Log out
            </button>
          </div>
        </div>
      </aside>

      {/* Mobile top bar */}
      <div className="sticky top-0 z-40 border-b border-line bg-surface lg:hidden">
        <div className="flex items-center justify-between px-4 py-3">
          <Link href="/" className="flex items-center gap-2">
            <L96A1Wordmark size="mini" />
            <span className="text-sm font-bold text-heading">L96A1</span>
          </Link>
          <div className="flex items-center gap-2 font-mono text-[11px] text-sub">
            <span className="rounded-full border border-line px-2.5 py-1">
              TOKENS <span className="text-heading">{leftLabel}</span>{isOwner ? "" : `/${limit}`}
            </span>
            <button onClick={logout} aria-label="Log out" className="rounded-full border border-line p-1.5 text-sub"><LogOut size={13} /></button>
          </div>
        </div>
        <nav className="flex gap-1 overflow-x-auto px-3 pb-2.5">
          <NavItems pathname={pathname} isOwner={isOwner} />
        </nav>
      </div>
    </>
  );
}
