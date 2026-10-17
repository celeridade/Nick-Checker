import { NextResponse } from "next/server";

export const dynamic = "force-dynamic";

const API = process.env.L96A1_API_URL ?? "http://127.0.0.1:8080";

/** GET /api/l96a1/checked -> {size, ttl_secs} */
export async function GET() {
  try {
    const upstream = await fetch(`${API}/v1/checked`, { cache: "no-store" });
    const data = await upstream.json();
    return NextResponse.json(data, { status: upstream.status });
  } catch {
    return NextResponse.json({ size: 0, ttl_secs: 86400 }, { status: 200 });
  }
}

/** DELETE /api/l96a1/checked[?nick=] -> limpa tudo ou 1 nick (libera re-check) */
export async function DELETE(req: Request) {
  const url = new URL(req.url);
  const nick = url.searchParams.get("nick") ?? "";
  try {
    const upstream = await fetch(
      `${API}/v1/checked${nick ? `?nick=${encodeURIComponent(nick)}` : ""}`,
      { method: "DELETE" },
    );
    const data = await upstream.json();
    return NextResponse.json(data, { status: upstream.status });
  } catch {
    return NextResponse.json({ error: "engine offline" }, { status: 502 });
  }
}

/** POST /api/l96a1/checked {ttl_secs} -> define expiração (0 = só manual) */
export async function POST(req: Request) {
  const body = await req.json().catch(() => ({}));
  try {
    const upstream = await fetch(`${API}/v1/checked/ttl`, {
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
