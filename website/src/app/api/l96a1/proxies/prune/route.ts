import { NextResponse } from "next/server";

export const dynamic = "force-dynamic";

const API = process.env.L96A1_API_URL ?? "http://127.0.0.1:8080";

/** POST /api/l96a1/proxies/prune -> remove 100% mortos (com .bak). */
export async function POST() {
  try {
    const upstream = await fetch(`${API}/v1/proxies/prune`, { method: "POST" });
    return NextResponse.json(await upstream.json(), { status: upstream.status });
  } catch {
    return NextResponse.json({ error: "engine offline" }, { status: 502 });
  }
}
