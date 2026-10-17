import { NextResponse } from "next/server";

export const dynamic = "force-dynamic";

const API = process.env.L96A1_API_URL ?? "http://127.0.0.1:8080";

/** GET /api/l96a1/proxies/unmask?url=<mascarado> — revela integral (sessão já exigida nas rotas de scan; aqui repasse simples). */
export async function GET(req: Request) {
  const url = new URL(req.url);
  const want = url.searchParams.get("url") ?? "";
  try {
    const upstream = await fetch(`${API}/v1/proxies/unmask?url=${encodeURIComponent(want)}`, {
      cache: "no-store",
    });
    const data = await upstream.json();
    return NextResponse.json(data, { status: upstream.status });
  } catch {
    return NextResponse.json({ error: "engine offline" }, { status: 502 });
  }
}
