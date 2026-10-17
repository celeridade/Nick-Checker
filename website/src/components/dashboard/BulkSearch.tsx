"use client";

import { useCallback, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import { ArrowRight, Download, Upload, Zap } from "lucide-react";
import ResultCard from "@/components/dashboard/ResultCard";
import ProfileModal from "@/components/dashboard/ProfileModal";
import PlatformPicker from "@/components/dashboard/PlatformPicker";
import RateBox from "@/components/dashboard/RateBox";
import type { SearchResult, WarnInfo } from "@/lib/results";
import { downloadFile, resultsToCSV, resultsToJSON, resultsToTXT } from "@/lib/export";
import { freesToTXTCompact, type FreeNick } from "@/lib/available";
import ClaimBox from "@/components/dashboard/ClaimBox";

function parseUsers(raw: string): string[] {
  // Reusa a regra do CLI read_user_file/parse_users_arg: vírgula, linha, ; — ignora # e vazias.
  // Tenta JSON ["a","b"] ou [{username}] antes do split simples.
  const t = raw.trim();
  if (!t) return [];
  try {
    const v = JSON.parse(t);
    if (Array.isArray(v)) {
      const out: string[] = [];
      for (const item of v) {
        if (typeof item === "string" && item.trim()) out.push(item.trim());
        else if (item && typeof item.username === "string" && item.username.trim())
          out.push(item.username.trim());
      }
      if (out.length > 0) return [...new Set(out)].slice(0, 500);
    }
  } catch {
    // não é JSON — segue split
  }
  return [
    ...new Set(
      t
        .split(/[\n,;]+/)
        .map((s) => s.trim().replace(/^@/, ""))
        .filter((s) => s && !s.startsWith("#")),
    ),
  ].slice(0, 500);
}

export default function BulkSearch() {
  const [raw, setRaw] = useState("");
  const [platforms, setPlatforms] = useState("all");
  const [phase, setPhase] = useState<"idle" | "streaming" | "polling" | "done">("idle");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [frees, setFrees] = useState<FreeNick[]>([]);
  const [warns, setWarns] = useState<WarnInfo[]>([]);
  const [checked, setChecked] = useState(0);
  const [total, setTotal] = useState(0);
  const [jobId, setJobId] = useState<string | null>(null);
  const [modal, setModal] = useState<SearchResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const abortRef = useRef<{ cancelled: boolean }>({ cancelled: false });
  const fileRef = useRef<HTMLInputElement>(null);
  const router = useRouter();

  const users = parseUsers(raw);

  const applyEvent = useCallback((name: string, data: string) => {
    try {
      const parsed = JSON.parse(data);
      if (name === "result" && parsed.platform) {
        const r = parsed as SearchResult;
        setResults((prev) =>
          prev.some((p) => p.platform === r.platform && p.username === r.username)
            ? prev
            : [...prev, r],
        );
      } else if (name === "checked") setChecked(parsed.total ?? 0);
      else if (name === "free" && parsed.nick && parsed.platform) {
        const f = parsed as FreeNick;
        setFrees((prev) =>
          prev.some((p) => p.nick === f.nick && p.platform === f.platform) ? prev : [...prev, f],
        );
      } else if (name === "start" && typeof parsed.total === "number") {
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
        setPhase("done");
      }
    } catch {
      /* ignora chunk */
    }
  }, []);

  const stop = async () => {
    abortRef.current.cancelled = true;
    // Job (polling): avisa o motor p/ parar entre chunks e salvar parcial
    // como cancelled no history. Stream direto: só para localmente.
    if (jobId && (phase === "polling")) {
      try {
        await fetch(`/api/l96a1/scans/${encodeURIComponent(jobId)}/cancel`, { method: "POST" });
      } catch {
        /* polling já parou localmente */
      }
    }
    setPhase("done");
  };

  /** SSE direto: divide em chunks de 100 e checa de 100 em 100. */
  const runStream = async () => {
    const list = parseUsers(raw);
    if (list.length === 0) {
      setError("Cole 1+ nicks (1 por linha ou a,b,c).");
      return;
    }
    setError(null);
    abortRef.current.cancelled = false;
    setPhase("streaming");
    setResults([]);
    setFrees([]);
    setWarns([]);
    setChecked(0);
    setTotal(0);
    setJobId(null);
    const chunks: string[][] = [];
    for (let i = 0; i < list.length; i += 100) chunks.push(list.slice(i, i + 100));
    let totalAcc = 0;
    let checkedAcc = 0;
    try {
      for (let ci = 0; ci < chunks.length; ci++) {
        if (abortRef.current.cancelled) break;
        const chunk = chunks[ci];
        const res = await fetch(
          `/api/l96a1/bulk?users=${encodeURIComponent(chunk.join(","))}&platforms=${encodeURIComponent(platforms || "all")}`,
        );
        if (!res.ok) {
          const err = await res.json().catch(() => ({}));
          if (res.status === 401) router.push("/login");
          throw new Error(typeof err?.error === "string" ? err.error : `HTTP ${res.status}`);
        }
        if (!res.body) continue;
        const reader = res.body.getReader();
        const decoder = new TextDecoder();
        let buffer = "";
        let chunkTotal = 0;
        for (;;) {
          if (abortRef.current.cancelled) {
            reader.cancel().catch(() => undefined);
            break;
          }
          const { done, value } = await reader.read();
          if (done) break;
          buffer += decoder.decode(value, { stream: true });
          const blocks = buffer.split("\n\n");
          buffer = blocks.pop() ?? "";
          for (const block of blocks) {
            const name = block.match(/^event:\s*(.+)$/m)?.[1]?.trim();
            const data = block.match(/^data:\s*(.+)$/m)?.[1];
            if (!name || !data) continue;
            if (name === "start") {
              try {
                const p = JSON.parse(data);
                chunkTotal = typeof p.total === "number" ? p.total : 0;
                totalAcc += chunkTotal;
                setTotal(totalAcc);
              } catch {
                /* ignora */
              }
            } else if (name === "checked") {
              try {
                const p = JSON.parse(data);
                setChecked(checkedAcc + (p.total ?? 0));
              } catch {
                /* ignora */
              }
            } else if (name === "done") {
              checkedAcc += chunkTotal;
              setChecked(checkedAcc);
              // não propaga done parcial p/ applyEvent (só no fim)
            } else {
              applyEvent(name, data);
            }
          }
        }
      }
      setPhase("done");
    } catch (e) {
      setError(e instanceof Error ? e.message : "falha no stream (engine offline?)");
      setPhase("done");
    }
  };

  /** Job com polling (100–500 nicks, arquivo, dataset): POST /api/l96a1/scans. */
  const runJob = async () => {
    const list = parseUsers(raw);
    if (list.length === 0) {
      setError("Cole 1+ nicks.");
      return;
    }
    setError(null);
    abortRef.current.cancelled = false;
    setPhase("polling");
    setResults([]);
    setFrees([]);
    setWarns([]);
    setChecked(0);
    setTotal(0);
    try {
      const res = await fetch("/api/l96a1/scans", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ usernames: list, platforms: platforms || "all", source: "bulk" }),
      });
      const data = await res.json();
      if (!res.ok) {
        if (res.status === 401) router.push("/login");
        throw new Error(data?.error ?? `HTTP ${res.status}`);
      }
      setJobId(data.jobId);
      setTotal(data.total ?? 0);
      for (;;) {
        if (abortRef.current.cancelled) break;
        await new Promise((r) => setTimeout(r, 1500));
        const poll = await fetch(`/api/l96a1/scans/${encodeURIComponent(data.jobId)}`);
        const job = await poll.json();
        if (!poll.ok) throw new Error(job?.error ?? `HTTP ${poll.status}`);
        setChecked(job.checked ?? 0);
        setTotal(job.total ?? 0);
        if (Array.isArray(job.results)) setResults(job.results);
        if (Array.isArray(job.available)) setFrees(job.available);
        if (Array.isArray(job.warnings)) setWarns(job.warnings);
        if (job.status === "done" || job.status === "error" || job.status === "cancelled") {
          setPhase("done");
          if (job.status === "error") setError(job.error ?? "job falhou");
          break;
        }
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "falha ao criar job");
      setPhase("done");
    }
  };

  const onFiles = async (files: FileList | File[] | undefined) => {
    if (!files) return;
    const list = Array.from(files);
    if (list.length === 0) return;
    const texts = await Promise.all(list.map((f) => f.text()));
    const mergedLines: string[] = [];
    for (const text of texts) {
      // txt/md/csv: 1ª coluna; json: reutiliza parseUsers.
      try {
        const v = JSON.parse(text.trim());
        if (Array.isArray(v)) {
          for (const item of v) {
            if (typeof item === "string" && item.trim()) mergedLines.push(item.trim());
            else if (item && typeof item.username === "string" && item.username.trim())
              mergedLines.push(item.username.trim());
          }
          continue;
        }
      } catch {
        /* não é JSON — segue split */
      }
      for (const l of text.split("\n")) {
        const t = l.trim();
        if (!t || t.startsWith("#")) continue;
        const cell = t.split(/[,;]/)[0]?.trim().replace(/^["']|["']$/g, "") ?? "";
        if (cell) mergedLines.push(cell);
      }
    }
    const merged = [...new Set([...parseUsers(raw), ...parseUsers(mergedLines.join("\n"))])].slice(0, 500);
    setRaw(merged.join("\n"));
  };

  const onFile = async (f: File | undefined) => {
    if (!f) return;
    await onFiles([f]);
  };

  const doExport = (kind: "json" | "csv" | "txt") => {
    const stamp = new Date().toISOString().slice(0, 19).replace(/[:T]/g, "-");
    if (kind === "json") downloadFile(`l96a1-bulk-${stamp}.json`, resultsToJSON(results, `${users.length} nicks`), "application/json");
    if (kind === "csv") downloadFile(`l96a1-bulk-${stamp}.csv`, resultsToCSV(results), "text/csv");
    if (kind === "txt") downloadFile(`l96a1-bulk-${stamp}.txt`, resultsToTXT(results, `${users.length} nicks`), "text/plain");
  };

  const busy = phase === "streaming" || phase === "polling";
  const byNick = new Map<string, SearchResult[]>();
  for (const r of results) {
    const k = r.sourceQuery || r.username;
    if (!byNick.has(k)) byNick.set(k, []);
    byNick.get(k)!.push(r);
  }

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-bold text-heading">Bulk Scan</h1>
        <p className="mt-0.5 text-sm text-mute">
          1 nick por linha ou <span className="font-mono">a,b,c</span> · vários arquivos txt/csv/json · Stream divide de 100 em 100, job até 500.
        </p>
      </div>

      <div className="rounded-xl border border-line bg-card p-4">
        <div className="grid gap-3 lg:grid-cols-[1fr_220px]">
          <textarea
            value={raw}
            onChange={(e) => setRaw(e.target.value)}
            rows={6}
            placeholder={"adoecido\ntorvalds\nhikaru"}
            spellCheck={false}
            className="w-full rounded-lg border border-line bg-input p-3 font-mono text-[13px] text-heading placeholder:text-mute focus:border-linestrong focus:outline-none"
          />
          <div className="space-y-2">
            <div className="block text-[12px] text-mute">
              <span className="mb-1 block">Plataformas</span>
              <PlatformPicker value={platforms} onChange={setPlatforms} />
            </div>
            <div className="flex gap-2">
              <button
                onClick={runStream}
                disabled={busy}
                className="flex flex-1 items-center justify-center gap-1.5 rounded-lg bg-white px-3 py-2 text-[13px] font-semibold text-black hover:bg-body disabled:opacity-50"
              >
                <ArrowRight size={13} /> Stream <Zap size={11} fill="currentColor" />
              </button>
              <button
                onClick={runJob}
                disabled={busy}
                className="flex flex-1 items-center justify-center gap-1.5 rounded-lg border border-linestrong px-3 py-2 text-[13px] font-semibold text-body hover:border-white hover:text-white disabled:opacity-50"
              >
                Criar job
              </button>
            </div>
            <div className="flex gap-2">
              <button
                onClick={() => fileRef.current?.click()}
                className="flex flex-1 items-center justify-center gap-1.5 rounded-lg border border-line px-3 py-2 text-[12px] text-sub hover:border-linestrong hover:text-heading"
              >
                <Upload size={12} /> Arquivo
              </button>
              {busy ? (
                <button
                  onClick={stop}
                  className="flex-1 rounded-lg border border-danger px-3 py-2 text-[12px] font-semibold text-danger hover:bg-danger/10"
                >
                  Parar
                </button>
              ) : (
                results.length > 0 && (
                  <button
                    onClick={() => doExport("json")}
                    className="flex flex-1 items-center justify-center gap-1.5 rounded-lg border border-line px-3 py-2 text-[12px] text-sub hover:border-linestrong hover:text-heading"
                  >
                    <Download size={12} /> JSON
                  </button>
                )
              )}
            </div>
            <input
              ref={fileRef}
              type="file"
              accept=".txt,.md,.csv,.json"
              multiple
              className="hidden"
              onChange={(e) => {
                onFiles(e.target.files ?? undefined);
                e.target.value = "";
              }}
            />
          </div>
        </div>
        <div className="mt-2 flex flex-wrap items-center gap-2 font-mono text-[11px] text-mute">
          <span>{users.length} nicks</span>
          <span className="text-line">|</span>
          <span>
            checked {checked}/{total || "—"}
          </span>
          <span className="text-line">|</span>
          <span>{results.length} hits</span>
          <span className="text-line">|</span>
          <span className="text-found">{new Set(frees.map((f) => f.nick)).size} livres</span>
          {jobId && (
            <>
              <span className="text-line">|</span>
              <span>job {jobId}</span>
            </>
          )}
          {error && <span className="text-danger">{error}</span>}
        </div>
      </div>

      {frees.length > 0 && (
        <ClaimBox frees={frees} title={`bulk ${users.length} nicks`} />
      )}

      {users.length > 100 && frees.length > 0 && (
        <div className="rounded-xl border border-found/40 bg-card p-4">
          <div className="flex flex-wrap items-center gap-2">
            <p className="text-sm font-semibold text-heading">
              Resultado compacto <span className="font-mono text-[11px] font-normal text-mute">1 arquivo · nick + plataformas</span>
            </p>
            <button
              onClick={() => {
                const stamp = new Date().toISOString().slice(0, 19).replace(/[:T]/g, "-");
                downloadFile(
                  `l96a1-bulk-compacto-${users.length}nicks-${stamp}.txt`,
                  freesToTXTCompact(frees, `bulk ${users.length} nicks`),
                  "text/plain",
                );
              }}
              className="ml-auto flex items-center gap-1.5 rounded-lg bg-white px-4 py-2 text-[13px] font-semibold text-black hover:bg-body"
            >
              <Download size={13} /> Baixar compacto (.txt)
            </button>
          </div>
        </div>
      )}

      {warns.length > 0 && <RateBox warns={warns} title={`bulk ${users.length} nicks`} />}

      {byNick.size > 0 && (
        <div className="space-y-6">
          {[...byNick.entries()].map(([nick, list]) => (
            <div key={nick}>
              <p className="mb-2 font-mono text-[12px] text-sub">
                <span className="text-heading">@{nick}</span> · {list.length} hits
              </p>
              <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                {list.map((r) => (
                  <ResultCard key={`${r.username}-${r.platform}`} result={r} onOpen={setModal} />
                ))}
              </div>
            </div>
          ))}
        </div>
      )}

      <ProfileModal result={modal} onClose={() => setModal(null)} />
    </div>
  );
}
