"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import {
  ArrowRight, ArrowUpDown, ChevronDown, Download, FileJson, FileText,
  Funnel, LayoutGrid, ListPlus, Lock, Search, Table2, X, Zap,
} from "lucide-react";
import ScopeReticle from "@/components/ScopeReticle";
import ResultCard from "@/components/dashboard/ResultCard";
import Aggregators from "@/components/dashboard/Aggregators";
import ProfileModal from "@/components/dashboard/ProfileModal";
import PlatformPicker from "@/components/dashboard/PlatformPicker";
import RateBox from "@/components/dashboard/RateBox";
import {
  buildStreamEventsFiltered, CATEGORY_LABELS, faviconUrl, fmtCount, fmtDate,
  type Category, type SearchResult, type WarnInfo,
} from "@/lib/results";
import { downloadFile, resultsToCSV, resultsToJSON, resultsToTXT } from "@/lib/export";
import ClaimBox from "@/components/dashboard/ClaimBox";

type Phase = "idle" | "streaming" | "done";
type SortMode = "default" | "followers" | "posts" | "newest" | "az";
type ViewMode = "cards" | "table";

const SORT_LABELS: Record<SortMode, string> = {
  default: "Default order",
  followers: "Most followers",
  posts: "Most posts",
  newest: "Recently joined",
  az: "Platform A–Z",
};

