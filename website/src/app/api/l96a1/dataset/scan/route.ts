import { NextResponse } from "next/server";
import { chargeTokens, getQuota, refundTokens, requireSession } from "@/lib/auth";

export const dynamic = "force-dynamic";

const API = process.env.L96A1_API_URL ?? "http://127.0.0.1:8080";

export async function POST(req: Request) {
  const session = await requireSession(req).catch(() => null);
  if (!session) return NextResponse.json({ error: "faça login para pesquisar" }, { status: 401 });
  const body = (await req.json().catch(() => ({}))) as { limit?: number };
  const cost = Math.min(Math.max(Number(body.limit) || 100, 1), 500);
  const quota = await getQuota(session.uid).catch(() => null);
  if (!quota) return NextResponse.json({ error: "banco offline" }, { status: 503 });
  if (!quota.unlimited && quota.used + cost > quota.limit)
    return NextResponse.json({ error: `sem tokens — dataset precisa de ${cost}`, quota }, { status: 402 });
  const charge = await chargeTokens(session.uid, cost).catch(() => null);
  if (!charge?.ok) return NextResponse.json({ error: "sem tokens — veja /pricing" }, { status: 402 });
  try {
    const upstream = await fetch(`${API}/v1/dataset/scan`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    const data = await upstream.json();
    if (!upstream.ok) await refundTokens(session.uid, cost).catch(() => undefined);
    return NextResponse.json(data, { status: upstream.status });
  } catch {
    await refundTokens(session.uid, cost).catch(() => undefined);
    return NextResponse.json({ error: "engine offline" }, { status: 502 });
  }
}
