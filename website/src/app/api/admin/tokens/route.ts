import { NextResponse } from "next/server";
import { eq } from "drizzle-orm";
import { db } from "@/db";
import { users } from "@/db/schema";
import { PLANS, requireOwner } from "@/lib/auth";

export const dynamic = "force-dynamic";

/**
 * Adiciona tokens / troca plano de qualquer conta (só owner).
 * Body: { email, addTokens?, setLimit?, plan?, resetUsed? }
 */
export async function POST(req: Request) {
  try {
    const owner = await requireOwner(req);
    if (!owner) return NextResponse.json({ error: "acesso negado (owner)" }, { status: 403 });
    const body = (await req.json().catch(() => ({}))) as {
      email?: string;
      addTokens?: number;
      setLimit?: number;
      plan?: string;
      resetUsed?: boolean;
    };
    const email = (body.email ?? "").trim().toLowerCase();
    if (!email) return NextResponse.json({ error: "email obrigatório" }, { status: 400 });
    const rows = await db.select().from(users).where(eq(users.email, email));
    const target = rows[0];
    if (!target) return NextResponse.json({ error: "conta não encontrada" }, { status: 404 });

    const patch: Partial<typeof users.$inferInsert> = {};
    if (typeof body.addTokens === "number" && Number.isFinite(body.addTokens)) {
      patch.tokensLimit = Math.max(0, target.tokensLimit + Math.trunc(body.addTokens));
    }
    if (typeof body.setLimit === "number" && Number.isFinite(body.setLimit)) {
      patch.tokensLimit = Math.max(0, Math.trunc(body.setLimit));
    }
    if (typeof body.plan === "string" && PLANS[body.plan]) {
      patch.plan = body.plan;
      patch.tokensLimit = PLANS[body.plan].tokens;
    }
    if (body.resetUsed) {
      patch.tokensUsed = 0;
      patch.tokensResetAt = new Date(Date.now() + 30 * 86400 * 1000);
    }
    if (Object.keys(patch).length === 0)
      return NextResponse.json({ error: "nada para aplicar (addTokens|setLimit|plan|resetUsed)" }, { status: 400 });

    await db.update(users).set(patch).where(eq(users.id, target.id));
    const updated = (await db.select().from(users).where(eq(users.id, target.id)))[0];
    return NextResponse.json({
      ok: true,
      email: updated.email,
      plan: updated.plan,
      tokensUsed: updated.tokensUsed,
      tokensLimit: updated.tokensLimit,
    });
  } catch {
    return NextResponse.json({ error: "banco offline" }, { status: 503 });
  }
}
