import Link from "next/link";
import L96A1Wordmark from "@/components/L96A1Wordmark";

const LINKS = [
  { href: "/#demo", label: "Demo" },
  { href: "/#features", label: "Features" },
  { href: "/#security", label: "Security" },
  { href: "/pricing", label: "Pricing" },
  { href: "/#blog", label: "Blog" },
];

export default function SiteNav() {
  return (
    <header className="fixed inset-x-0 top-4 z-50 px-4">
      <nav className="mx-auto flex max-w-[1100px] items-center justify-between rounded-xl border border-line bg-pill px-5 py-3">
        <Link href="/" className="flex items-center gap-2.5" aria-label="L96A1 home">
          <L96A1Wordmark size="mini" className="shrink-0" />
          <span className="text-[15px] font-bold tracking-tight text-heading">L96A1</span>
        </Link>
        <div className="hidden items-center gap-7 md:flex">
          {LINKS.map((l) => (
            <Link
              key={l.label}
              href={l.href}
              className="text-sm text-sub transition-colors hover:text-heading"
            >
              {l.label}
            </Link>
          ))}
        </div>
        <div className="flex items-center gap-3">
          <Link href="/login" className="hidden text-sm text-sub transition-colors hover:text-heading sm:block">
            Log in
          </Link>
          <Link
            href="/dashboard/search"
            className="rounded-lg bg-white px-4 py-2 text-sm font-semibold text-black transition-colors hover:bg-body"
          >
            Dashboard
          </Link>
        </div>
      </nav>
    </header>
  );
}
