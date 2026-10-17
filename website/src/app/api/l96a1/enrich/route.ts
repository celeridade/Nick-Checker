import { NextResponse } from "next/server";

export const dynamic = "force-dynamic";

const API = process.env.L96A1_API_URL ?? "http://127.0.0.1:8080";

/**
 * BFF enrich (API separada do check — Fase 1).
 * GET /api/l96a1/enrich?platform=github&user=adoecido
 * POST /api/l96a1/enrich {items:[{platform,user}]} (cap 20)
 * Só dado público; nunca bloqueia o scan.
 */
export async function GET(req: Request) {
  const url = new URL(req.url);
  const platform = url.searchParams.get("platform") ?? "";
  const user = url.searchParams.get("user") ?? url.searchParams.get("nick") ?? "";
  try {
    const upstream = await fetch(
      `${API}/v1/enrich?platform=${encodeURIComponent(platform)}&user=${encodeURIComponent(user)}`,
      { cache: "no-store" },
    );
    const data = await upstream.json();
    return NextResponse.json(data, { status: upstream.status });
  } catch {
    return NextResponse.json({ error: "engine offline" }, { status: 502 });
  }
}

export async function POST(req: Request) {
  const body = await req.json().catch(() => ({}));
  try {
    const upstream = await fetch(`${API}/v1/enrich/batch`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    const data = await upstream.json();
    return NextResponse.json(data, { status: upstream.status });
  } catch {
    return NextResponse.json({ error: "engine offline" }, { status: 502 });
  }
}
