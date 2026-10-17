"use client";

import { useState } from "react";
import { ArrowRight, Minus, Plus, Zap } from "lucide-react";
import ScopeReticle from "@/components/ScopeReticle";

const TABS = ["Email", "Phone", "Name", "Address", "Map"] as const;
type Tab = (typeof TABS)[number];

const inputCls =
  "w-full rounded-lg border border-line bg-input py-2.5 px-3.5 text-sm text-heading placeholder:text-mute focus:border-linestrong focus:outline-none";

function SearchBtn() {
  return (
    <button className="flex items-center justify-center gap-2 rounded-lg bg-white px-5 py-2.5 text-sm font-semibold text-black transition-colors hover:bg-body">
      <ArrowRight size={14} /> Search <Zap size={12} fill="currentColor" /> 1
    </button>
  );
}

function Field({ label, placeholder, className = "" }: { label: string; placeholder: string; className?: string }) {
  return (
    <label className={`block ${className}`}>
      <span className="mb-1.5 block text-[12px] text-mute">{label}</span>
      <input placeholder={placeholder} className={inputCls} />
    </label>
  );
}

/** Stylized satellite view of the US — muted terrain tones, functional mock. */
function SatelliteMap() {
  const [mode, setMode] = useState<"Satellite" | "Map">("Satellite");
  const [zoom, setZoom] = useState(1);
  return (
    <div className="relative overflow-hidden rounded-xl border border-line">
      <svg
        viewBox="0 0 900 520"
        className="h-[460px] w-full transition-transform duration-300"
        style={{ background: mode === "Satellite" ? "#0d141b" : "#0a0a0a", transform: `scale(${zoom})` }}
        role="img"
        aria-label="Satellite map of the United States"
      >
        {/* faint graticule */}
        <g stroke="#161616" strokeWidth="1">
          {Array.from({ length: 11 }).map((_, i) => (
            <line key={`v${i}`} x1={i * 90} y1="0" x2={i * 90} y2="520" />
          ))}
          {Array.from({ length: 7 }).map((_, i) => (
            <line key={`h${i}`} x1="0" y1={i * 90} x2="900" y2={i * 90} />
          ))}
        </g>
        {/* USA landmass */}
        <path
          d="M 95 105 L 305 68 L 415 78 L 480 70 L 585 130 L 600 175 L 555 205 L 575 235 L 545 275 L 690 320 L 660 350 L 560 335 L 520 360 L 470 385 L 430 350 L 380 365 L 330 345 L 290 365 L 240 340 L 195 300 L 150 250 L 110 185 L 85 140 Z"
          fill={mode === "Satellite" ? "#25301f" : "#1a1a1a"}
          stroke={mode === "Satellite" ? "#3a4a30" : "#2e2e2e"}
          strokeWidth="1.5"
        />
        {/* terrain variation */}
        {mode === "Satellite" && (
          <g opacity="0.85">
            <path d="M 120 130 L 260 100 L 300 140 L 230 190 L 140 175 Z" fill="#2c3822" />
            <path d="M 330 220 L 480 200 L 520 250 L 450 300 L 350 290 Z" fill="#33301f" />
            <path d="M 200 220 L 300 210 L 320 260 L 250 300 L 190 270 Z" fill="#2f3524" />
            <path d="M 430 120 L 540 150 L 520 200 L 430 185 Z" fill="#2a3626" />
          </g>
        )}
        {/* Florida */}
        <path d="M 660 350 L 700 380 L 715 420 L 685 405 L 660 370 Z"
          fill={mode === "Satellite" ? "#28331f" : "#1a1a1a"} stroke={mode === "Satellite" ? "#3a4a30" : "#2e2e2e"} strokeWidth="1.5" />
        {/* Mexico */}
        <path d="M 300 370 L 380 368 L 430 355 L 470 385 L 500 430 L 450 460 L 390 440 L 340 415 L 305 390 Z"
          fill={mode === "Satellite" ? "#33301f" : "#161616"} stroke={mode === "Satellite" ? "#4a4428" : "#2e2e2e"} strokeWidth="1.5" />
        {/* Cuba */}
        <path d="M 560 445 q 70 -14 140 2 q -70 12 -140 -2 Z"
          fill={mode === "Satellite" ? "#2c3822" : "#161616"} stroke={mode === "Satellite" ? "#3a4a30" : "#2e2e2e"} strokeWidth="1.5" />
        {/* city dots */}
        <g fill={mode === "Satellite" ? "#8b8b6a" : "#424242"}>
          {[[180, 150], [260, 200], [420, 160], [520, 190], [600, 260], [330, 300], [470, 310]].map(([x, y], i) => (
            <circle key={i} cx={x} cy={y} r="2.4" />
          ))}
        </g>
        {/* labels */}
        <g fontFamily="ui-monospace, monospace" fontSize="13" letterSpacing="4" fill="#8a8a8a">
          <text x="330" y="230">UNITED STATES</text>
          <text x="360" y="412" fontSize="11">MEXICO</text>
          <text x="610" y="470" fontSize="10">CUBA</text>
        </g>
      </svg>

      {/* top-center search pill */}
      <div className="absolute top-3 left-1/2 -translate-x-1/2">
        <button className="flex items-center gap-2 rounded-full bg-white px-4 py-2 text-[13px] font-semibold text-black shadow-lg hover:bg-body">
          Search this area <Zap size={12} fill="currentColor" /> 1
        </button>
      </div>

      {/* bottom-left mode toggle */}
      <div className="absolute bottom-3 left-3 flex rounded-lg border border-line bg-pill p-1">
        {(["Satellite", "Map"] as const).map((m) => (
          <button
            key={m}
            onClick={() => setMode(m)}
            className={`rounded-md px-3 py-1.5 text-[12px] font-medium transition-colors ${
              mode === m ? "bg-white text-black" : "text-sub hover:text-heading"
            }`}
          >
            {m}
          </button>
        ))}
      </div>

      {/* bottom-right zoom + attribution */}
      <div className="absolute right-3 bottom-3 flex flex-col items-end gap-2">
        <div className="flex flex-col overflow-hidden rounded-lg border border-line bg-pill">
          <button onClick={() => setZoom((z) => Math.min(2, +(z + 0.25).toFixed(2)))} className="grid size-8 place-items-center text-sub hover:text-heading" aria-label="Zoom in">
            <Plus size={14} />
          </button>
          <span className="h-px bg-line" />
          <button onClick={() => setZoom((z) => Math.max(0.75, +(z - 0.25).toFixed(2)))} className="grid size-8 place-items-center text-sub hover:text-heading" aria-label="Zoom out">
            <Minus size={14} />
          </button>
        </div>
        <p className="rounded bg-black/70 px-1.5 py-0.5 font-mono text-[9px] text-mute">L96A1 terrain tiles © 2026</p>
      </div>
    </div>
  );
}

