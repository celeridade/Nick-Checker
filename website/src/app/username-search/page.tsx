"use client";

import { useState } from "react";
import Link from "next/link";
import { ArrowRight, Crosshair, Search } from "lucide-react";
import SiteNav from "@/components/SiteNav";
import SiteFooter from "@/components/SiteFooter";
import ScopeReticle from "@/components/ScopeReticle";

const TABS = [
  { id: "username", label: "Username", placeholder: "Enter a username..." },
  { id: "email", label: "Email", placeholder: "Enter an email address..." },
  { id: "phone", label: "Phone", placeholder: "Enter a phone number..." },
] as const;

type TabId = (typeof TABS)[number]["id"];

export default function UsernameSearchPage() {
  const [tab, setTab] = useState<TabId>("username");
  const [value, setValue] = useState("");
  const active = TABS.find((t) => t.id === tab)!;

  return (
    <div className="flex min-h-screen flex-col bg-page">
      <SiteNav />
      <main className="mx-auto flex w-full max-w-[820px] flex-1 flex-col items-center px-6 pt-40 pb-20">
        <p className="eyebrow">PUBLIC OSINT SEARCH</p>
        <h1 className="mt-3 text-center text-[34px] font-extrabold tracking-tight text-heading sm:text-[42px]">
          Fan one query across 700+ platforms
        </h1>

        <div className="mt-9 flex gap-1 rounded-full border border-line bg-pill p-1">
          {TABS.map((t) => (
            <button
              key={t.id}
              onClick={() => setTab(t.id)}
              className={`rounded-full px-5 py-2 text-sm font-medium transition-colors ${
                tab === t.id ? "bg-white text-black" : "text-sub hover:text-heading"
              }`}
            >
              {t.label}
            </button>
          ))}
        </div>

        <div className="mt-5 flex w-full gap-2">
          <div className="relative flex-1">
            <Search size={16} className="absolute top-1/2 left-4 -translate-y-1/2 text-mute" />
            <input
              value={value}
              onChange={(e) => setValue(e.target.value)}
              placeholder={active.placeholder}
              className="w-full rounded-xl border border-line bg-input py-3.5 pr-12 pl-11 text-[15px] text-heading placeholder:text-mute focus:border-linestrong focus:outline-none"
            />
            <Crosshair size={16} className="absolute top-1/2 right-4 -translate-y-1/2 text-mute" />
          </div>
          <Link
            href={value ? `/dashboard/search?q=${encodeURIComponent(value)}` : "/login"}
            className="flex items-center gap-2 rounded-xl bg-white/85 px-5 text-sm font-semibold text-black transition-colors hover:bg-white"
          >
            <ArrowRight size={15} /> Search
          </Link>
        </div>

        <p className="mt-4 max-w-md text-center text-[12px] leading-relaxed text-mute">
          By using this search you agree to our{" "}
          <span className="text-sub underline underline-offset-2">Terms</span> and{" "}
          <span className="text-sub underline underline-offset-2">Privacy Policy</span>. Don&apos;t want to appear?{" "}
          <span className="text-sub underline underline-offset-2">Opt out</span>.
        </p>

        <div className="mt-14 flex flex-col items-center">
          <ScopeReticle size={200} />
          <p className="mt-10 text-[16px] font-semibold text-heading">Search social accounts</p>
          <p className="mt-1 text-sm text-mute">Enter a query above to get started.</p>
        </div>
      </main>
      <SiteFooter />
    </div>
  );
}
