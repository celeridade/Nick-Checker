import { NextResponse } from "next/server";

export const dynamic = "force-dynamic";

const API = process.env.L96A1_API_URL ?? "http://127.0.0.1:8080";

export async function GET() {
  try {
    const upstream = await fetch(`${API}/v1/webhooks`, { cache: "no-store" });
    return NextResponse.json(await upstream.json(), { status: upstream.status });
  } catch {
    return NextResponse.json({ url_configured: false, mode: "batch" }, { status: 200 });
  }
}
