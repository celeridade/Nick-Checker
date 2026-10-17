import { SignJWT, jwtVerify } from "jose";
import { eq } from "drizzle-orm";
import { db } from "@/db";
import { users } from "@/db/schema";

export const SESSION_COOKIE = "l96a1_session";
const SESSION_DAYS = 30;

export const PLANS: Record<string, { tokens: number; label: string }> = {
  free: { tokens: 25, label: "Free" },
  pro: { tokens: 500, label: "Pro" },
  max: { tokens: 2500, label: "Max" },
  enterprise: { tokens: 10000, label: "Enterprise" },
  // Plano do owner: distinto de TODOS os premium (ilimitado, nunca cobrado).
  owner: { tokens: 999999, label: "Owner" },
};

export const OWNER_PLAN = "owner";

function secret(): Uint8Array {
  const s = process.env.AUTH_SECRET;
  if (!s) throw new Error("AUTH_SECRET ausente — gere com: openssl rand -base64 32");
  return new TextEncoder().encode(s);
}

export interface Session {
  uid: number;
  email: string;
}

export async function signSession(s: Session): Promise<string> {
  return new SignJWT({ ...s })
    .setProtectedHeader({ alg: "HS256" })
    .setIssuedAt()
    .setExpirationTime(`${SESSION_DAYS}d`)
    .sign(secret());
}

export async function verifySession(token: string): Promise<Session | null> {
  try {
    const { payload } = await jwtVerify(token, secret());
    if (typeof payload.uid !== "number" || typeof payload.email !== "string") return null;
    return { uid: payload.uid, email: payload.email };
  } catch {
    return null;
  }
}

export function sessionCookie(token: string): string {
  const secure = process.env.NODE_ENV === "production" ? "; Secure" : "";
  return `${SESSION_COOKIE}=${token}; Path=/; HttpOnly; SameSite=Lax; Max-Age=${SESSION_DAYS * 86400}${secure}`;
}

export function clearSessionCookie(): string {
  return `${SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0`;
}

export function getSessionToken(req: Request): string | null {
  const cookie = req.headers.get("cookie") ?? "";
  for (const part of cookie.split(";")) {
    const [k, ...v] = part.trim().split("=");
    if (k === SESSION_COOKIE) return v.join("=");
  }
  return null;
}

export async function requireSession(req: Request): Promise<Session | null> {
  const token = getSessionToken(req);
  if (!token) return null;
  return verifySession(token);
}

export interface Quota {
  used: number;
  limit: number;
  remaining: number;
  plan: string;
  role: string;
  unlimited: boolean;
  resetAt: string | null;
}

/** Owner (você) tem acesso a tudo: sem cobrança, sem limite. */
export function isOwnerRole(role: string | null | undefined): boolean {
  return role === "owner";
}

/** Garante reset mensal e retorna a quota atual.
 * Owner (role=owner) sempre aparece como plano `owner` — distinto de
 * free/pro/max/enterprise — e ilimitado, mesmo se a coluna plan estiver "free".
 */
export async function getQuota(userId: number): Promise<(Quota & { row: typeof users.$inferSelect }) | null> {
  const rows = await db.select().from(users).where(eq(users.id, userId));
  const row = rows[0];
  if (!row) return null;
  const unlimited = isOwnerRole(row.role);
  const plan = unlimited ? OWNER_PLAN : row.plan;
  const now = new Date();
  if (!row.tokensResetAt || row.tokensResetAt <= now) {
    const resetAt = new Date(now.getTime() + 30 * 86400 * 1000);
    await db.update(users).set({ tokensUsed: 0, tokensResetAt: resetAt }).where(eq(users.id, userId));
    return {
      used: 0,
      limit: row.tokensLimit,
      remaining: unlimited ? row.tokensLimit : row.tokensLimit,
      plan,
      role: row.role,
      unlimited,
      resetAt: resetAt.toISOString(),
      row: { ...row, plan, tokensUsed: 0, tokensResetAt: resetAt },
    };
  }
  return {
    used: row.tokensUsed,
    remaining: unlimited ? row.tokensLimit : Math.max(0, row.tokensLimit - row.tokensUsed),
    limit: row.tokensLimit,
    plan,
    role: row.role,
    unlimited,
    resetAt: row.tokensResetAt.toISOString(),
    row: { ...row, plan },
  };
}

/**
 * Cobra `cost` tokens. Retorna {ok} ou {ok:false, quota} se estourado.
 * Chamado ANTES de disparar o scan (single=1, bulk/dataset=nicks).
 */
export async function chargeTokens(
  userId: number,
  cost: number,
): Promise<{ ok: true; quota: Quota } | { ok: false; quota: Quota }> {
  const q = await getQuota(userId);
  if (!q) throw new Error("usuário não existe");
  if (q.unlimited) return { ok: true as const, quota: q };
  if (q.used + cost > q.limit) return { ok: false as const, quota: q };
  await db
    .update(users)
    .set({ tokensUsed: q.used + cost })
    .where(eq(users.id, userId));
  const quota: Quota = {
    used: q.used + cost,
    limit: q.limit,
    remaining: q.limit - q.used - cost,
    plan: q.plan,
    role: q.role,
    unlimited: q.unlimited,
    resetAt: q.resetAt,
  };
  return { ok: true as const, quota };
}

/** Exige papel owner (acesso a tudo). Retorna a sessão ou null. */
export async function requireOwner(req: Request): Promise<Session | null> {
  const session = await requireSession(req);
  if (!session) return null;
  const rows = await db.select({ role: users.role }).from(users).where(eq(users.id, session.uid));
  if (!isOwnerRole(rows[0]?.role)) return null;
  return session;
}

/** Devolve tokens quando o engine falha depois da cobrança. */
export async function refundTokens(userId: number, cost: number): Promise<void> {
  const q = await getQuota(userId);
  if (!q || q.unlimited) return;
  await db
    .update(users)
    .set({ tokensUsed: Math.max(0, q.used - cost) })
    .where(eq(users.id, userId));
}

export function isValidEmail(email: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]{2,}$/.test(email.trim());
}
