import { buildStreamEvents } from "@/lib/results";
import { chargeTokens, getQuota, requireSession } from "@/lib/auth";

export const dynamic = "force-dynamic";

const API = process.env.L96A1_API_URL ?? "http://127.0.0.1:8080";
const COST = 1;

/**
 * BFF SSE: tenta o motor Rust real (l96a1-api /v1/search) e faz pipe
 * sem buffer; se a API estiver fora (dev sem docker), cai no mock
 * local buildStreamEvents para não quebrar a UI.
 *
 * Eventos reais: connected -> start{total} ->
 * loop(result + free + warn + checked) -> done{type,total,hits,frees,checked,rates,errors,unknowns}
 */
export async function GET(req: Request) {
  const url = new URL(req.url);
  const q = url.searchParams.get("q") ?? "adoecido";
  const platforms = url.searchParams.get("p") ?? url.searchParams.get("platforms") ?? "all";

  const session = await requireSession(req).catch(() => null);
  if (!session) return Response.json({ error: "faça login para pesquisar" }, { status: 401 });
  const pre = await getQuota(session.uid).catch(() => null);
  if (!pre) return Response.json({ error: "banco offline" }, { status: 503 });
  if (!pre.unlimited && pre.used + COST > pre.limit)
    return Response.json({ error: "sem tokens — veja /pricing", quota: pre }, { status: 402 });

  try {
    const upstream = await fetch(
      `${API}/v1/search?q=${encodeURIComponent(q)}&platforms=${encodeURIComponent(platforms)}`,
      { headers: { Accept: "text/event-stream" }, signal: req.signal },
    );
    if (upstream.ok && upstream.body) {
      // Cobra SÓ no engine real (mock local de dev é grátis).
      const charge = await chargeTokens(session.uid, COST).catch(() => null);
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
    // API fora — fallback mock abaixo (grátis, dev only)
  }

  const events = buildStreamEvents(q);
  const encoder = new TextEncoder();

  const stream = new ReadableStream<Uint8Array>({
    start(controller) {
      let i = 0;
      const push = () => {
        if (i >= events.length) {
          controller.close();
          return;
        }
        const ev = events[i++];
        controller.enqueue(encoder.encode(`event: ${ev.name}\ndata: ${ev.data}\n\n`));
        // results/checked stream at ~400ms total pair rhythm, meta events fast
        const d = ev.name === "result" ? 380 : ev.name === "checked" ? 20 : 120;
        timer = setTimeout(push, d);
      };
      let timer = setTimeout(push, 200);
      req.signal.addEventListener("abort", () => {
        clearTimeout(timer);
        controller.close();
      });
    },
  });

  return new Response(stream, {
    headers: {
      "Content-Type": "text/event-stream; charset=utf-8",
      "Cache-Control": "no-cache, no-transform",
      Connection: "keep-alive",
    },
  });
}
