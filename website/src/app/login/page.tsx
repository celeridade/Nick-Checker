"use client";

import { Suspense, useState } from "react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import L96A1Wordmark from "@/components/L96A1Wordmark";

function LoginForm() {
  const router = useRouter();
  const params = useSearchParams();
  const next = params.get("next") || "/dashboard/search";
  const [mode, setMode] = useState<"login" | "register">("login");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setBusy(true);
    try {
      const res = await fetch(`/api/auth/${mode}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, password }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data?.error ?? `HTTP ${res.status}`);
      router.push(next);
      router.refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : "falha na autenticação");
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="dotgrid flex min-h-screen flex-col items-center justify-center bg-page px-4">
      <Link href="/" className="mb-8 flex items-center gap-3">
        <L96A1Wordmark size="sm" />
      </Link>
      <div className="w-full max-w-sm rounded-xl border border-line bg-card p-8">
        <p className="eyebrow">AUTHENTICATE</p>
        <h1 className="mt-2 text-xl font-bold text-heading">
          {mode === "login" ? "Log in to L96A1" : "Create your account"}
        </h1>
        <p className="mt-1.5 text-sm text-mute">Zero to scoped-in in ten seconds.</p>

        <div className="mt-6 flex gap-1 rounded-full border border-line bg-pill p-1">
          {(["login", "register"] as const).map((m) => (
            <button
              key={m}
              onClick={() => { setMode(m); setError(null); }}
              className={`flex-1 rounded-full px-4 py-1.5 text-sm font-medium capitalize transition-colors ${
                mode === m ? "bg-white text-black" : "text-sub hover:text-heading"
              }`}
            >
              {m === "login" ? "Log in" : "Sign up"}
            </button>
          ))}
        </div>

        <form onSubmit={submit} className="mt-4 space-y-3">
          <input
            type="email"
            required
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="Email address"
            autoComplete="email"
            className="w-full rounded-lg border border-line bg-input px-3.5 py-2.5 text-sm text-heading placeholder:text-mute focus:border-linestrong focus:outline-none"
          />
          <input
            type="password"
            required
            minLength={8}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="Password (8+ chars)"
            autoComplete={mode === "login" ? "current-password" : "new-password"}
            className="w-full rounded-lg border border-line bg-input px-3.5 py-2.5 text-sm text-heading placeholder:text-mute focus:border-linestrong focus:outline-none"
          />
          {error && <p className="text-[13px] text-danger">{error}</p>}
          <button
            type="submit"
            disabled={busy}
            className="w-full rounded-lg bg-white px-4 py-2.5 text-sm font-semibold text-black transition-colors hover:bg-body disabled:opacity-50"
          >
            {busy ? "…" : mode === "login" ? "Log in" : "Create account"}
          </button>
        </form>

        <div className="my-5 flex items-center gap-3 text-[11px] text-mute">
          <span className="h-px flex-1 bg-line" />OAUTH EM BREVE<span className="h-px flex-1 bg-line" />
        </div>
        <button
          disabled
          title="Login com Google chega em breve"
          className="flex w-full cursor-not-allowed items-center justify-center gap-3 rounded-lg border border-line px-4 py-2.5 text-sm font-semibold text-mute"
        >
          <span className="grid size-5 place-items-center rounded-sm border border-line font-mono text-[11px] font-bold">G</span>
          Continue with Google
        </button>

        <p className="mt-6 text-center text-[11px] leading-relaxed text-mute">
          By continuing you agree to our{" "}
          <span className="text-sub underline underline-offset-2">Terms</span> and{" "}
          <span className="text-sub underline underline-offset-2">Privacy Policy</span>.
        </p>
      </div>
      <Link href="/" className="mt-6 font-mono text-xs text-mute hover:text-sub">← back to l96a1</Link>
    </div>
  );
}

export default function LoginPage() {
  return (
    <Suspense>
      <LoginForm />
    </Suspense>
  );
}
