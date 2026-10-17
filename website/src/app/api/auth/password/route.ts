import { NextResponse } from "next/server";
import bcrypt from "bcryptjs";
import { eq } from "drizzle-orm";
import { db } from "@/db";
import { users } from "@/db/schema";
import { requireSession } from "@/lib/auth";

export const dynamic = "force-dynamic";

/** Define/troca senha (habilita login por email em contas Google). */
export async function POST(req: Request) {
  try {
    const session = await requireSession(req);
    if (!session) return NextResponse.json({ error: "não autenticado" }, { status: 401 });
    const { current, next } = (await req.json().catch(() => ({}))) as {
      current?: string;
      next?: string;
    };
    if (!next || next.length < 8)
      return NextResponse.json({ error: "nova senha: mínimo 8 caracteres" }, { status: 400 });
    const rows = await db.select().from(users).where(eq(users.id, session.uid));
    const user = rows[0];
    if (!user) return NextResponse.json({ error: "conta removida" }, { status: 401 });
    if (user.passwordHash) {
      if (!current || !(await bcrypt.compare(current, user.passwordHash)))
        return NextResponse.json({ error: "senha atual incorreta" }, { status: 401 });
    }
    await db
      .update(users)
      .set({ passwordHash: await bcrypt.hash(next, 12), provider: "email" })
      .where(eq(users.id, session.uid));
    return NextResponse.json({ ok: true });
  } catch {
    return NextResponse.json({ error: "banco offline" }, { status: 503 });
  }
}
