import { NextResponse } from "next/server";

export const dynamic = "force-dynamic";

const API = process.env.L96A1_API_URL ?? "http://127.0.0.1:8080";

export async function GET() {
  try {
    const upstream = await fetch(`${API}/v1/rate-limit`, { cache: "no-store" });
    if (upstream.ok) return NextResponse.json(await upstream.json());
  } catch {
    // fallback
  }
  return NextResponse.json({
    search: { used: 0, limit: 10, remaining: 10 },
    quota: { used: 2, limit: 25, remaining: 23 },
    plan: "free",
  });
}
