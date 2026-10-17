import { NextResponse } from "next/server";
import { eq } from "drizzle-orm";
import { db } from "@/db";
import { searches, users } from "@/db/schema";
import { clearSessionCookie, getQuota, requireSession } from "@/lib/auth";

export const dynamic = "force-dynamic";

export async function GET(req: Request) {
  try {
    const session = await requireSession(req);
    if (!session) return NextResponse.json({ error: "não autenticado" }, { status: 401 });
    const quota = await getQuota(session.uid);
    if (!quota) {
      const res = NextResponse.json({ error: "conta removida" }, { status: 401 });
      res.headers.set("Set-Cookie", clearSessionCookie());
      return res;
    }
    return NextResponse.json({
      email: quota.row.email,
      plan: quota.row.plan,
      role: quota.row.role,
      provider: quota.row.provider,
      tokens: { used: quota.used, limit: quota.limit, remaining: quota.remaining, unlimited: quota.unlimited },
      resetAt: quota.resetAt,
    });
  } catch {
    return NextResponse.json({ error: "banco offline" }, { status: 503 });
  }
}

export async function DELETE(req: Request) {
  try {
    const session = await requireSession(req);
    if (!session) return NextResponse.json({ error: "não autenticado" }, { status: 401 });
    await db.delete(searches).where(eq(searches.userId, session.uid));
    await db.delete(users).where(eq(users.id, session.uid));
    const res = NextResponse.json({ ok: true });
    res.headers.set("Set-Cookie", clearSessionCookie());
    return res;
  } catch {
    return NextResponse.json({ error: "banco offline" }, { status: 503 });
  }
}
