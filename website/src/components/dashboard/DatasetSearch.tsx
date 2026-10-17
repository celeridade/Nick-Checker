"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import { ArrowRight, Check, Download, FileText, Folder, FolderOpen, Minus, Search, Zap } from "lucide-react";
import ResultCard from "@/components/dashboard/ResultCard";
import ProfileModal from "@/components/dashboard/ProfileModal";
import type { SearchResult, WarnInfo } from "@/lib/results";
import { downloadFile, resultsToCSV, resultsToJSON, resultsToTXT } from "@/lib/export";
import type { FreeNick } from "@/lib/available";
import ClaimBox from "@/components/dashboard/ClaimBox";
import RateBox from "@/components/dashboard/RateBox";

interface DatasetFile {
  path: string;
  count: number;
}

interface TreeFile {
  path: string;
  count: number;
  rel: string;
  leaf: string;
}

interface TreeNode {
  name: string;
  dirs: Map<string, TreeNode>;
  files: TreeFile[];
}

/** Cache local da lista (SWR): recarregar a página pinta na hora e revalida atrás. */
const FILES_CACHE_KEY = "l96a1-dataset-files-v1";

function readFilesCache(): { files: DatasetFile[]; dir: string } {
  try {
    const raw = localStorage.getItem(FILES_CACHE_KEY);
    if (!raw) return { files: [], dir: "" };
    const j = JSON.parse(raw) as { files?: unknown; dir?: unknown };
    if (!Array.isArray(j.files)) return { files: [], dir: "" };
    const files = j.files
      .filter(
        (f): f is DatasetFile =>
          !!f && typeof (f as DatasetFile).path === "string" && typeof (f as DatasetFile).count === "number",
      )
      .slice(0, 6000);
    return { files, dir: typeof j.dir === "string" ? j.dir : "" };
  } catch {
    return { files: [], dir: "" };
  }
}

function writeFilesCache(files: DatasetFile[], dir: string) {
  try {
    localStorage.setItem(FILES_CACHE_KEY, JSON.stringify({ at: Date.now(), dir, files }));
  } catch {
    /* quota cheia ou privada — segue sem cache */
  }
}

function collectPaths(n: TreeNode): string[] {
  const out: string[] = n.files.map((f) => f.path);
  for (const c of n.dirs.values()) out.push(...collectPaths(c));
  return out;
}

/** Checkbox custom (fora do padrão do navegador). */
function CheckBox({
  checked,
  indeterminate,
  onToggle,
  label,
}: {
  checked: boolean;
  indeterminate?: boolean;
  onToggle: () => void;
  label: string;
}) {
  return (
    <button
      onClick={(e) => {
        e.stopPropagation();
        onToggle();
      }}
      aria-label={label}
      aria-checked={checked}
      className={`grid size-[18px] shrink-0 place-items-center rounded-md border transition-colors ${
        checked || indeterminate
          ? "border-white bg-white text-black"
          : "border-linestrong bg-input text-transparent hover:border-white"
      }`}
    >
      {indeterminate ? <Minus size={12} strokeWidth={3} /> : <Check size={12} strokeWidth={3} />}
    </button>
  );
}

