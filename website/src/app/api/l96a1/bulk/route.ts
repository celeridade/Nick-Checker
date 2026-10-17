import { chargeTokens, getQuota, requireSession } from "@/lib/auth";

export const dynamic = "force-dynamic";

const API = process.env.L96A1_API_URL ?? "http://127.0.0.1:8080";

/**
 * BFF bulk SSE: GET /api/l96a1/bulk?users=a,b,c&platforms=all
 * Faz pipe do Rust GET /v1/bulk sem buffer (max 100 nicks/job).
 */
export async function GET(req: Request) {
  const url = new URL(req.url);
  const users = url.searchParams.get("users") ?? url.searchParams.get("q") ?? "";
  const platforms = url.searchParams.get("p") ?? url.searchParams.get("platforms") ?? "all";
  // 1 token por nick (mesma regra do botão Search ⚡1).
  const nicks = users.split(/[\n,;]+/).map((s) => s.trim()).filter(Boolean).length;
  const cost = Math.min(Math.max(nicks, 1), 100);

  const session = await requireSession(req).catch(() => null);
  if (!session) return Response.json({ error: "faça login para pesquisar" }, { status: 401 });
  const pre = await getQuota(session.uid).catch(() => null);
  if (!pre) return Response.json({ error: "banco offline" }, { status: 503 });
  if (!pre.unlimited && pre.used + cost > pre.limit)
    return Response.json({ error: `sem tokens — bulk precisa de ${cost}`, quota: pre }, { status: 402 });

  try {
    const upstream = await fetch(
      `${API}/v1/bulk?users=${encodeURIComponent(users)}&platforms=${encodeURIComponent(platforms)}`,
      { headers: { Accept: "text/event-stream" }, signal: req.signal },
    );
    if (upstream.ok && upstream.body) {
      const charge = await chargeTokens(session.uid, cost).catch(() => null);
      if (!charge?.ok) return Response.json({ error: "sem tokens — veja /pricing" }, { status: 402 });
      return new Response(upstream.body, {
        headers: {
          "Content-Type": "text/event-stream; charset=utf-8",
          "Cache-Control": "no-cache, no-transform",
          Connection: "keep-alive",
          "X-Tokens-Remaining": String(charge.quota.remaining),
        },
      });
    }
  } catch {
    // cai no 502 abaixo
  }
  return Response.json({ error: "engine offline (l96a1-api fora do ar)" }, { status: 502 });
}
