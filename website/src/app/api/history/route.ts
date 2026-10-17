import { NextResponse } from "next/server";

export const dynamic = "force-dynamic";

const API = process.env.L96A1_API_URL ?? "http://127.0.0.1:8080";

export async function GET() {
  try {
    const upstream = await fetch(`${API}/v1/history`, { cache: "no-store" });
    if (upstream.ok) {
      const data = await upstream.json();
      if (Array.isArray((data as { history?: unknown }).history)) return NextResponse.json(data);
    }
  } catch {
    // engine fora — sem mock: history real persiste no backend
  }
  return NextResponse.json({ history: [], total: 0, offline: true });
}

/** DELETE /api/history[?query=] -> apaga 1 linha ou tudo (libera re-check) */
export async function DELETE(req: Request) {
  const url = new URL(req.url);
  const query = url.searchParams.get("query") ?? "";
  try {
    const upstream = await fetch(
      `${API}/v1/history${query ? `?query=${encodeURIComponent(query)}` : ""}`,
      { method: "DELETE" },
    );
    const data = await upstream.json();
    return NextResponse.json(data, { status: upstream.status });
  } catch {
    return NextResponse.json({ error: "engine offline" }, { status: 502 });
  }
}