function FolderView({
  node,
  depth,
  picked,
  onToggleFile,
  onToggleFolder,
  expandCmd,
}: {
  node: TreeNode;
  depth: number;
  picked: Set<string>;
  onToggleFile: (p: string) => void;
  onToggleFolder: (n: TreeNode) => void;
  expandCmd: { v: boolean; n: number } | null;
}) {
  const [open, setOpen] = useState(depth < 2);
  // Comando externo expandir/recolher tudo: ajuste durante o render
  // (padrão React p/ "resetar estado quando a prop muda" — sem efeito).
  const [prevCmd, setPrevCmd] = useState(expandCmd);
  if (expandCmd !== prevCmd) {
    setPrevCmd(expandCmd);
    if (expandCmd) setOpen(expandCmd.v);
  }
  const all = collectPaths(node);
  const sel = all.filter((p) => picked.has(p)).length;
  const allSel = all.length > 0 && sel === all.length;
  const someSel = sel > 0 && !allSel;
  const dirEntries = [...node.dirs.entries()].sort((a, b) => a[0].localeCompare(b[0]));
  const FolderIcon = open ? FolderOpen : Folder;
  return (
    <div className={depth > 0 ? "ml-4 border-l border-line/70 pl-1" : ""}>
      {node.name && (
        <div
          className={`group flex cursor-pointer items-center gap-2 rounded-lg px-1.5 py-1.5 transition-colors hover:bg-raised ${someSel ? "bg-raised/50" : ""}`}
          onClick={() => setOpen(!open)}
        >
          <CheckBox
            checked={allSel}
            indeterminate={someSel}
            onToggle={() => onToggleFolder(node)}
            label={`Marcar pasta ${node.name}`}
          />
          <FolderIcon size={15} className={open ? "shrink-0 text-body" : "shrink-0 text-mute group-hover:text-sub"} />
          <span className="truncate font-mono text-[12px] font-medium text-heading">{node.name}/</span>
          <span className="shrink-0 rounded-full border border-line bg-pill px-1.5 font-mono text-[10px] text-mute">{all.length}</span>
          {sel > 0 && <span className="shrink-0 font-mono text-[10px] text-found">{sel} ✓</span>}
          <span className={`ml-auto shrink-0 text-[11px] text-mute transition-transform ${open ? "" : "-rotate-90"}`}>▾</span>
        </div>
      )}
      {(open || !node.name) && (
        <>
          {node.files.map((f) => (
            <div
              key={f.path}
              onClick={() => onToggleFile(f.path)}
              className="flex cursor-pointer items-center gap-2.5 rounded-lg px-1.5 py-1 font-mono text-[12px] text-sub transition-colors hover:bg-raised hover:text-heading"
            >
              <CheckBox checked={picked.has(f.path)} onToggle={() => onToggleFile(f.path)} label={`Marcar ${f.leaf}`} />
              <FileText size={13} className="shrink-0 text-mute" />
              <span className="truncate" title={f.rel}>{f.leaf}</span>
              <span className="ml-auto shrink-0 rounded-full bg-pill px-1.5 text-[10px] text-mute">{f.count}</span>
            </div>
          ))}
          {dirEntries.map(([name, child]) => (
            <FolderView key={name} node={child} depth={depth + 1} picked={picked} onToggleFile={onToggleFile} onToggleFolder={onToggleFolder} expandCmd={expandCmd} />
          ))}
        </>
      )}
    </div>
  );
}

