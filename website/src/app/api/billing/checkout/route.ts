import { NextResponse } from "next/server";
import { eq } from "drizzle-orm";
import { db } from "@/db";
import { users } from "@/db/schema";
import { PLANS, requireSession } from "@/lib/auth";

export const dynamic = "force-dynamic";

/**
 * Checkout em MODO TESTE: sem Stripe configurado, aplica o plano
 * imediatamente (pro/max). Trocar por sessão Stripe real quando houver
 * STRIPE_SECRET_KEY — ver comentário TODO abaixo.
 */
export async function POST(req: Request) {
  try {
    const session = await requireSession(req);
    if (!session) return NextResponse.json({ error: "faça login para assinar" }, { status: 401 });
    const { plan } = (await req.json().catch(() => ({}))) as { plan?: string };
    const key = (plan ?? "").toLowerCase();
    // TODO(stripe): criar Checkout Session com STRIPE_SECRET_KEY e
    // confirmar via webhook antes de atualizar; manter este stub atrás
    // de env (ex: BILLING_TEST_MODE=1) em produção real.
    if (!["pro", "max"].includes(key))
      return NextResponse.json({ error: "plano: pro|max (enterprise = contato)" }, { status: 400 });
    await db
      .update(users)
      .set({
        plan: key,
        tokensLimit: PLANS[key].tokens,
        tokensUsed: 0,
        tokensResetAt: new Date(Date.now() + 30 * 86400 * 1000),
      })
      .where(eq(users.id, session.uid));
    return NextResponse.json({ ok: true, plan: key, test_mode: !process.env.STRIPE_SECRET_KEY });
  } catch {
    return NextResponse.json({ error: "banco offline" }, { status: 503 });
  }
}
