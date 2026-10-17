import { NextResponse } from "next/server";
import bcrypt from "bcryptjs";
import { eq } from "drizzle-orm";
import { db } from "@/db";
import { users } from "@/db/schema";
import { sessionCookie, signSession } from "@/lib/auth";

export const dynamic = "force-dynamic";

export async function POST(req: Request) {
  try {
    const { email, password } = (await req.json().catch(() => ({}))) as {
      email?: string;
      password?: string;
    };
    const clean = (email ?? "").trim().toLowerCase();
    const rows = await db.select().from(users).where(eq(users.email, clean));
    const user = rows[0];
    if (!user || !user.passwordHash || !password)
      return NextResponse.json({ error: "email ou senha inválidos" }, { status: 401 });
    const ok = await bcrypt.compare(password, user.passwordHash);
    if (!ok) return NextResponse.json({ error: "email ou senha inválidos" }, { status: 401 });
    const token = await signSession({ uid: user.id, email: user.email });
    const res = NextResponse.json({ ok: true, email: user.email, plan: user.plan });
    res.headers.set("Set-Cookie", sessionCookie(token));
    return res;
  } catch (e) {
    const msg = e instanceof Error ? e.message : "";
    if (msg.includes("DATABASE_URL") || msg.includes("AUTH_SECRET"))
      return NextResponse.json({ error: "auth indisponível (banco/secret fora do ar)" }, { status: 503 });
    return NextResponse.json({ error: "falha no login" }, { status: 500 });
  }
}