export default function DatasetSearch() {
  const [files, setFiles] = useState<DatasetFile[]>([]);
  const [dir, setDir] = useState("");
  const [filter, setFilter] = useState("");
  const [picked, setPicked] = useState<Set<string>>(new Set());
  const [expandCmd, setExpandCmd] = useState<{ v: boolean; n: number } | null>(null);
  const fireExpand = (v: boolean) => setExpandCmd((p) => ({ v, n: (p?.n ?? 0) + 1 }));
  const [smart, setSmart] = useState(true);
  const [limit, setLimit] = useState(100);
  const [platforms, setPlatforms] = useState("all");
  const [jobId, setJobId] = useState<string | null>(null);
  const [phase, setPhase] = useState<"idle" | "running" | "done">("idle");
  const [checked, setChecked] = useState(0);
  const [total, setTotal] = useState(0);
  const [results, setResults] = useState<SearchResult[]>([]);
  const [frees, setFrees] = useState<FreeNick[]>([]);
  const [warns, setWarns] = useState<WarnInfo[]>([]);
  const [modal, setModal] = useState<SearchResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [cancelled, setCancelled] = useState(false);
  const abortRef = useRef(false);
  const router = useRouter();

  const [loaded, setLoaded] = useState(false);

  // Busca pura (sem setState).
  const fetchFiles = useCallback(async () => {
    const res = await fetch("/api/l96a1/dataset/files");
    const data = await res.json();
    return { files: (data.files ?? []) as DatasetFile[], dir: (data.dataset_dir ?? "") as string };
  }, []);

  // Mount: 1) hidrata do cache local (instantâneo), 2) revalida na engine.
  // setStates só em callbacks async + guarda alive (sem efeito em cascata).
  useEffect(() => {
    let alive = true;
    const tick = setTimeout(() => {
      if (!alive) return;
      const c = readFilesCache();
      if (c.files.length > 0) {
        setFiles(c.files);
        setDir(c.dir);
      }
      setLoaded(true);
    }, 0);
    const run = async () => {
      try {
        const f = await fetchFiles();
        if (!alive) return;
        setFiles(f.files);
        setDir(f.dir);
        writeFilesCache(f.files, f.dir);
      } catch {
        /* engine offline — vale o cache */
      } finally {
        if (alive) setLoaded(true);
      }
    };
    void run();
    return () => {
      alive = false;
      clearTimeout(tick);
    };
  }, [fetchFiles]);

  const visible = files.filter((f) => {
    const q = filter.trim().toLowerCase().replace(/\\/g, "/");
    if (!q) return true;
    // Normaliza `\` → `/` dos dois lados (Windows x digitação).
    return f.path.toLowerCase().replace(/\\/g, "/").includes(q);
  });

  const relOf = (p: string) => {
    const d = (dir || "").replace(/\\/g, "/").replace(/\/$/, "");
    const n = p.replace(/\\/g, "/");
    const rel = d && n.startsWith(d) ? n.slice(d.length).replace(/^\/+/, "") : n;
    return rel || n;
  };

  const buildTree = (): TreeNode => {
    const root: TreeNode = { name: "", dirs: new Map(), files: [] };
    for (const f of files) {
      const rel = relOf(f.path);
      const parts = rel.split("/").filter(Boolean);
      let node = root;
      for (let i = 0; i < parts.length - 1; i++) {
        const seg = parts[i];
        let next = node.dirs.get(seg);
        if (!next) {
          next = { name: seg, dirs: new Map(), files: [] };
          node.dirs.set(seg, next);
        }
        node = next;
      }
      node.files.push({ path: f.path, count: f.count, rel, leaf: parts[parts.length - 1] ?? rel });
    }
    const sortNode = (n: TreeNode) => {
      n.files.sort((a, b) => a.leaf.localeCompare(b.leaf));
      for (const c of n.dirs.values()) sortNode(c);
    };
    sortNode(root);
    return root;
  };

  const toggle = (path: string) => {
    const next = new Set(picked);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    setPicked(next);
  };

  const toggleFolder = (node: TreeNode) => {
    const all = collectPaths(node);
    const every = all.every((p) => picked.has(p));
    const next = new Set(picked);
    if (every) for (const p of all) next.delete(p);
    else for (const p of all) next.add(p);
    setPicked(next);
  };

  const tree = buildTree();

  const scan = async () => {
    setError(null);
    setPhase("running");
    setResults([]);
    setFrees([]);
    setWarns([]);
    setChecked(0);
    setTotal(0);
    setCancelled(false);
    abortRef.current = false;
    try {
      const body =
        picked.size > 0
          ? { files: [...picked], smart, limit, platforms }
          : { filter: filter.trim() || undefined, smart, limit, platforms };
      const res = await fetch("/api/l96a1/dataset/scan", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });
      const data = await res.json();
      if (!res.ok) {
        if (res.status === 401) router.push("/login");
        throw new Error(data?.error ?? `HTTP ${res.status}`);
      }
      setJobId(data.jobId);
      for (;;) {
        if (abortRef.current) break;
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
          if (job.status === "cancelled") setCancelled(true);
          setPhase("done");
          break;
        }
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "falha no scan");
      setPhase("done");
    }
  };

  const stop = async () => {
    abortRef.current = true;
    if (jobId) {
      try {
        await fetch(`/api/l96a1/scans/${encodeURIComponent(jobId)}/cancel`, { method: "POST" });
      } catch {
        /* o polling já parou localmente */
      }
    }
    setCancelled(true);
    setPhase("done");
  };

  const doExport = (kind: "json" | "csv" | "txt") => {
    const stamp = new Date().toISOString().slice(0, 19).replace(/[:T]/g, "-");
    if (kind === "json") downloadFile(`l96a1-dataset-${stamp}.json`, resultsToJSON(results, "dataset"), "application/json");
    if (kind === "csv") downloadFile(`l96a1-dataset-${stamp}.csv`, resultsToCSV(results), "text/csv");
    if (kind === "txt") downloadFile(`l96a1-dataset-${stamp}.txt`, resultsToTXT(results, "dataset"), "text/plain");
  };

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-bold text-heading">Dataset</h1>
        <p className="mt-0.5 font-mono text-[12px] text-mute">{dir || "carregando…"} · smart 3C/4C/rarity + boost Forebears/Trends</p>
      </div>

      <div className="rounded-xl border border-line bg-card p-4">
        <div className="flex flex-wrap items-center gap-2">
          <div className="relative min-w-[200px] flex-1">
            <Search size={13} className="absolute top-1/2 left-3 -translate-y-1/2 text-mute" />
            <input
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              placeholder="Filtrar arquivos (3C, forebears, patterns…)"
              className="w-full rounded-lg border border-line bg-input py-2 pl-8 pr-3 text-[13px] text-heading placeholder:text-mute focus:border-linestrong focus:outline-none"
            />
          </div>
          <label className="flex items-center gap-2 text-[12px] text-sub">
            <input type="checkbox" checked={smart} onChange={(e) => setSmart(e.target.checked)} />
            smart
          </label>
          <label className="flex items-center gap-2 text-[12px] text-sub">
            lote
            <input
              type="number"
              value={limit}
              min={1}
              max={100}
              title="Tamanho do lote (1–100): o scan percorre todos os candidatos de lote em lote até o final"
              onChange={(e) => setLimit(Math.min(100, Math.max(1, Number(e.target.value) || 100)))}
              className="w-20 rounded-lg border border-line bg-input px-2 py-1.5 font-mono text-[12px] text-heading focus:border-linestrong focus:outline-none"
            />
          </label>
          <input
            value={platforms}
            onChange={(e) => setPlatforms(e.target.value)}
            placeholder="all"
            className="w-36 rounded-lg border border-line bg-input px-3 py-2 font-mono text-[12px] text-heading placeholder:text-mute focus:border-linestrong focus:outline-none"
          />
          <button
            onClick={scan}
            disabled={phase === "running"}
            className="flex items-center gap-2 rounded-lg bg-white px-4 py-2 text-[13px] font-semibold text-black hover:bg-body disabled:opacity-50"
          >
            <ArrowRight size={13} /> Escanear <Zap size={11} fill="currentColor" />
          </button>
          {phase === "running" && (
            <button
              onClick={stop}
              className="rounded-lg border border-danger px-4 py-2 text-[13px] font-semibold text-danger hover:bg-danger/10"
            >
              Parar
            </button>
          )}
        </div>
        <p className="mt-2 font-mono text-[11px] text-mute">
          {picked.size > 0 ? `${picked.size} arquivos marcados` : "nada marcado = usa o filtro"} · lote {limit} · checked {checked}/{total || "—"} · {results.length} hits · <span className="text-found">{new Set(frees.map((f) => f.nick)).size} livres</span>
          {jobId && ` · job ${jobId}`}
          {cancelled && <span className="text-amber-300"> · cancelado (parcial salvo no history)</span>}
          {error && <span className="text-danger"> · {error}</span>}
        </p>
      </div>

      {frees.length > 0 && (
        <ClaimBox frees={frees} title={`dataset lote ${limit}`} />
      )}

      {warns.length > 0 && <RateBox warns={warns} title={`dataset lote ${limit}`} />}

      {!loaded && (
        <div className="rounded-xl border border-line bg-card p-4">
          <p className="font-mono text-[12px] text-mute">Carregando dataset…</p>
        </div>
      )}

      {loaded && files.length === 0 && (
        <div className="rounded-xl border border-amber-500/40 bg-card p-4">
          <p className="text-[13px] text-sub">
            Nenhum arquivo no dataset{dir ? <> em <span className="font-mono text-heading">{dir}</span></> : ""}.
            <span className="text-mute"> Verifique se a engine está rodando e o DATASET_DIR aponta para a pasta com os .txt</span>
          </p>
        </div>
      )}

      <div className="rounded-xl border border-line bg-card p-4">
        <div className="mb-2 flex flex-wrap items-center gap-2 font-mono text-[11px] text-mute">
          <span>{files.length} arquivos</span>
          <span className="text-line">|</span>
          <span>{picked.size} marcados</span>
          {filter.trim() && (
            <>
              <span className="text-line">|</span>
              <span>{visible.length} batem no filtro</span>
            </>
          )}
          {picked.size > 0 && (
            <button onClick={() => setPicked(new Set())} className="ml-auto underline underline-offset-2 hover:text-sub">
              limpar
            </button>
          )}
          {!filter.trim() && files.length > 0 && (
            <span className="ml-auto flex gap-2">
              <button onClick={() => fireExpand(true)} className="underline underline-offset-2 hover:text-sub">
                expandir tudo
              </button>
              <button onClick={() => fireExpand(false)} className="underline underline-offset-2 hover:text-sub">
                recolher
              </button>
            </span>
          )}
        </div>
        {filter.trim() ? (
          <div className="max-h-96 overflow-y-auto">
            {visible.slice(0, 500).map((f) => (
              <div
                key={f.path}
                onClick={() => toggle(f.path)}
                className="flex cursor-pointer items-center gap-2.5 rounded-lg border-b border-line/40 px-1.5 py-1.5 font-mono text-[12px] text-sub last:border-0 hover:bg-raised hover:text-heading"
              >
                <CheckBox checked={picked.has(f.path)} onToggle={() => toggle(f.path)} label={`Marcar ${f.path}`} />
                <FileText size={13} className="shrink-0 text-mute" />
                <span className="truncate" title={f.path}>{relOf(f.path)}</span>
                <span className="ml-auto shrink-0 rounded-full bg-pill px-1.5 text-[10px] text-mute">{f.count}</span>
              </div>
            ))}
            {visible.length === 0 && <p className="px-4 py-8 text-center text-sm text-mute">Nenhum arquivo bate no filtro.</p>}
            {visible.length > 500 && <p className="py-2 text-center font-mono text-[11px] text-mute">+{visible.length - 500} ocultos — refine o filtro</p>}
          </div>
        ) : (
          <div className="max-h-96 overflow-y-auto pr-1">
            <FolderView node={tree} depth={0} picked={picked} onToggleFile={toggle} onToggleFolder={toggleFolder} expandCmd={expandCmd} />
            {loaded && files.length === 0 && <p className="px-4 py-8 text-center text-sm text-mute">Nenhum arquivo (engine offline?).</p>}
          </div>
        )}
      </div>

      {results.length > 0 && (
        <>
          <div className="flex items-center gap-2">
            <span className="text-[13px] text-mute">{results.length} hits</span>
            <div className="ml-auto flex gap-1.5">
              {(["json", "csv", "txt"] as const).map((k) => (
                <button
                  key={k}
                  onClick={() => doExport(k)}
                  className="flex items-center gap-1.5 rounded-lg border border-line px-3 py-1.5 text-[12px] text-sub hover:border-linestrong hover:text-heading"
                >
                  <Download size={12} /> {k.toUpperCase()}
                </button>
              ))}
            </div>
          </div>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {results.map((r) => (
              <ResultCard key={`${r.username}-${r.platform}`} result={r} onOpen={setModal} />
            ))}
          </div>
        </>
      )}

      <ProfileModal result={modal} onClose={() => setModal(null)} />
    </div>
  );
}
