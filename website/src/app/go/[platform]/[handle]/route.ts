import { redirect } from "next/navigation";

export const dynamic = "force-dynamic";

const API = process.env.L96A1_API_URL ?? "http://127.0.0.1:8080";

/**
 * GET /go/:platform/:handle — redirect estilo fingerprint.to.
 * Fonte única: l96a1-api (checker.profile_url normalizado).
 * A UI só mostra o link em hits (Taken/Private/Banned).
 */
export async function GET(
  _req: Request,
  { params }: { params: Promise<{ platform: string; handle: string }> },
) {
  const { platform, handle } = await params;
  try {
    const upstream = await fetch(
      `${API}/v1/${encodeURIComponent(platform)}/${encodeURIComponent(handle)}`,
      { cache: "no-store" },
    );
    if (upstream.ok) {
      const data = await upstream.json();
      if (typeof data?.profileUrl === "string" && data.profileUrl.startsWith("http")) {
        redirect(data.profileUrl);
      }
    }
  } catch {
    // cai no 404 abaixo
  }
  return Response.json({ error: "perfil não encontrado" }, { status: 404 });
}
