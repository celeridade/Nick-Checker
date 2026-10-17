import type { SearchResult } from "@/lib/results";
import { fmtDate } from "@/lib/results";

const COLS: Array<[string, (r: SearchResult) => string]> = [
  ["platform", (r) => r.platform],
  ["username", (r) => `@${r.username}`],
  ["display_name", (r) => r.displayName ?? ""],
  ["followers", (r) => (r.followers !== null ? String(r.followers) : "")],
  ["following", (r) => (r.following !== null ? String(r.following) : "")],
  ["posts", (r) => (r.posts !== null ? String(r.posts) : "")],
  ["location", (r) => r.location ?? ""],
  ["created", (r) => fmtDate(r.createdAt) ?? ""],
  ["last_active", (r) => fmtDate(r.lastActive) ?? ""],
  ["profile_url", (r) => r.profileUrl],
];

export const UTF8_BOM = "\uFEFF";

export function withBOM(s: string): string {
  return s.startsWith(UTF8_BOM) ? s : UTF8_BOM + s;
}

export function resultsToCSV(results: SearchResult[]): string {
  const esc = (v: string) => (/[",\n]/.test(v) ? `"${v.replace(/"/g, '""')}"` : v);
  const head = COLS.map(([k]) => k).join(",");
  const rows = results.map((r) => COLS.map(([, fn]) => esc(fn(r))).join(","));
  // BOM no início: Excel/Notepad (Windows) detectam UTF-8 e não quebram
  // acentos/—/emojis. Sem BOM, "—" vira "â€”" e "í" vira "Ã­".
  return withBOM([head, ...rows].join("\n"));
}

export function resultsToTXT(results: SearchResult[], query: string): string {
  const lines = [
    "L96A1 SNIPER — search export",
    `query: ${query} | hits: ${results.length} | exported: ${new Date().toISOString()}`,
    "=".repeat(72),
    ...results.map((r) => {
      const bits = [
        r.platform.toUpperCase().padEnd(18),
        `@${r.username}`,
        r.displayName ? `(${r.displayName})` : "",
        r.followers !== null ? `${r.followers} followers` : "",
        r.location ? `loc: ${r.location}` : "",
        r.createdAt ? `joined: ${fmtDate(r.createdAt)}` : "",
        r.profileUrl,
      ].filter(Boolean);
      return bits.join("  ");
    }),
  ];
  return withBOM(lines.join("\n"));
}

export function resultsToJSON(results: SearchResult[], query: string): string {
  // Sem BOM: JSON.parse quebra com \uFEFF ("Unexpected token").
  // JSON é UTF-8 por padrão e editores modernos detectam sem BOM.
  return JSON.stringify(
    {
      tool: "L96A1 Sniper",
      query,
      count: results.length,
      exportedAt: new Date().toISOString(),
      results,
    },
    null,
    2
  );
}

export function downloadFile(filename: string, content: string, mime: string) {
  // Garante BOM + charset=utf-8 para .txt/.csv/.md não quebrar
  // caracteres no Windows (Notepad/Excel). Sem BOM, "—" vira "â€”"
  // e "í" vira "Ã­". JSON não leva BOM (quebra JSON.parse).
  const isJson = mime.includes("json") || filename.toLowerCase().endsWith(".json");
  const body = isJson
    ? content.startsWith(UTF8_BOM)
      ? content.slice(1)
      : content
    : content.startsWith(UTF8_BOM)
      ? content
      : UTF8_BOM + content;
  const blob = new Blob([body], { type: `${mime};charset=utf-8` });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}
