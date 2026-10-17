"use client";

import type { SearchResult } from "@/lib/results";

export interface RichProfile {
  display_name?: string | null;
  bio?: string | null;
  pfp_url?: string | null;
  banner_url?: string | null;
  user_id?: string | null;
  followers?: number | null;
  following?: number | null;
  posts?: number | null;
  likes?: number | null;
  verified?: boolean | null;
  website?: string | null;
  emails?: string[];
}

const cache = new Map<string, Promise<RichProfile | null>>();

/** Enrich sob demanda (API separada — nunca no caminho do scan). Dedup por (platform|user). */
export function fetchEnrich(platform: string, user: string): Promise<RichProfile | null> {
  const k = `${platform.toLowerCase()}|${user.toLowerCase()}`;
  const hit = cache.get(k);
  if (hit) return hit;
  const p = fetch(`/api/l96a1/enrich?platform=${encodeURIComponent(platform)}&user=${encodeURIComponent(user)}`)
    .then((r) => (r.ok ? r.json() : null))
    .then((j) => (j?.profile as RichProfile) ?? null)
    .catch(() => null);
  cache.set(k, p);
  return p;
}

/** Aplica o RichProfile num SearchResult (retorna cópia enriquecida). */
export function applyRich(base: SearchResult, rich: RichProfile | null): SearchResult {
  if (!rich) return base;
  return {
    ...base,
    displayName: base.displayName ?? rich.display_name ?? null,
    bio: base.bio ?? rich.bio ?? null,
    pfpUrl: base.pfpUrl ?? rich.pfp_url ?? null,
    bannerUrl: base.bannerUrl ?? rich.banner_url ?? null,
    userId: base.userId ?? rich.user_id ?? null,
    followers: base.followers ?? rich.followers ?? null,
    following: base.following ?? rich.following ?? null,
    posts: base.posts ?? rich.posts ?? null,
    likes: base.likes ?? rich.likes ?? null,
    isVerified: base.isVerified ?? rich.verified ?? null,
    website: base.website ?? rich.website ?? null,
    emails: base.emails.length > 0 ? base.emails : rich.emails ?? [],
  };
}
