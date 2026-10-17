"use client";

import { jsPDF } from "jspdf";
import { downloadFile } from "@/lib/export";

/** Nick livre (Available) — o único conteúdo dos resultados/download. */
export interface FreeNick {
  nick: string;
  platform: string;
  profileUrl?: string;
}

const stamp = () => new Date().toISOString().slice(0, 19).replace(/[:T]/g, "-");

export const UTF8_BOM = "\uFEFF";

function withBOM(s: string): string {
  return s.startsWith(UTF8_BOM) ? s : UTF8_BOM + s;
}

/** jsPDF usa fontes padrão (latin1): dobra para ASCII para não quebrar o PDF. */
function asciiFold(s: string): string {
  return s
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[—–]/g, "-")
    .replace(/[“”«»]/g, '"')
    .replace(/[‘’]/g, "'")
    .replace(/[…]/g, "...");
}

export function freesToTXT(frees: FreeNick[], title: string): string {
  const nicks = [...new Set(frees.map((f) => f.nick))].sort();
  const byNick = new Map<string, string[]>();
  for (const f of frees) {
    if (!byNick.has(f.nick)) byNick.set(f.nick, []);
    if (!byNick.get(f.nick)!.includes(f.platform)) byNick.get(f.nick)!.push(f.platform);
  }
  return withBOM(
    [
      `L96A1 SNIPER — disponíveis para claim (${nicks.length})`,
      `scan: ${title} | exported: ${new Date().toISOString()}`,
      "=".repeat(60),
      ...nicks.map((n) => `@${n} — ${(byNick.get(n) ?? []).join(", ") || "—"}`),
    ].join("\n")
  );
}

export function freesToMD(frees: FreeNick[], title: string): string {
  const nicks = [...new Set(frees.map((f) => f.nick))].sort();
  const byNick = new Map<string, FreeNick[]>();
  for (const f of frees) {
    if (!byNick.has(f.nick)) byNick.set(f.nick, []);
    if (!byNick.get(f.nick)!.some((x) => x.platform === f.platform)) byNick.get(f.nick)!.push(f);
  }
  return withBOM(
    [
      `# L96A1 — disponíveis (${nicks.length})`,
      ``,
      `scan: \`${title}\` · exported: ${new Date().toISOString()}`,
      ``,
      ...nicks.flatMap((n) => {
        const plats = (byNick.get(n) ?? []).map((f) =>
          f.profileUrl ? `[${f.platform}](${f.profileUrl})` : f.platform,
        );
        return [`- [ ] \`@${n}\` — livre em: ${plats.join(", ") || "—"}`];
      }),
    ].join("\n")
  );
}

/**
 * Compacto p/ bulk grande (>100 nicks): 1 arquivo só, 1 linha por par
 * (nick, plataforma), ordenado — sem repetições, sem enfeite.
 */
export function freesToTXTCompact(frees: FreeNick[], title: string): string {
  const seen = new Set<string>();
  const rows: Array<[string, string]> = [];
  for (const f of frees) {
    const k = `${f.nick}\0${f.platform}`;
    if (seen.has(k)) continue;
    seen.add(k);
    rows.push([f.nick, f.platform]);
  }
  rows.sort((a, b) => a[0].localeCompare(b[0]) || a[1].localeCompare(b[1]));
  return withBOM(
    [
      `# ${title} · ${rows.length} pares livres · ${new Date().toISOString()}`,
      ...rows.map(([n, p]) => `@${n} · ${p}`),
    ].join("\n")
  );
}

export function freesToCSV(frees: FreeNick[]): string {
  const esc = (v: string) => (/[",\n]/.test(v) ? `"${v.replace(/"/g, '""')}"` : v);
  const rows = frees.map((f) => [f.nick, f.platform, f.profileUrl ?? ""].map(esc).join(","));
  return withBOM(["nick,platform,profile_url", ...rows].join("\n"));
}

export function downloadFreesPDF(frees: FreeNick[], title: string) {
  const nicks = [...new Set(frees.map((f) => f.nick))].sort();
  const doc = new jsPDF();
  doc.setFont("courier", "bold");
  doc.setFontSize(12);
  doc.text(asciiFold(`L96A1 - disponiveis (${nicks.length})`), 10, 12);
  doc.setFont("courier", "normal");
  doc.setFontSize(10);
  doc.text(asciiFold(`scan: ${title.slice(0, 80)}`), 10, 19);
  let y = 28;
  for (const n of nicks) {
    if (y > 282) {
      doc.addPage();
      y = 14;
    }
    doc.text(asciiFold(`[ ] @${n}`), 10, y);
    y += 7;
  }
  doc.save(`l96a1-livres-${stamp()}.pdf`);
}

export function downloadFrees(kind: "txt" | "md" | "csv" | "pdf", frees: FreeNick[], title: string) {
  const base = `l96a1-livres-${stamp()}`;
  if (kind === "txt") downloadFile(`${base}.txt`, freesToTXT(frees, title), "text/plain");
  if (kind === "md") downloadFile(`${base}.md`, freesToMD(frees, title), "text/markdown");
  if (kind === "csv") downloadFile(`${base}.csv`, freesToCSV(frees), "text/csv");
  if (kind === "pdf") downloadFreesPDF(frees, title);
}