export default function SocialSearch({ initialQuery, live, initialPlatforms }: { initialQuery: string; live: boolean; initialPlatforms?: string }) {
  const router = useRouter();
  const [tab, setTab] = useState<"username" | "email" | "phone">("username");
  const [input, setInput] = useState(initialQuery);
  const [platforms, setPlatforms] = useState(initialPlatforms ?? "all");

  const [phase, setPhase] = useState<Phase>(initialQuery ? "streaming" : "idle");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [freePlatforms, setFreePlatforms] = useState<string[]>([]);
  const [warns, setWarns] = useState<WarnInfo[]>([]);
  const [warnCounts, setWarnCounts] = useState<{ rates: number; errors: number; unknowns: number }>({ rates: 0, errors: 0, unknowns: 0 });
  const [flash, setFlash] = useState<string | null>(null);
  const [checked, setChecked] = useState(0);
  const [total, setTotal] = useState(748);
  const [authError, setAuthError] = useState<string | null>(null);
  const [elapsed, setElapsed] = useState(0);
  const [modal, setModal] = useState<SearchResult | null>(null);

  // Troca de busca reseta o stream durante o render (padrão React p/
  // "ajustar estado quando a prop muda" — sem efeito em cascata).
  const [prevQuery, setPrevQuery] = useState(initialQuery);
  const [prevLive, setPrevLive] = useState(live);
  const [prevPlats, setPrevPlats] = useState(initialPlatforms);
  if (initialQuery !== prevQuery || live !== prevLive || initialPlatforms !== prevPlats) {
    setPrevQuery(initialQuery);
    setPrevLive(live);
    setPrevPlats(initialPlatforms);
    const q = initialQuery.trim();
    if (!q) {
      setPhase("idle");
      setResults([]);
    } else {
      setPhase("streaming");
      setResults([]);
      setFreePlatforms([]);
      setWarns([]);
      setWarnCounts({ rates: 0, errors: 0, unknowns: 0 });
      setChecked(0);
      setTotal(748);
      setAuthError(null);
      setElapsed(0);
    }
  }

  // toolbar state
  const [text, setText] = useState("");
  const [cats, setCats] = useState<Set<Category>>(new Set());
  const [sort, setSort] = useState<SortMode>("default");
  const [view, setView] = useState<ViewMode>("cards");
  const [openMenu, setOpenMenu] = useState<"filters" | "sort" | "export" | null>(null);
  const [hidePills, setHidePills] = useState(false);

  const menuRef = useRef<HTMLDivElement>(null);
  const clockRef = useRef<number | null>(null);
  const startedAtRef = useRef<number>(0);
  const stopRef = useRef(false);
  const readerRef = useRef<ReadableStreamDefaultReader<Uint8Array> | null>(null);

  const stop = useCallback(() => {
    // Para o stream local; o motor registra parcial como cancelled no history.
    stopRef.current = true;
    readerRef.current?.cancel().catch(() => undefined);
    setPhase("done");
  }, []);

  const stopClock = useCallback(() => {
    if (clockRef.current !== null) {
      window.clearInterval(clockRef.current);
      clockRef.current = null;
    }
  }, []);

  /* ---------------------------- stream engine ---------------------------- */
  const applyEvent = useCallback((name: string, data: string) => {
    try {
      const parsed = JSON.parse(data);
      if (name === "result" && parsed.platform) {
        const r = parsed as SearchResult;
        setResults((prev) => (prev.some((p) => p.platform === r.platform) ? prev : [...prev, r]));
        setFlash(r.platform);
        window.setTimeout(() => setFlash((f) => (f === r.platform ? null : f)), 850);
      } else if (name === "checked") setChecked(parsed.total ?? 0);
      else if (name === "free" && parsed.platform) {
        const p = String(parsed.platform);
        setFreePlatforms((prev) => (prev.includes(p) ? prev : [...prev, p]));
      }
      else if (name === "start" && typeof parsed.total === "number") {
        setTotal(parsed.total);
        setChecked(0);
      } else if (name === "warn" && parsed.platform) {
        const w = parsed as WarnInfo;
        setWarns((prev) =>
          prev.some((p) => p.platform === w.platform && p.nick === w.nick && p.reason === w.reason)
            ? prev
            : [...prev, w],
        );
      } else if (name === "done") {
        if (typeof parsed.total === "number") setTotal(parsed.total);
        if (typeof parsed.rates === "number" || typeof parsed.errors === "number") {
          setWarnCounts({
            rates: parsed.rates ?? 0,
            errors: parsed.errors ?? 0,
            unknowns: parsed.unknowns ?? 0,
          });
        }
        // Congela o cronômetro no tempo final — sem isso o intervalo
        // continuava atualizando `elapsed` para sempre.
        if (startedAtRef.current) {
          setElapsed((performance.now() - startedAtRef.current) / 1000);
        }
        setPhase("done");
      }
    } catch {
      /* ignore malformed chunk */
    }
  }, []);

  // Assinatura do stream: liga/desliga timers + reader conforme a busca.
  // O reset de estado vive no render acima; aqui só há I/O e callbacks.
  useEffect(() => {
    const query = initialQuery.trim();
    if (!query) return;
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout> | null = null;
    let reader: ReadableStreamDefaultReader<Uint8Array> | null = null;
    stopRef.current = false;
    readerRef.current = null;

    startedAtRef.current = performance.now();
    if (clockRef.current !== null) window.clearInterval(clockRef.current);
    clockRef.current = window.setInterval(() => {
      if (!cancelled) setElapsed((performance.now() - startedAtRef.current) / 1000);
    }, 100);

    if (live) {
      // Fetch-ready adapter: reads text/event-stream with zero UI changes.
      (async () => {
        try {
          const plats = (initialPlatforms ?? "all").trim() || "all";
          const res = await fetch(`/api/l96a1/search?q=${encodeURIComponent(query)}&platforms=${encodeURIComponent(plats)}`);
          if (!res.ok) {
            const err = await res.json().catch(() => ({}));
            if (res.status === 401) router.push("/login");
            setAuthError(
              typeof err?.error === "string" ? err.error : `busca falhou (HTTP ${res.status})`,
            );
            setPhase("done");
            return;
          }
          if (!res.body) return;
          reader = res.body.getReader();
          readerRef.current = reader;
          const decoder = new TextDecoder();
          let buffer = "";
          while (!cancelled && !stopRef.current) {
            const { done, value } = await reader.read();
            if (done) break;
            buffer += decoder.decode(value, { stream: true });
            const blocks = buffer.split("\n\n");
            buffer = blocks.pop() ?? "";
            for (const block of blocks) {
              const name = block.match(/^event:\s*(.+)$/m)?.[1]?.trim();
              const data = block.match(/^data:\s*(.+)$/m)?.[1];
              if (name && data) applyEvent(name, data);
            }
          }
        } catch {
          if (!cancelled) setPhase("done");
        }
      })();
    } else {
      // Simulação local RESPEITANDO plataformas (antes ignorava e checava tudo).
      const events = buildStreamEventsFiltered(query, (initialPlatforms ?? "all").trim() || "all");
      let i = 0;
      const tick = () => {
        if (cancelled) return;
        if (i >= events.length) return;
        const ev = events[i++];
        applyEvent(ev.name, ev.data);
        const delay = ev.name === "result" ? 380 : ev.name === "checked" ? 20 : 110;
        timer = setTimeout(tick, delay);
      };
      timer = setTimeout(tick, 450);
    }

    return () => {
      cancelled = true;
      readerRef.current = null;
      if (clockRef.current !== null) {
        window.clearInterval(clockRef.current);
        clockRef.current = null;
      }
      if (timer) clearTimeout(timer);
      reader?.cancel().catch(() => undefined);
    };
  }, [initialQuery, live, initialPlatforms, applyEvent, stopClock, router]);

  // Rede de segurança: qualquer transição para "done" (inclui erros HTTP
  // que chamam setPhase("done") sem evento "done") para o relógio.
  useEffect(() => {
    if (phase === "done") stopClock();
  }, [phase, stopClock]);

  /* ------------------------------ derived UI ------------------------------ */
  const visible = useMemo(() => {
    const q = text.trim().toLowerCase();
    let list = results.filter((r) => {
      if (cats.size > 0 && !cats.has(r.category)) return false;
      if (!q) return true;
      return [r.platform, r.username, r.displayName ?? "", r.location ?? ""]
        .join(" ")
        .toLowerCase()
        .includes(q);
    });
    list = [...list].sort((a, b) => {
      switch (sort) {
        case "followers": return (b.followers ?? -1) - (a.followers ?? -1);
        case "posts": return (b.posts ?? -1) - (a.posts ?? -1);
        case "newest": return (b.createdAt ?? "").localeCompare(a.createdAt ?? "");
        case "az": return a.platform.localeCompare(b.platform);
        default: return a.displayOrder - b.displayOrder;
      }
    });
    return list;
  }, [results, text, cats, sort]);

  const discovered = useMemo(() => results.slice(-3).reverse(), [results]);
  const streaming = phase === "streaming";
  const done = phase === "done";
  const elapsedLabel = elapsed.toFixed(1);

  useEffect(() => {
    const close = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) setOpenMenu(null);
    };
    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  }, []);

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const q = input.trim();
    if (!q) return;
    const plats = platforms.trim() || "all";
    router.replace(`/dashboard/search?q=${encodeURIComponent(q)}${live ? "&live=1" : ""}&platforms=${encodeURIComponent(plats)}`);
  };

  const doExport = (kind: "json" | "csv" | "txt") => {
    const stamp = new Date().toISOString().slice(0, 19).replace(/[:T]/g, "-");
    const base = `l96a1-${initialQuery || "search"}-${stamp}`;
    if (kind === "json") downloadFile(`${base}.json`, resultsToJSON(visible, initialQuery), "application/json");
    if (kind === "csv") downloadFile(`${base}.csv`, resultsToCSV(visible), "text/csv");
    if (kind === "txt") downloadFile(`${base}.txt`, resultsToTXT(visible, initialQuery), "text/plain");
    setOpenMenu(null);
  };

  const catCounts = useMemo(() => {
    const m = new Map<Category, number>();
    for (const r of results) m.set(r.category, (m.get(r.category) ?? 0) + 1);
    return m;
  }, [results]);

  return (
    <div className="space-y-5">
      {/* ------------------------------------------------ search controls */}
      <div className="flex flex-col items-center pt-2">
        <div className="flex gap-1 rounded-full border border-line bg-pill p-1">
          {(["username", "email", "phone"] as const).map((t) => (
            <button
              key={t}
              onClick={() => setTab(t)}
              className={`rounded-full px-5 py-1.5 text-sm font-medium capitalize transition-colors ${
                tab === t ? "bg-white text-black" : "text-sub hover:text-heading"
              }`}
            >
              {t}
            </button>
          ))}
        </div>
        <form onSubmit={submit} className="mt-4 flex w-full max-w-xl gap-2">
          <div className="relative flex-1">
            <Search size={15} className="absolute top-1/2 left-3.5 -translate-y-1/2 text-mute" />
            <input
              value={input}
              onChange={(e) => setInput(e.target.value)}
              placeholder={tab === "username" ? "Enter a username..." : tab === "email" ? "Enter an email address..." : "Enter a phone number..."}
              className="w-full rounded-lg border border-line bg-input py-2.5 pl-10 pr-3 text-sm text-heading placeholder:text-mute focus:border-linestrong focus:outline-none"
            />
          </div>
          <button
            type="submit"
            className="flex items-center gap-2 rounded-lg bg-white px-4 py-2.5 text-sm font-semibold text-black transition-colors hover:bg-body"
          >
            <ArrowRight size={14} /> Search <Zap size={12} fill="currentColor" /> 1
          </button>
        </form>
        <div className="mt-3 w-full max-w-xl">
          <PlatformPicker value={platforms} onChange={setPlatforms} />
        </div>
      </div>

      {/* ------------------------------------------------------ empty state */}
      {phase === "idle" && (
        <div className="flex flex-col items-center py-14">
          <ScopeReticle size={196} />
          <h1 className="mt-10 text-[17px] font-semibold text-heading">Search social accounts</h1>
          <p className="mt-1.5 text-sm text-mute">Enter a query above to get started.</p>
        </div>
      )}

      {phase !== "idle" && (
        <>
          {authError && (
            <div className="rounded-xl border border-danger/60 bg-card px-4 py-3 text-center">
              <p className="text-sm font-semibold text-danger">{authError}</p>
              <button
                onClick={() => router.push("/pricing")}
                className="mt-2 rounded-lg bg-white px-4 py-1.5 text-[13px] font-semibold text-black hover:bg-body"
              >
                Ver planos
              </button>
            </div>
          )}
          {/* unlock banner */}
          <div className="flex flex-wrap items-center gap-x-4 gap-y-2 rounded-xl border border-line bg-card px-4 py-3">
            <div>
              <p className="flex items-center gap-1.5 text-sm font-semibold text-heading">
                <Lock size={13} className="text-mute" /> Unlock profiles and extra fields
              </p>
              <p className="mt-0.5 text-[12px] text-mute">
                Searching is included on every plan. Profile links and extra fields need a paid plan.
              </p>
            </div>
            <button
              onClick={() => router.push("/pricing")}
              className="ml-auto flex items-center gap-1.5 rounded-lg bg-white px-3.5 py-1.5 text-[13px] font-semibold text-black hover:bg-body"
            >
              Unlock <ArrowRight size={13} />
            </button>
          </div>

          {/* counter */}
          <div className="text-center">
            <p className="text-[15px]">
              <span className="font-bold text-heading">
                {results.length} profile{results.length === 1 ? "" : "s"} found
              </span>
              <span className="text-mute"> · {elapsedLabel}s</span>
              {streaming && (
                <span className="ml-2 font-mono text-[11px] text-mute">
                  checked {Math.min(checked, total)}/{total}
                </span>
              )}
            </p>
            {streaming && (
              <button
                onClick={stop}
                className="mt-2 rounded-lg border border-danger px-3 py-1.5 text-[12px] font-semibold text-danger hover:bg-danger/10"
              >
                Parar (salva parcial como cancelado)
              </button>
            )}
            {done && (
              <p className="mt-1 text-[13px] text-mute">
                First seen <span className="font-semibold text-heading">Nov 9, 2005</span> · Last seen{" "}
                <span className="font-semibold text-heading">Sep 30, 2026</span>
              </p>
            )}
            {/* discovered pills */}
            {!hidePills && discovered.length > 0 && (
              <div className="mt-3 flex flex-wrap items-center justify-center gap-1.5 text-[12px]">
                <span className="text-mute">Discovered:</span>
                {discovered.map((r) => (
                  <span key={r.platform} className="card-enter rounded-full border border-line bg-pill px-2.5 py-0.5 font-mono text-[10.5px] text-sub">
                    @{r.username} · {r.platform}
                  </span>
                ))}
                <button onClick={() => setHidePills(true)} className="rounded-full border border-line px-2.5 py-0.5 text-[10.5px] text-mute hover:text-sub">
                  + Hide
                </button>
              </div>
            )}
          </div>

          {freePlatforms.length > 0 && (
            <ClaimBox
              frees={freePlatforms.map((p) => ({ nick: initialQuery.trim(), platform: p }))}
              title={`single ${initialQuery.trim()}`}
            />
          )}

          {warns.length > 0 && <RateBox warns={warns} title={`single ${initialQuery.trim()}`} />}

          {/* aggregators */}
          <Aggregators results={results} />

          {/* toolbar */}
          <div ref={menuRef} className="flex flex-wrap items-center gap-2">
            <div className="relative min-w-[200px] flex-1">
              <Search size={13} className="absolute top-1/2 left-3 -translate-y-1/2 text-mute" />
              <input
                value={text}
                onChange={(e) => setText(e.target.value)}
                placeholder="Filter results..."
                className="w-full rounded-lg border border-line bg-input py-2 pl-8.5 pr-3 text-[13px] text-heading placeholder:text-mute focus:border-linestrong focus:outline-none"
              />
            </div>

            <div className="relative">
              <button
                onClick={() => setOpenMenu(openMenu === "filters" ? null : "filters")}
                className={`flex items-center gap-1.5 rounded-lg border px-3 py-2 text-[13px] transition-colors ${
                  cats.size ? "border-white text-heading" : "border-line text-sub hover:border-linestrong hover:text-heading"
                }`}
              >
                <Funnel size={13} /> Filters {cats.size > 0 && `(${cats.size})`}
              </button>
              {openMenu === "filters" && (
                <div className="absolute right-0 z-30 mt-2 w-72 rounded-xl border border-line bg-raised p-3 shadow-xl">
                  <p className="eyebrow mb-2">PLATFORM GROUPS</p>
                  <div className="flex flex-wrap gap-1.5">
                    {(Object.keys(CATEGORY_LABELS) as Category[]).map((c) => {
                      const on = cats.has(c);
                      return (
                        <button
                          key={c}
                          onClick={() => {
                            const next = new Set(cats);
                            if (on) next.delete(c); else next.add(c);
                            setCats(next);
                          }}
                          className={`rounded-full border px-2.5 py-1 font-mono text-[11px] transition-colors ${
                            on ? "border-white bg-white text-black" : "border-line text-sub hover:border-linestrong hover:text-heading"
                          }`}
                        >
                          {CATEGORY_LABELS[c]} <span className={on ? "text-black/60" : "text-mute"}>{catCounts.get(c) ?? 0}</span>
                        </button>
                      );
                    })}
                  </div>
                  <div className="mt-3 flex items-center justify-between border-t border-line pt-2.5">
                    <span className="text-[11px] text-mute">{catCounts.size} groups</span>
                    <button
                      onClick={() => setCats(new Set())}
                      className="text-[11px] text-sub underline underline-offset-2 hover:text-heading"
                    >
                      Clear all
                    </button>
                  </div>
                </div>
              )}
            </div>

            <span className="text-[12px] text-mute">{visible.length} found</span>

            <div className="ml-auto flex items-center gap-1.5">
              {/* sort */}
              <div className="relative">
                <button
                  onClick={() => setOpenMenu(openMenu === "sort" ? null : "sort")}
                  aria-label="Sort"
                  className={`grid size-9 place-items-center rounded-lg border transition-colors ${
                    sort !== "default" ? "border-white text-heading" : "border-line text-sub hover:border-linestrong hover:text-heading"
                  }`}
                >
                  <ArrowUpDown size={14} />
                </button>
                {openMenu === "sort" && (
                  <div className="absolute right-0 z-30 mt-2 w-44 rounded-xl border border-line bg-raised p-1.5 shadow-xl">
                    {(Object.keys(SORT_LABELS) as SortMode[]).map((m) => (
                      <button
                        key={m}
                        onClick={() => { setSort(m); setOpenMenu(null); }}
                        className={`flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-[13px] ${
                          sort === m ? "text-heading" : "text-sub hover:bg-card hover:text-heading"
                        }`}
                      >
                        {SORT_LABELS[m]}
                        {sort === m && <span className="font-mono text-[10px]">✓</span>}
                      </button>
                    ))}
                  </div>
                )}
              </div>

              {/* view toggle */}
              <button
                onClick={() => setView("table")}
                aria-label="Table view"
                className={`grid size-9 place-items-center rounded-lg border transition-colors ${
                  view === "table" ? "border-white text-heading" : "border-line text-sub hover:border-linestrong hover:text-heading"
                }`}
              >
                <Table2 size={14} />
              </button>
              <button
                onClick={() => setView("cards")}
                aria-label="Card view"
                className={`grid size-9 place-items-center rounded-lg border transition-colors ${
                  view === "cards" ? "border-white text-heading" : "border-line text-sub hover:border-linestrong hover:text-heading"
                }`}
              >
                <LayoutGrid size={14} />
              </button>

              {/* export */}
              <div className="relative">
                <button
                  onClick={() => setOpenMenu(openMenu === "export" ? null : "export")}
                  className="flex items-center gap-1.5 rounded-lg border border-line px-3 py-2 text-[13px] text-sub transition-colors hover:border-linestrong hover:text-heading"
                >
                  <Download size={13} /> Export <ChevronDown size={12} />
                </button>
                {openMenu === "export" && (
                  <div className="absolute right-0 z-30 mt-2 w-44 rounded-xl border border-line bg-raised p-1.5 shadow-xl">
                    {([
                      ["json", "JSON", FileJson],
                      ["csv", "CSV", Table2],
                      ["txt", "TXT", FileText],
                    ] as const).map(([kind, label, Icon]) => (
                      <button
                        key={kind}
                        onClick={() => doExport(kind)}
                        className="flex w-full items-center gap-2.5 rounded-lg px-3 py-2 text-left text-[13px] text-sub hover:bg-card hover:text-heading"
                      >
                        <Icon size={13} /> Export as {label}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* results */}
          {view === "cards" ? (
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {visible.map((r) => (
                <ResultCard key={r.platform} result={r} flash={flash === r.platform} onOpen={setModal} />
              ))}
            </div>
          ) : (
            <div className="overflow-x-auto rounded-xl border border-line bg-card">
              <table className="w-full min-w-[640px] text-sm">
                <thead>
                  <tr className="border-b border-line text-left font-mono text-[10px] tracking-widest text-mute">
                    {["PLATFORM", "HANDLE", "FOLLOWERS", "LOCATION", "CREATED", "ACTIVE"].map((h) => (
                      <th key={h} className="px-4 py-3 font-medium">{h}</th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {visible.map((r) => (
                    <tr
                      key={r.platform}
                      onClick={() => r.hasChecker && setModal(r)}
                      className={`border-b border-line/60 last:border-0 ${
                        r.hasChecker ? "cursor-pointer hover:bg-raised" : "opacity-45"
                      }`}
                    >
                      <td className="px-4 py-3">
                        <span className="flex items-center gap-2">
                          {/* eslint-disable-next-line @next/next/no-img-element */}
                          <img src={faviconUrl(r.platformUrl)} alt="" width={14} height={14} loading="lazy" className="rounded-sm" />
                          <span className="font-mono text-[11px] font-semibold tracking-wider text-heading uppercase">{r.platform}</span>
                          {!r.hasChecker && <span className="rounded-full border border-line px-1.5 font-mono text-[8px] text-mute">SOON</span>}
                        </span>
                      </td>
                      <td className="px-4 py-3 font-mono text-[12px] text-sub">@{r.username}</td>
                      <td className="px-4 py-3 font-mono text-[12px] text-sub">{r.followers !== null ? fmtCount(r.followers) : "—"}</td>
                      <td className="px-4 py-3 text-[12px] text-sub">{r.location ?? "—"}</td>
                      <td className="px-4 py-3 text-[12px] text-sub">{fmtDate(r.createdAt) ?? "—"}</td>
                      <td className="px-4 py-3 text-[12px] text-sub">{fmtDate(r.lastActive) ?? "—"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* streaming skeleton — rifle shimmer while cards arrive */}
          {streaming && (
            <div className="flex flex-col items-center gap-4 py-12">
              <ScopeReticle size={128} scanning labels={false} />
              <p className="font-mono text-[11px] tracking-[0.25em] text-mute">
                SCANNING {Math.min(checked, total)}/{total} — {results.length} FOUND
              </p>
            </div>
          )}

          {done && visible.length === 0 && (
            <div className="rounded-xl border border-line bg-card px-6 py-10 text-center">
              <ListPlus size={20} className="mx-auto text-mute" />
              <p className="mt-3 text-sm text-heading">No results match the active filters.</p>
              <button
                onClick={() => { setText(""); setCats(new Set()); }}
                className="mt-2 flex items-center gap-1.5 mx-auto text-[12px] text-mute underline underline-offset-2 hover:text-sub"
              >
                <X size={11} /> Clear filters
              </button>
            </div>
          )}
        </>
      )}

      <ProfileModal result={modal} onClose={() => setModal(null)} />
    </div>
  );
}
