import { NextResponse } from "next/server";
import { chargeTokens, getQuota, refundTokens, requireSession } from "@/lib/auth";

export const dynamic = "force-dynamic";

const API = process.env.L96A1_API_URL ?? "http://127.0.0.1:8080";

/** POST /api/l96a1/scans {usernames|users, platforms, source} -> {jobId,total} */
export async function POST(req: Request) {
  const session = await requireSession(req).catch(() => null);
  if (!session) return NextResponse.json({ error: "faça login para pesquisar" }, { status: 401 });
  const body = (await req.json().catch(() => ({}))) as {
    usernames?: string[];
    users?: string;
    platforms?: string;
    source?: string;
  };
  const count = Math.min(
    (body.usernames?.length ?? 0) +
      (typeof body.users === "string"
        ? body.users.split(/[\n,;]+/).filter((s) => s.trim()).length
        : 0) || 1,
    500,
  );
  const quota = await getQuota(session.uid).catch(() => null);
  if (!quota) return NextResponse.json({ error: "banco offline" }, { status: 503 });
  if (!quota.unlimited && quota.used + count > quota.limit)
    return NextResponse.json({ error: `sem tokens — job precisa de ${count}`, quota }, { status: 402 });
  const charge = await chargeTokens(session.uid, count).catch(() => null);
  if (!charge?.ok) return NextResponse.json({ error: "sem tokens — veja /pricing" }, { status: 402 });
  try {
    const upstream = await fetch(`${API}/v1/scans`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    const data = await upstream.json();
    if (!upstream.ok) await refundTokens(session.uid, count).catch(() => undefined);
    return NextResponse.json(data, { status: upstream.status });
  } catch {
    await refundTokens(session.uid, count).catch(() => undefined);
    return NextResponse.json({ error: "engine offline" }, { status: 502 });
  }
}

/** GET /api/l96a1/scans -> lista jobs recentes */
export async function GET() {
  try {
    const upstream = await fetch(`${API}/v1/scans`, { cache: "no-store" });
    const data = await upstream.json();
    return NextResponse.json(data, { status: upstream.status });
  } catch {
    return NextResponse.json({ scans: [], total: 0 }, { status: 200 });
  }
}
