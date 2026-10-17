import { Suspense } from "react";
import SocialSearch from "@/components/dashboard/SocialSearch";

export default async function SocialSearchPage({
  searchParams,
}: {
  searchParams: Promise<{ q?: string; live?: string; platforms?: string; p?: string }>;
}) {
  const params = await searchParams;
  const q = typeof params.q === "string" ? params.q : "";
  const live = params.live === "1";
  const platforms =
    typeof params.platforms === "string"
      ? params.platforms
      : typeof params.p === "string"
        ? params.p
        : "all";
  return (
    <Suspense>
      <SocialSearch key={`${q}-${live}-${platforms}`} initialQuery={q} live={live} initialPlatforms={platforms} />
    </Suspense>
  );
}
