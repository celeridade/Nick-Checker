import { NextResponse } from "next/server";

export const dynamic = "force-dynamic";

const API = process.env.L96A1_API_URL ?? "http://127.0.0.1:8080";

export async function POST(req: Request) {
  try {
    const body = await req.json().catch(() => ({}));
    const upstream = await fetch(`${API}/v1/bot/setup`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    return NextResponse.json(await upstream.json(), { status: upstream.status });
  } catch {
    return NextResponse.json({ error: "engine offline" }, { status: 502 });
  }
}
