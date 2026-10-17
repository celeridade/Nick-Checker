import { NextResponse } from "next/server";
import bcrypt from "bcryptjs";
import { eq } from "drizzle-orm";
import { db } from "@/db";
import { users } from "@/db/schema";
import { PLANS, isValidEmail, sessionCookie, signSession } from "@/lib/auth";

export const dynamic = "force-dynamic";

export async function POST(req: Request) {
  try {
    const { email, password } = (await req.json().catch(() => ({}))) as {
      email?: string;
      password?: string;
    };
    const clean = (email ?? "").trim().toLowerCase();
    if (!isValidEmail(clean))
      return NextResponse.json({ error: "email inválido" }, { status: 400 });
    if (!password || password.length < 8)
      return NextResponse.json({ error: "senha: mínimo 8 caracteres" }, { status: 400 });

    const existing = await db.select({ id: users.id }).from(users).where(eq(users.email, clean));
    if (existing.length > 0)
      return NextResponse.json({ error: "email já cadastrado — faça login" }, { status: 409 });

    const hash = await bcrypt.hash(password, 12);
    const resetAt = new Date(Date.now() + 30 * 86400 * 1000);
    const inserted = await db
      .insert(users)
      .values({
        email: clean,
        passwordHash: hash,
        provider: "email",
        plan: "free",
        tokensUsed: 0,
        tokensLimit: PLANS.free.tokens,
        tokensResetAt: resetAt,
      })
      .returning({ id: users.id, email: users.email });
    const user = inserted[0];
    const token = await signSession({ uid: user.id, email: user.email });
    const res = NextResponse.json({ ok: true, email: user.email, plan: "free" });
    res.headers.set("Set-Cookie", sessionCookie(token));
    return res;
  } catch (e) {
    const msg = e instanceof Error ? e.message : "";
    if (msg.includes("DATABASE_URL") || msg.includes("AUTH_SECRET"))
      return NextResponse.json({ error: "auth indisponível (banco/secret fora do ar)" }, { status: 503 });
    return NextResponse.json({ error: "falha no cadastro" }, { status: 500 });
  }
}
