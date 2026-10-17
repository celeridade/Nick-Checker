import { NextResponse } from "next/server";

export const dynamic = "force-dynamic";

const API = process.env.L96A1_API_URL ?? "http://127.0.0.1:8080";

/** GET /api/l96a1/scans/:id -> polling do job (checked/hits/results) */
export async function GET(_req: Request, { params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  try {
    const upstream = await fetch(`${API}/v1/scans/${encodeURIComponent(id)}`, {
      cache: "no-store",
    });
    const data = await upstream.json();
    return NextResponse.json(data, { status: upstream.status });
  } catch {
    return NextResponse.json({ error: "engine offline" }, { status: 502 });
  }
}