export default function PeopleSearchPage() {
  const [tab, setTab] = useState<Tab>("Email");

  return (
    <div className="space-y-5">
      <div className="flex flex-col items-center pt-2">
        <div className="flex gap-1 rounded-full border border-line bg-pill p-1">
          {TABS.map((t) => (
            <button
              key={t}
              onClick={() => setTab(t)}
              className={`rounded-full px-5 py-1.5 text-sm font-medium transition-colors ${
                tab === t ? "bg-white text-black" : "text-sub hover:text-heading"
              }`}
            >
              {t}
            </button>
          ))}
        </div>
      </div>

      {tab === "Map" ? (
        <SatelliteMap />
      ) : (
        <>
          <div className="mx-auto w-full max-w-xl rounded-xl border border-line bg-card p-5">
            {tab === "Email" && (
              <div className="space-y-4">
                <Field label="Email address" placeholder="name@example.com" />
                <SearchBtn />
              </div>
            )}
            {tab === "Phone" && (
              <div className="space-y-4">
                <label className="block">
                  <span className="mb-1.5 block text-[12px] text-mute">Phone number</span>
                  <div className="flex items-center gap-2">
                    <span className="flex items-center gap-1.5 rounded-lg border border-line bg-input px-3 py-2.5 text-sm text-sub">
                      <svg viewBox="0 0 20 14" className="h-3 w-4.5 rounded-[2px]" aria-hidden>
                        <rect width="20" height="14" fill="#4a4a4a" />
                        {Array.from({ length: 6 }).map((_, i) => (
                          <rect key={i} y={i * 2 + 1} width="20" height="1" fill="#7a7a7a" />
                        ))}
                        <rect width="9" height="7" fill="#5c5c5c" />
                      </svg>
                      +1
                    </span>
                    <input placeholder="(555) 000-0000" className={inputCls} />
                  </div>
                </label>
                <SearchBtn />
              </div>
            )}
            {tab === "Name" && (
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-3">
                  <Field label="First name *" placeholder="John" />
                  <Field label="Last name *" placeholder="Doe" />
                </div>
                <div className="grid grid-cols-3 gap-3">
                  <Field label="City" placeholder="Austin" />
                  <Field label="State" placeholder="TX" />
                  <Field label="ZIP" placeholder="78701" />
                </div>
                <SearchBtn />
              </div>
            )}
            {tab === "Address" && (
              <div className="space-y-4">
                <Field label="Street address *" placeholder="1200 Congress Ave" />
                <div className="grid grid-cols-3 gap-3">
                  <Field label="City *" placeholder="Austin" />
                  <Field label="State *" placeholder="TX" />
                  <Field label="ZIP *" placeholder="78701" />
                </div>
                <SearchBtn />
              </div>
            )}
          </div>

          <div className="flex flex-col items-center py-14">
            <ScopeReticle size={196} />
            <h1 className="mt-10 text-[17px] font-semibold text-heading">Search for anyone</h1>
            <p className="mt-1.5 text-sm text-mute">Select a search type above to get started.</p>
          </div>
        </>
      )}

    </div>
  );
}
