import { NextResponse } from "next/server";
import { db } from "@/db";
import { users } from "@/db/schema";
import { requireOwner } from "@/lib/auth";

export const dynamic = "force-dynamic";

/** Lista contas (só owner). Sem hash, sem segredo. */
export async function GET(req: Request) {
  try {
    const owner = await requireOwner(req);
    if (!owner) return NextResponse.json({ error: "acesso negado (owner)" }, { status: 403 });
    const rows = await db.select({
      id: users.id,
      email: users.email,
      plan: users.plan,
      role: users.role,
      tokensUsed: users.tokensUsed,
      tokensLimit: users.tokensLimit,
      createdAt: users.createdAt,
    }).from(users);
    return NextResponse.json({ users: rows, total: rows.length });
  } catch {
    return NextResponse.json({ error: "banco offline" }, { status: 503 });
  }
}
